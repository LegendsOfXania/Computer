use crate::server::session::consume_token;
use std::{net::TcpListener, thread};
use tracing::{error, info, warn};
use tungstenite::{Message, accept};

pub fn listen(addr: &str) {
    let listener = match TcpListener::bind(addr) {
        Ok(listener) => listener,
        Err(err) => {
            error!(reason = %err, "Failed to bind WS server on {}", addr);
            return;
        }
    };
    info!("Computer WS server listening on ws://{}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle(stream));
            }
            Err(err) => {
                error!(reason = %err, "Failed to accept WS connection");
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

    let authenticated = loop {
        match ws.read() {
            Ok(Message::Text(text)) => match extract_token(&text) {
                Some(token) => {
                    if consume_token(&token) {
                        break true;
                    } else {
                        let _ = ws.send(Message::Text(
                            r#"{"type":"error","message":"invalid or expired token"}"#.into(),
                        ));
                        return;
                    }
                }
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

    if !authenticated {
        return;
    }

    info!("Panel WS authenticated");
    let _ = ws.send(Message::Text(r#"{"type":"ready"}"#.into()));

    loop {
        match ws.read() {
            Ok(Message::Text(msg)) => {
                info!(msg = %msg, "Panel message received");
            }
            Ok(Message::Close(_)) => {
                info!("Panel WS disconnected");
                break;
            }
            Err(err) => {
                warn!(reason = %err, "Panel WS error");
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
