use computer_model::protocol::PROTOCOL_VERSION;
use computer_model::protocol::message::{ClientMessage, ProtocolMessage, ServerMessage};
use dioxus::hooks::{use_coroutine, Coroutine, UnboundedReceiver};
use dioxus::signals::{Signal, WritableExt};
use futures_util::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use tracing::{error, warn};

use crate::state::AppState;
use crate::state::status::ConnectionStatus;

pub fn ws_client(
    url: String,
    mut state: AppState,
    mut status: Signal<ConnectionStatus>,
) -> Coroutine<ClientMessage> {
    use_coroutine(move |mut rx: UnboundedReceiver<ClientMessage>| {
        let url = url.clone();
        async move {
            let ws = match WebSocket::open(&url) {
                Ok(ws) => ws,
                Err(err) => {
                    error!("Could not connect from WebSocket: {err:?}");
                    status.set(ConnectionStatus::Failed(err.to_string()));
                    return;
                }
            };

            let (mut write, mut read) = ws.split();

            let hello = ClientMessage::Hello { version: PROTOCOL_VERSION };
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
                                error!("Handshake failed : {reason}");
                                status.set(ConnectionStatus::Failed(reason));
                                break;
                            }
                            Ok(msg) => state.apply(msg),
                            Err(err) => error!("Could not decode ServerMessage: {err:?}"),
                        },
                        Ok(Message::Text(_)) => warn!("Found text message, ignored"),
                        Err(err) => {
                            error!("WebSocket error: {err}");
                            status.set(ConnectionStatus::Failed(err.to_string()));
                            break;
                        }
                    }
                }
            };

            let write_task = async move {
                while let Some(client_message) = rx.next().await {
                    match client_message.encode() {
                        Ok(bytes) => {
                            if write.send(Message::Bytes(bytes)).await.is_err() {
                                break;
                            }
                        }
                        Err(err) => error!("Could not encode ClientMessage: {err:?}"),
                    }
                }
            };

            futures_util::future::join(read_task, write_task).await;
        }
    })
}