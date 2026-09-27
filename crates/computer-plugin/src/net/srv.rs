use std::{
    collections::HashMap,
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use computer_model::protocol::{
    message::{ClientMessage, ProtocolMessage, ServerMessage},
    PROTOCOL_VERSION,
};
use pumpkin_plugin_api::{scheduler::SchedulerExt, Server};
use tungstenite::{
    handshake::derive_accept_key,
    protocol::Role,
    Error as WsError,
    Message,
    WebSocket,
};

use crate::net::{assets::{self, Asset}, http};

const REQUEST_BUFFER: usize = 8 * 1024;

type ConnectionState = Connection;

enum Connection {
    Pending {
        stream: TcpStream,
        buffer: Vec<u8>,
    },
    WebSocket(WebSocket<TcpStream>),
    Writing(Writing),
}

struct Writing {
    stream: TcpStream,
    header: Vec<u8>,
    header_sent: usize,
    body: Arc<[u8]>,
    body_sent: usize,
}

struct State {
    listener: TcpListener,
    connections: Vec<ConnectionState>,
    assets: HashMap<String, Asset>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

pub enum StartOutcome {
    Started,
    AlreadyRunning,
}

pub fn ensure_started(server: &Server, ip: &str, port: u16) -> Result<StartOutcome, String> {
    if STATE.lock().map_err(|e| e.to_string())?.is_some() {
        return Ok(StartOutcome::AlreadyRunning);
    }

    let listener = TcpListener::bind((ip, port)).map_err(|e| e.to_string())?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;

    let assets = assets::load().unwrap_or_default();

    server.schedule_repeating_task(0, 1, |_| poll());

    *STATE.lock().map_err(|e| e.to_string())? = Some(State {
        listener,
        connections: Vec::new(),
        assets,
    });

    Ok(StartOutcome::Started)
}

fn poll() {
    let Ok(mut guard) = STATE.lock() else {
        return;
    };

    let Some(state) = guard.as_mut() else {
        return;
    };

    accept_connections(state);

    let assets = &state.assets;

    state.connections = std::mem::take(&mut state.connections)
        .into_iter()
        .filter_map(|connection| connection.poll(assets))
        .collect();
}

fn accept_connections(state: &mut State) {
    loop {
        match state.listener.accept() {
            Ok((stream, _)) => {
                if stream.set_nonblocking(true).is_ok() {
                    state.connections.push(Connection::Pending {
                        stream,
                        buffer: Vec::with_capacity(REQUEST_BUFFER),
                    });
                }
            }

            Err(error) if error.kind() == ErrorKind::WouldBlock => break,

            Err(error) => {
                tracing::warn!("Failed to accept connection: {error}");
                break;
            }
        }
    }
}

impl Connection {
    fn poll(self, assets: &HashMap<String, Asset>) -> Option<Self> {
        match self {
            Self::Pending { stream, buffer } => poll_pending(stream, buffer, assets),

            Self::Writing(writing) => poll_writing(writing),

            Self::WebSocket(mut ws) => {
                poll_websocket(&mut ws).then_some(Self::WebSocket(ws))
            }
        }
    }
}

fn poll_pending(
    mut stream: TcpStream,
    mut buffer: Vec<u8>,
    assets: &HashMap<String, Asset>,
) -> Option<Connection> {
    let mut chunk = [0u8; 4096];

    loop {
        match stream.read(&mut chunk) {
            Ok(0) => return None,

            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);

                if buffer.len() >= REQUEST_BUFFER {
                    break;
                }

                if http::parse(&buffer).is_ok_and(|request| request.is_some()) {
                    break;
                }
            }

            Err(error) if error.kind() == ErrorKind::WouldBlock => break,

            Err(error) => {
                tracing::debug!("HTTP read failed: {error}");
                return None;
            }
        }
    }

    let request = match http::parse(&buffer) {
        Ok(Some(request)) => request,

        Ok(None) => {
            if buffer.len() >= REQUEST_BUFFER {
                tracing::warn!("Request headers exceeded {REQUEST_BUFFER} bytes");
                return None;
            }

            return Some(Connection::Pending { stream, buffer });
        }

        Err(()) => {
            tracing::warn!("Could not parse HTTP request");
            return None;
        }
    };
    
    if let Some(ws_key) = request.ws_key {
        return start_websocket(stream, buffer, ws_key);
    }

    Some(serve_file(stream, assets, &request.path))
}

