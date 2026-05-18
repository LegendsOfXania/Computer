use crate::server::session::consume_token;
use std::{
    net::TcpStream,
    sync::{Mutex, OnceLock},
};
use tracing::{info, warn};
use tungstenite::{Message, WebSocket, accept};

static CLIENTS: OnceLock<Mutex<Vec<Client>>> = OnceLock::new();

fn clients() -> &'static Mutex<Vec<Client>> {
    CLIENTS.get_or_init(|| Mutex::new(Vec::new()))
}

struct Client {
    ws: WebSocket<TcpStream>,
    authenticated: bool,
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
        Err(err) => {
            warn!(reason = %err, "Panel WS: handshake failed");
        }
    }
}

pub fn poll_clients() {
    let mut guard = clients().lock().unwrap();

    let mut i = guard.len();
    while i > 0 {
        i -= 1;
        if !tick_client(&mut guard[i]) {
            guard.swap_remove(i);
        }
    }
}

#[allow(dead_code)]
pub fn broadcast(msg: &str) {
    let mut guard = clients().lock().unwrap();
    let message = Message::Text(msg.into());

    guard.retain_mut(|client| {
        if !client.authenticated {
            return true;
        }
        match client.ws.send(message.clone()) {
            Ok(_) => true,
            Err(err) => {
                warn!(reason = %err, "Panel WS: broadcast send error, dropping client");
                false
            }
        }
    });
}

fn tick_client(client: &mut Client) -> bool {
    loop {
        match client.ws.read() {
            Ok(Message::Text(text)) if !client.authenticated => match extract_token(&text) {
                Some(token) if consume_token(&token) => {
                    client.authenticated = true;
                    info!("Panel WS: client authenticated");
                    let _ = client.ws.send(Message::Text(r#"{"type":"ready"}"#.into()));
                }
                Some(_) => {
                    let _ = client.ws.send(Message::Text(
                        r#"{"type":"error","message":"invalid or expired token"}"#.into(),
                    ));
                    return false;
                }
                None => {
                    let _ = client.ws.send(Message::Text(
                        r#"{"type":"error","message":"expected auth message"}"#.into(),
                    ));
                    return false;
                }
            },

            Ok(Message::Text(msg)) => {
                info!(msg = %msg, "Panel WS: message received");
                // TODO: dispatch to command handlers
            }

            Ok(Message::Close(_)) => {
                info!("Panel WS: client disconnected");
                return false;
            }

            Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                return true;
            }

            Err(err) => {
                warn!(reason = %err, "Panel WS: read error, dropping client");
                return false;
            }

            _ => {}
        }
    }
}

fn extract_token(json: &str) -> Option<String> {
    let key = r#""token":""#;
    let start = json.find(key)? + key.len();
    let end = json[start..].find('"')? + start;
    Some(json[start..end].to_string())
}
