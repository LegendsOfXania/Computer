use std::{
    cell::RefCell,
    collections::HashMap,
    pin::pin,
    rc::Rc,
};

use computer_model::protocol::{
    message::{ClientMessage, ProtocolMessage, ServerMessage},
    PROTOCOL_VERSION,
};
use dioxus::hooks::{UnboundedReceiver, use_context, use_coroutine};
use dioxus::signals::{Signal, WritableExt};
use futures_channel::oneshot;
use futures_util::{future::select, SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use tracing::{error, warn};

use crate::state::{
    app::AppState,
    ConnectionStatus,
};

use super::client::Client;

type Pending = Rc<
    RefCell<HashMap<u64, oneshot::Sender<Result<(), String>>>>,
>;

pub fn ws_client() -> Client {
    let mut state = use_context::<AppState>();
    let mut status = use_context::<Signal<ConnectionStatus>>();

    let pending: Pending = Rc::new(RefCell::new(HashMap::new()));
    let pending_task = Rc::clone(&pending);

    let coroutine = use_coroutine(
        move |mut rx: UnboundedReceiver<ClientMessage>| {
            let pending = Rc::clone(&pending_task);

            async move {
                let url = match ws_url().await {
                    Ok(url) => url,
                    Err(err) => {
                        fail(&mut status, err.clone());
                        fail_pending(&pending, err);
                        return;
                    }
                };

                let ws = match WebSocket::open(&url) {
                    Ok(ws) => ws,
                    Err(err) => {
                        let reason = err.to_string();
                        fail(&mut status, reason.clone());
                        fail_pending(&pending, reason);
                        return;
                    }
                };

                let (mut write, mut read) = ws.split();

                let hello = ClientMessage::Hello {
                    version: PROTOCOL_VERSION,
                };

                if let Ok(bytes) = hello.encode() {
                    let _ = write.send(Message::Bytes(bytes)).await;
                }

                let pending_read = Rc::clone(&pending);
                let pending_write = Rc::clone(&pending);

                let read_task = async move {
                    while let Some(frame) = read.next().await {
                        match frame {
                            Ok(Message::Bytes(bytes)) => {
                                match ServerMessage::decode(&bytes) {
                                    Ok(ServerMessage::Handshake(Ok(_))) => {
                                        status.set(
                                            ConnectionStatus::Connected,
                                        );
                                    }

                                    Ok(ServerMessage::Handshake(Err(reason))) => {
                                        fail(&mut status, reason);
                                        return;
                                    }

                                    Ok(ServerMessage::Response {
                                        id,
                                        result,
                                    }) => {
                                        if let Some(sender) =
                                            pending_read.borrow_mut().remove(&id)
                                        {
                                            let _ = sender.send(result);
                                        } else {
                                            warn!(
                                                "Received Response for unknown request id: {id}"
                                            );
                                        }
                                    }

                                    Ok(message) => state.apply(message),

                                    Err(err) => {
                                        error!(
                                            "Could not decode ServerMessage: {err:?}"
                                        );
                                    }
                                }
                            }

                            Ok(Message::Text(_)) => {
                                warn!("Found text message, ignored");
                            }

                            Err(err) => {
                                fail(&mut status, err.to_string());
                                return;
                            }
                        }
                    }

                    fail(&mut status, "Connection closed".to_string());
                };

                let write_task = async move {
                    while let Some(message) = rx.next().await {
                        match message.encode() {
                            Ok(bytes) => {
                                if write.send(Message::Bytes(bytes)).await.is_err() {
                                    fail(
                                        &mut status,
                                        "Connection closed".to_string(),
                                    );
                                    return;
                                }
                            }

                            Err(err) => {
                                error!(
                                    "Could not encode ClientMessage: {err:?}"
                                );
                            }
                        }
                    }
                };

                select(pin!(read_task), pin!(write_task)).await;

                fail_pending(
                    &pending_write,
                    "Connection closed".to_string(),
                );
            }
        },
    );

    Client::new(coroutine, pending)
}

async fn ws_url() -> Result<String, String> {
    let value = dioxus::document::eval(
        if cfg!(feature = "dev") {
            r#"
            return (location.protocol === "https:" ? "wss://" : "ws://")
                + location.hostname
                + ":8080";
            "#
        } else {
            r#"
            return (location.protocol === "https:" ? "wss://" : "ws://")
                + location.host;
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

fn fail(status: &mut Signal<ConnectionStatus>, reason: String) {
    error!("{reason}");
    status.set(ConnectionStatus::Failed(reason));
}

fn fail_pending(pending: &Pending, reason: String) {
    for (_, sender) in pending.borrow_mut().drain() {
        let _ = sender.send(Err(reason.clone()));
    }
}