fn start_websocket(
    mut stream: TcpStream,
    buffer: Vec<u8>,
    key: String,
) -> Option<Connection> {
    let header_end = find_header_end(&buffer)?;

    let accept_key = derive_accept_key(key.as_bytes());

    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {accept_key}\r\n\
         \r\n"
    );

    if let Err(error) = stream.write_all(response.as_bytes()) {
        if error.kind() != ErrorKind::WouldBlock {
            tracing::warn!("WebSocket handshake response failed: {error}");
            return None;
        }

        tracing::warn!("WebSocket handshake response would block");
        return None;
    }

    let remaining = buffer[header_end..].to_vec();

    let ws = WebSocket::from_partially_read(
        stream,
        remaining,
        Role::Server,
        None,
    );

    tracing::debug!("WebSocket handshake completed");

    Some(Connection::WebSocket(ws))
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
}

fn serve_file(
    stream: TcpStream,
    assets: &HashMap<String, Asset>,
    request_path: &str,
) -> Connection {
    let (status, reason, content_type, body) = match http::resolve(assets, request_path) {
        Some(asset) => (200, "OK", asset.content_type, asset.body.clone()),

        None => (
            404,
            "Not Found",
            "text/plain; charset=utf-8",
            Arc::from(&b"Not Found"[..]),
        ),
    };

    let header = format!(
        "HTTP/1.1 {status} {reason}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n",
        body.len()
    )
    .into_bytes();

    Connection::Writing(Writing {
        stream,
        header,
        header_sent: 0,
        body,
        body_sent: 0,
    })
}

fn poll_writing(mut writing: Writing) -> Option<Connection> {
    loop {
        if writing.header_sent < writing.header.len() {
            match writing.stream.write(&writing.header[writing.header_sent..]) {
                Ok(0) => return None,

                Ok(n) => {
                    writing.header_sent += n;
                }

                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    return Some(Connection::Writing(writing));
                }

                Err(_) => return None,
            }

            continue;
        }

        if writing.body_sent < writing.body.len() {
            match writing.stream.write(&writing.body[writing.body_sent..]) {
                Ok(0) => return None,

                Ok(n) => {
                    writing.body_sent += n;
                }

                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    return Some(Connection::Writing(writing));
                }

                Err(_) => return None,
            }

            continue;
        }

        return None;
    }
}

fn poll_websocket(ws: &mut WebSocket<TcpStream>) -> bool {
    loop {
        match ws.read() {
            Ok(Message::Binary(bytes)) => {
                if !on_data(ws, &bytes) {
                    return false;
                }
            }

            Ok(Message::Text(_)) => {
                tracing::warn!("Received text frame, ignored");
            }

            Ok(Message::Ping(_)) | Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => {}

            Ok(Message::Close(_)) => return false,

            Err(WsError::Io(error)) if error.kind() == ErrorKind::WouldBlock => {
                return true;
            }

            Err(error) => {
                tracing::warn!("WebSocket error: {error}");
                return false;
            }
        }
    }
}

fn on_data(ws: &mut WebSocket<TcpStream>, data: &[u8]) -> bool {
    let message = match ClientMessage::decode(data) {
        Ok(message) => message,

        Err(error) => {
            tracing::error!("Could not decode ClientMessage: {error:?}");
            return false;
        }
    };

    match message {
        ClientMessage::Hello { version } => {
            let result = if version == PROTOCOL_VERSION {
                Ok(PROTOCOL_VERSION)
            } else {
                Err(format!("Unsupported protocol version: {version}"))
            };

            let reply = ServerMessage::Handshake(result);

            match reply.encode() {
                Ok(bytes) => {
                    if ws.send(Message::Binary(bytes.into())).is_err() {
                        return false;
                    }
                }

                Err(error) => {
                    tracing::error!("Could not encode handshake: {error:?}");
                    return false;
                }
            }
        }

        ClientMessage::Request { .. } => {
            tracing::warn!(
                "Received request before request handling is implemented"
            );
        }
    }

    true
}