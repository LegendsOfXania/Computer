use std::pin::pin;

use computer_model::id::Token;
use computer_model::protocol::{
    PROTOCOL_VERSION,
    message::{ClientMessage, ProtocolMessage, ServerMessage},
};
use dioxus::hooks::{UnboundedReceiver, use_context, use_coroutine};
use dioxus::logger::tracing::{error, warn};
use dioxus::prelude::use_hook;
use dioxus::signals::WritableExt;
use futures_util::{
    SinkExt, StreamExt,
    future::{Either, select},
};
use gloo_net::websocket::{Message, futures::WebSocket};
use gloo_timers::future::TimeoutFuture;

use crate::state::{ConnectionStatus, app::AppState};

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

pub fn ws_client() -> Client {
    let state = use_context::<AppState>();
    let pending = use_hook(Pending::new);

    let coroutine = use_coroutine(move |mut rx: UnboundedReceiver<ClientMessage>| async move {
        run(&mut rx, state, pending).await;
    });

    use_hook(move || Client::new(coroutine, state.status, pending))
}

async fn run(rx: &mut UnboundedReceiver<ClientMessage>, mut state: AppState, pending: Pending) {
    let mut attempt = 0u32;
    let mut refusals = 0u32;

    loop {
        let (end, connected) = session(rx, state, pending).await;

        pending.clear();

        while rx.try_recv().is_ok() {}

        if connected {
            attempt = 0;
        }

        match end {
            End::Refused(reason) => {
                refusals += 1;

                error!("Handshake refused ({refusals}/{MAX_REFUSALS}): {reason}");

                if refusals >= MAX_REFUSALS {
                    state.status.set(ConnectionStatus::Failed(reason));
                    return;
                }
            }

            End::Lost(reason) => {
                warn!("Connection lost: {reason}");

                refusals = 0;
            }
        }

        attempt += 1;

        state.status.set(ConnectionStatus::Reconnecting(attempt));

        TimeoutFuture::new(backoff_ms(attempt)).await;
    }
}

async fn session(
    rx: &mut UnboundedReceiver<ClientMessage>,
    mut state: AppState,
    pending: Pending,
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

    let token = match get_token().await {
        Ok(t) => t,
        Err(err) => return (End::Refused(err), false),
    };

    let hello = ClientMessage::Hello {
        version: PROTOCOL_VERSION,
        token,
    };

    let hello = match hello.encode() {
        Ok(bytes) => bytes,
        Err(error) => return (End::Lost(error), false),
    };

    if let Err(error) = write.send(Message::Bytes(hello)).await {
        return (End::Lost(error.to_string()), false);
    }

    let mut connected = false;

    let read_task = async {
        loop {
            let frame = if connected {
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
                Ok(Message::Bytes(bytes)) => match ServerMessage::decode(&bytes) {
                    Ok(ServerMessage::Handshake(Ok(_))) => {
                        connected = true;
                        state.status.set(ConnectionStatus::Connected);
                    }

                    Ok(ServerMessage::Handshake(Err(reason))) => {
                        return End::Refused(reason);
                    }

                    Ok(ServerMessage::Response { id, result }) => pending.resolve(id, result),

                    Ok(message) => state.apply(message),

                    Err(error) => {
                        error!("Could not decode ServerMessage: {error}");
                    }
                },

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
                    if let Err(error) = write.send(Message::Bytes(bytes)).await {
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

    (end, connected)
}

async fn ws_url() -> Result<String, String> {
    let value = dioxus::document::eval(if cfg!(feature = "dev") {
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
    })
    .await
    .map_err(|err| format!("Could not determine WebSocket URL: {err}"))?;

    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "Could not read WebSocket URL: unexpected eval result".to_string())
}

async fn get_token() -> Result<Token, String> {
    let value = dioxus::document::eval(
        r#"
            return new URLSearchParams(location.hash.slice(1)).get("t");
        "#,
    )
    .await
    .map_err(|err| format!("Could not read token: {err}"))?;

    value
        .as_str()
        .filter(|token| !token.is_empty())
        .map(|token| Token::from(token.to_owned()))
        .ok_or_else(|| "Token is missing in the page URL".to_string())
}
