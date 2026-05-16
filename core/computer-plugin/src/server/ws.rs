use std::{net::TcpListener, thread};
use tracing::{info, warn};
use tungstenite::{Message, accept};

use crate::server::session::consume_token;

pub fn listen(addr: &str) {
    let listener = TcpListener::bind(addr).expect("Failed to bind WS server");
    info!("Computer WS server listening on ws://{}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle(stream));
            }
            Err(err) => {
                warn!(reason = %err, "Failed to accept WS connection");
            }
        }
    }
}

fn handle(stream: std::net::TcpStream) {
    let mut ws = match accept(stream) {
        Ok(ws) => ws,
        Err(err) => {
            warn!(reason = %err, "WS handshake failed");
            return;
        }
    };

    let session = loop {
        match ws.read() {
            Ok(Message::Text(text)) => match extract_token(&text) {
                Some(token) => match consume_token(&token) {
                    Some(session) => break session,
                    None => {
                        let _ = ws.send(Message::Text(
                            r#"{"type":"error","message":"invalid or expired token"}"#.into(),
                        ));
                        return;
                    }
                },
                None => {
                    let _ = ws.send(Message::Text(
                        r#"{"type":"error","message":"expected auth message"}"#.into(),
                    ));
                    return;
                }
            },
            Ok(Message::Close(_)) | Err(_) => return,
            _ => continue,
        }
    };

    info!(player = %session.id, "Panel WS authenticated");
    let _ = ws.send(Message::Text(r#"{"type":"ready"}"#.into()));

    // Boucle principale
    loop {
        match ws.read() {
            Ok(Message::Text(msg)) => {
                // TODO: dispatcher les messages
                info!(player = %session.id, msg = %msg, "Panel message received");
            }
            Ok(Message::Close(_)) => {
                info!(player = %session.id, "Panel WS disconnected");
                break;
            }
            Err(err) => {
                warn!(player = %session.id, reason = %err, "Panel WS error");
                break;
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
