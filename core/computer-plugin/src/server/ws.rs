use super::session::consume_token;
use serde::{Deserialize, Serialize};
use std::{
    net::TcpStream,
    sync::{Mutex, OnceLock},
};
use tracing::{info, warn};
use tungstenite::handshake::{
    MidHandshake,
    server::{NoCallback, ServerHandshake},
};
use tungstenite::{HandshakeError, Message, WebSocket, accept};

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

struct Client {
    ws: WebSocket<TcpStream>,
    authenticated: bool,
}

impl Client {
    fn send(&mut self, msg: &ServerMsg<'_>) -> bool {
        let text = match serde_json::to_string(msg) {
            Ok(t) => t,
            Err(e) => {
                warn!(reason = %e, "WS: serialization error");
                return false;
            }
        };
        self.ws
            .send(Message::Text(text.into()))
            .map_err(|e| warn!(reason = %e, "WS: send error, dropping client"))
            .is_ok()
    }
}

static CLIENTS: OnceLock<Mutex<Vec<Client>>> = OnceLock::new();
static PENDING: OnceLock<Mutex<Vec<MidHandshake<ServerHandshake<TcpStream, NoCallback>>>>> =
    OnceLock::new();

fn clients() -> &'static Mutex<Vec<Client>> {
    CLIENTS.get_or_init(|| Mutex::new(Vec::new()))
}

fn pending() -> &'static Mutex<Vec<MidHandshake<ServerHandshake<TcpStream, NoCallback>>>> {
    PENDING.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn accept_stream(stream: TcpStream) {
    match accept(stream) {
        Ok(ws) => {
            info!("WS: new connection, waiting for auth");
            clients().lock().unwrap().push(Client {
                ws,
                authenticated: false,
            });
        }
        Err(HandshakeError::Interrupted(mid)) => {
            pending().lock().unwrap().push(mid);
        }
        Err(HandshakeError::Failure(e)) => warn!(reason = %e, "WS: handshake failed"),
    }
}

pub fn broadcast(msg: &ServerMsg<'_>) {
    let text = match serde_json::to_string(msg) {
        Ok(t) => t,
        Err(e) => {
            warn!(reason = %e, "WS: broadcast serialization failed");
            return;
        }
    };
    let message = Message::Text(text.into());
    clients().lock().unwrap().retain_mut(|c| {
        c.authenticated
            && c.ws
                .send(message.clone())
                .map_err(|e| warn!(reason = %e, "WS: broadcast error, dropping client"))
                .is_ok()
    });
}

pub fn poll_clients() {
    poll_pending();
    clients().lock().unwrap().retain_mut(tick_client);
}

fn poll_pending() {
    let mut queue = pending().lock().unwrap();
    if queue.is_empty() {
        return;
    }
    let batch = std::mem::take(&mut *queue);
    for mid in batch {
        match mid.handshake() {
            Ok(ws) => {
                info!("WS: new connection, waiting for auth");
                clients().lock().unwrap().push(Client {
                    ws,
                    authenticated: false,
                });
            }
            Err(HandshakeError::Interrupted(mid)) => queue.push(mid),
            Err(HandshakeError::Failure(e)) => warn!(reason = %e, "WS: handshake failed"),
        }
    }
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
                info!("WS: client disconnected");
                return false;
            }
            Ok(Message::Ping(data)) => {
                let _ = client.ws.send(Message::Pong(data));
            }
            Ok(_) => {}
            Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                return true;
            }
            Err(e) => {
                warn!(reason = %e, "WS: read error, dropping client");
                return false;
            }
        }
    }
}

fn dispatch(client: &mut Client, text: &str) -> bool {
    let msg = match serde_json::from_str::<ClientMsg>(text) {
        Ok(m) => m,
        Err(_) => {
            return client.send(&ServerMsg::Error {
                message: "invalid message format",
            });
        }
    };

    match msg {
        ClientMsg::Auth { token } => {
            if consume_token(&token) {
                client.authenticated = true;
                info!("WS: client authenticated");
                client.send(&ServerMsg::Ready)
            } else {
                client.send(&ServerMsg::AuthError {
                    message: "invalid or expired token",
                });
                false
            }
        }

        _ if !client.authenticated => client.send(&ServerMsg::AuthError {
            message: "not authenticated",
        }),

        ClientMsg::Publish { file } => {
            info!(?file, "WS: publish request");
            // TODO: forward to computer-api publish handler
            client.send(&ServerMsg::Ready)
        }

        ClientMsg::RequestSync => {
            // TODO: build payload from server state
            let payload = serde_json::json!({});
            client.send(&ServerMsg::Sync { payload: &payload })
        }
    }
}
