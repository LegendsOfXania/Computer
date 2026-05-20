use super::session::consume_token;
use serde::{Deserialize, Serialize};
use std::{
    net::TcpStream,
    sync::{Mutex, OnceLock},
};
use tracing::{info, warn};
use tungstenite::{Message, WebSocket, accept};

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMsg {
    Auth { token: String },
    Publish { file: serde_json::Value },
    RequestSync,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg<'a> {
    Ready,
    AuthError { message: &'a str },
    Sync { payload: &'a serde_json::Value },
    Update { payload: &'a serde_json::Value },
    Error { message: &'a str },
}

static CLIENTS: OnceLock<Mutex<Vec<Client>>> = OnceLock::new();

fn clients() -> &'static Mutex<Vec<Client>> {
    CLIENTS.get_or_init(|| Mutex::new(Vec::new()))
}

struct Client {
    ws: WebSocket<TcpStream>,
    authenticated: bool,
}

impl Client {
    fn send_msg(&mut self, msg: &ServerMsg<'_>) -> bool {
        match serde_json::to_string(msg) {
            Ok(text) => match self.ws.send(Message::Text(text.into())) {
                Ok(_) => true,
                Err(e) => {
                    warn!(reason = %e, "Panel WS: send error, dropping client");
                    false
                }
            },
            Err(e) => {
                warn!(reason = %e, "Panel WS: serialization error");
                false
            }
        }
    }
}

pub fn accept_stream(stream: TcpStream) {
    match accept(stream) {
        Ok(ws) => {
            info!("Panel WS: new connection, waiting for auth");
            clients().lock().unwrap().push(Client {
                ws,
                authenticated: false,
            });
        }
        Err(e) => warn!(reason = %e, "Panel WS: handshake failed"),
    }
}

pub fn broadcast(msg: &ServerMsg<'_>) {
    let text = match serde_json::to_string(msg) {
        Ok(t) => t,
        Err(e) => {
            warn!(reason = %e, "Panel WS: broadcast serialization failed");
            return;
        }
    };
    let message = Message::Text(text.into());
    clients().lock().unwrap().retain_mut(|client| {
        if !client.authenticated {
            return true;
        }
        match client.ws.send(message.clone()) {
            Ok(_) => true,
            Err(e) => {
                warn!(reason = %e, "Panel WS: broadcast error, dropping client");
                false
            }
        }
    });
}

pub fn poll_clients() {
    clients().lock().unwrap().retain_mut(tick_client);
}

fn tick_client(client: &mut Client) -> bool {
    loop {
        match client.ws.read() {
            Ok(Message::Text(text)) => {
                if !dispatch(client, &text) {
                    return false;
                }
            }
            Ok(Message::Close(_)) => {
                info!("Panel WS: client disconnected");
                return false;
            }
            Ok(Message::Ping(data)) => {
                let _ = client.ws.send(Message::Pong(data));
            }
            Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                return true;
            }
            Err(e) => {
                warn!(reason = %e, "Panel WS: read error, dropping client");
                return false;
            }
            _ => {}
        }
    }
}

fn dispatch(client: &mut Client, text: &str) -> bool {
    let msg = match serde_json::from_str::<ClientMsg>(text) {
        Ok(m) => m,
        Err(_) => {
            return client.send_msg(&ServerMsg::Error {
                message: "invalid message format",
            });
        }
    };

    match msg {
        ClientMsg::Auth { token } => handle_auth(client, &token),

        _ if !client.authenticated => client.send_msg(&ServerMsg::AuthError {
            message: "not authenticated",
        }),

        ClientMsg::Publish { file } => {
            info!(?file, "Panel WS: publish request");
            // TODO: forward to computer-api publish handler
            client.send_msg(&ServerMsg::Ready) // ack
        }

        ClientMsg::RequestSync => {
            // TODO: build real payload from server state
            let payload = serde_json::json!({});
            client.send_msg(&ServerMsg::Sync { payload: &payload })
        }
    }
}

fn handle_auth(client: &mut Client, token: &str) -> bool {
    if consume_token(token) {
        client.authenticated = true;
        info!("Panel WS: client authenticated");
        client.send_msg(&ServerMsg::Ready)
    } else {
        client.send_msg(&ServerMsg::AuthError {
            message: "invalid or expired token",
        });
        false
    }
}
