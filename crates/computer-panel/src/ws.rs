use std::pin::pin;

use computer_model::protocol::PROTOCOL_VERSION;
use computer_model::protocol::message::{ClientMessage, ProtocolMessage, ServerMessage};
use dioxus::hooks::{Coroutine, UnboundedReceiver, use_context, use_coroutine};
use dioxus::signals::{Signal, WritableExt};
use futures_util::{SinkExt, StreamExt, future::select};
use gloo_net::websocket::{Message, futures::WebSocket};
use tracing::{error, warn};

use crate::state::AppState;
use crate::state::status::ConnectionStatus;

pub fn ws_client() -> Coroutine<ClientMessage> {
    let mut state = use_context::<AppState>();
    let mut status = use_context::<Signal<ConnectionStatus>>();

    use_coroutine(move |mut rx: UnboundedReceiver<ClientMessage>| async move {
        let url = match ws_url().await {
            Ok(url) => url,
            Err(err) => return fail(&mut status, err),
        };

        let ws = match WebSocket::open(&url) {
            Ok(ws) => ws,
            Err(err) => return fail(&mut status, err.to_string()),
        };

        let (mut write, mut read) = ws.split();

        let hello = ClientMessage::Hello {
            version: PROTOCOL_VERSION,
        };

        if let Ok(bytes) = hello.encode() {
            let _ = write.send(Message::Bytes(bytes)).await;
        }

        let read_task = async move {
            while let Some(frame) = read.next().await {
                match frame {
                    Ok(Message::Bytes(bytes)) => match ServerMessage::decode(&bytes) {
                        Ok(ServerMessage::Handshake(Ok(_))) => {
                            status.set(ConnectionStatus::Connected);
                        }

                        Ok(ServerMessage::Handshake(Err(reason))) => {
                            return fail(&mut status, reason);
                        }

                        Ok(message) => state.apply(message),

                        Err(err) => error!("Could not decode ServerMessage: {err:?}"),
                    },

                    Ok(Message::Text(_)) => warn!("Found text message, ignored"),

                    Err(err) => return fail(&mut status, err.to_string()),
                }
            }

            fail(&mut status, "Connection closed".to_string());
        };

        let write_task = async move {
            while let Some(message) = rx.next().await {
                match message.encode() {
                    Ok(bytes) => {
                        if write.send(Message::Bytes(bytes)).await.is_err() {
                            return fail(&mut status, "Connection closed".to_string());
                        }
                    }

                    Err(err) => error!("Could not encode ClientMessage: {err:?}"),
                }
            }
        };

        select(pin!(read_task), pin!(write_task)).await;
    })
}

async fn ws_url() -> Result<String, String> {
    let value = dioxus::document::eval(
        r#"return (location.protocol === "https:" ? "wss://" : "ws://") + location.host;"#,
    )
    .await
    .map_err(|err| format!("Could not determine WebSocket URL: {err}"))?;

    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "Could not read WebSocket URL: unexpected eval result".to_string())
}

fn fail(status: &mut Signal<ConnectionStatus>, reason: String) {
    error!("{reason}");
    status.set(ConnectionStatus::Failed(reason));
}