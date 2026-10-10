use std::{cell::{Cell, RefCell}, collections::HashMap, pin::pin, rc::Rc};

use computer_model::protocol::{
    message::{ClientMessage, ProtocolMessage, ServerMessage},
    PROTOCOL_VERSION,
};
use dioxus::hooks::{UnboundedReceiver, use_context, use_coroutine};
use dioxus::prelude::use_hook;
use dioxus::signals::{Signal, WritableExt};
use futures_util::{
    future::{select, Either},
    SinkExt, StreamExt,
};
use gloo_net::websocket::{futures::WebSocket, Message};
use gloo_timers::future::TimeoutFuture;
use tracing::{error, warn};

use crate::state::{app::AppState, ConnectionStatus};

use super::client::{Client, Pending};

const HANDSHAKE_TIMEOUT_MS: u32 = 10_000;

const MAX_REFUSALS: u32 = 3;

fn backoff_ms(attempt: u32) -> u32 {
    match attempt {
        0 | 1 => 1_000,
        2 => 2_000,
        3 => 5_000,
        _ => 10_000,
    }
}

enum End {
    Refused(String),
    Lost(String),
}

pub fn ws_client() -> Rc<Client> {
    let state = use_context::<AppState>();
    let status = use_context::<Signal<ConnectionStatus>>();

    let pending: Pending =
        use_hook(|| Rc::new(RefCell::new(HashMap::new())));
    let pending_task = Rc::clone(&pending);

    let coroutine = use_coroutine(
        move |mut rx: UnboundedReceiver<ClientMessage>| {
            let pending = Rc::clone(&pending_task);

            async move {
                run(&mut rx, state, status, pending).await;
            }
        },
    );

    use_hook(move || Rc::new(Client::new(coroutine, pending, status)))
}

async fn run(
    rx: &mut UnboundedReceiver<ClientMessage>,
    state: AppState,
    mut status: Signal<ConnectionStatus>,
    pending: Pending,
) {
    let mut attempt = 0u32;
    let mut refusals = 0u32;

    loop {
        let (end, connected) = session(rx, state, status, &pending).await;

        fail_pending(&pending, "Connection closed".to_string());

        while rx.try_recv().is_ok() {}

        if connected {
            attempt = 0;
        }

        match end {
            End::Refused(reason) => {
                refusals += 1;

                error!("Handshake refused ({refusals}/{MAX_REFUSALS}): {reason}");

                if refusals >= MAX_REFUSALS {
                    status.set(ConnectionStatus::Failed(reason));
                    return;
                }
            }

            End::Lost(reason) => {
                warn!("Connection lost: {reason}");

                refusals = 0;
            }
        }

        attempt += 1;

        status.set(ConnectionStatus::Reconnecting(attempt));

        TimeoutFuture::new(backoff_ms(attempt)).await;
    }
}

async fn session(
    rx: &mut UnboundedReceiver<ClientMessage>,
    mut state: AppState,
    mut status: Signal<ConnectionStatus>,
    pending: &Pending,
) -> (End, bool) {
    let url = match ws_url().await {
        Ok(url) => url,
        Err(error) => return (End::Lost(error), false),
    };

    let ws = match WebSocket::open(&url) {
        Ok(ws) => ws,
        Err(error) => return (End::Lost(error.to_string()), false),
    };

    let (mut write, mut read) = ws.split();

    let hello = ClientMessage::Hello {
        version: PROTOCOL_VERSION,
    };

    let hello = match hello.encode() {
        Ok(bytes) => bytes,
        Err(error) => return (End::Lost(error), false),
    };

    if let Err(error) = write.send(Message::Bytes(hello)).await {
        return (End::Lost(error.to_string()), false);
    }

    let connected = Cell::new(false);

    let read_task = async {
        loop {
            let frame = if connected.get() {
                read.next().await
            } else {
                match select(
                    pin!(read.next()),
                    pin!(TimeoutFuture::new(HANDSHAKE_TIMEOUT_MS)),
                )
                .await
                {
                    Either::Left((frame, _)) => frame,

                    Either::Right(_) => {
                        return End::Lost("Handshake timed out".to_string());
                    }
                }
            };

            let Some(frame) = frame else {
                return End::Lost("Connection closed".to_string());
            };

            match frame {
                Ok(Message::Bytes(bytes)) => {
                    match ServerMessage::decode(&bytes) {
                        Ok(ServerMessage::Handshake(Ok(_))) => {
                            connected.set(true);
                            status.set(ConnectionStatus::Connected);
                        }

                        Ok(ServerMessage::Handshake(Err(reason))) => {
                            return End::Refused(reason);
                        }

                        Ok(ServerMessage::Response { id, result }) => {
                            let sender = pending.borrow_mut().remove(&id);

                            match sender {
                                Some(sender) => {
                                    let _ = sender.send(result);
                                }

                                None => {
                                    warn!(
                                        "Received Response for unknown request id: {id}"
                                    );
                                }
                            }
                        }

                        Ok(message) => state.apply(message),

                        Err(error) => {
                            error!("Could not decode ServerMessage: {error}");
                        }
                    }
                }

                Ok(Message::Text(_)) => {
                    warn!("Found text message, ignored");
                }

                Err(error) => return End::Lost(error.to_string()),
            }
        }
    };

    let write_task = async {
        while let Some(message) = rx.next().await {
            match message.encode() {
                Ok(bytes) => {
                    if let Err(error) =
                        write.send(Message::Bytes(bytes)).await
                    {
                        return End::Lost(error.to_string());
                    }
                }

                Err(error) => {
                    error!("Could not encode ClientMessage: {error}");
                }
            }
        }

        End::Lost("Request channel closed".to_string())
    };

    let end = match select(pin!(read_task), pin!(write_task)).await {
        Either::Left((end, _)) => end,
        Either::Right((end, _)) => end,
    };

    (end, connected.get())
}

async fn ws_url() -> Result<String, String> {
    let value = dioxus::document::eval(
        if cfg!(feature = "dev") {
            r#"
                return location.protocol === "https:"
                    ? "wss://" + location.hostname + ":8080"
                    : "ws://" + location.hostname + ":8080";
            "#
        } else {
            r#"
                return location.protocol === "https:"
                    ? "wss://" + location.host
                    : "ws://" + location.host;
            "#
        },
    )
    .await
    .map_err(|err| format!("Could not determine WebSocket URL: {err}"))?;

    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| {
            "Could not read WebSocket URL: unexpected eval result".to_string()
        })
}

fn fail_pending(pending: &Pending, reason: String) {
    for (_, sender) in pending.borrow_mut().drain() {
        let _ = sender.send(Err(reason.clone()));
    }
}
