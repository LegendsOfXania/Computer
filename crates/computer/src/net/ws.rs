use std::{
    io::{ErrorKind, Write},
    net::TcpStream,
};

use computer_model::protocol::{
    event::Event,
    message::{ClientMessage, ProtocolMessage, ServerMessage},
    request::Request,
    PROTOCOL_VERSION,
};

use tungstenite::{
    handshake::derive_accept_key,
    protocol::{Role, WebSocketConfig},
    Bytes,
    Error as WsError,
    Message,
    WebSocket,
};

use crate::{data, net::{
    connection::{Connection, Socket},
    limits::{
        IDLE_TIMEOUT,
        MAX_INVALID_FRAMES,
        MAX_MESSAGE_SIZE,
        MAX_MESSAGES_PER_TICK,
        MAX_WRITE_BUFFER,
        PENDING_TIMEOUT,
        PING_INTERVAL,
    },
    request,
    srv::State,
}};

pub fn upgrade(
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

    match stream.write_all(response.as_bytes()) {
        Ok(()) => {}

        Err(error) if error.kind() == ErrorKind::WouldBlock => {
            tracing::debug!(
                "WebSocket handshake response would block"
            );

            return None;
        }

        Err(error) => {
            tracing::warn!(
                "WebSocket handshake response failed: {error}"
            );

            return None;
        }
    }

    let remaining = buffer[header_end..].to_vec();

    let peer = stream
        .peer_addr()
        .map(|address| address.to_string())
        .unwrap_or_else(|_| "unknown address".to_string());

    let config = WebSocketConfig::default()
        .max_message_size(Some(MAX_MESSAGE_SIZE))
        .max_frame_size(Some(MAX_MESSAGE_SIZE))
        .max_write_buffer_size(MAX_WRITE_BUFFER);

    let ws = WebSocket::from_partially_read(
        stream,
        remaining,
        Role::Server,
        Some(config),
    );

    tracing::debug!("WebSocket handshake completed");

    Some(Connection::WebSocket(Socket::new(ws, peer)))
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
}

enum Outcome {
    Continue,
    Event(Event),
    Close,
}

pub fn poll(socket: &mut Socket) -> (bool, Vec<Event>) {
    let mut events = Vec::new();

    socket.age += 1;
    socket.idle += 1;
    socket.since_ping += 1;

    for _ in 0..MAX_MESSAGES_PER_TICK {
        match socket.ws.read() {
            Ok(message) => {
                socket.idle = 0;

                match message {
                    Message::Binary(bytes) => {
                        match on_data(socket, &bytes) {
                            Outcome::Continue => {}

                            Outcome::Event(event) => {
                                events.push(event);
                            }

                            Outcome::Close => {
                                return (false, events);
                            }
                        }
                    }

                    Message::Text(_) => {
                        if note_invalid(socket, "text frame") {
                            return (false, events);
                        }
                    }

                    Message::Close(_) => {
                        return (false, events);
                    }

                    Message::Ping(_)
                    | Message::Pong(_)
                    | Message::Frame(_) => {}
                }
            }

            Err(error) if is_would_block(&error) => {
                break;
            }

            Err(WsError::ConnectionClosed | WsError::AlreadyClosed) => {
                return (false, events);
            }

            Err(error) => {
                tracing::warn!(
                    "WebSocket error: {error}"
                );

                return (false, events);
            }
        }
    }

    (keep_alive(socket), events)
}

fn keep_alive(socket: &mut Socket) -> bool {
    if !socket.ready && socket.age >= PENDING_TIMEOUT {
        tracing::debug!("WebSocket client never said hello");
        return false;
    }

    if socket.idle >= IDLE_TIMEOUT {
        tracing::info!("WebSocket client timed out");
        return false;
    }

    if socket.since_ping >= PING_INTERVAL {
        socket.since_ping = 0;

        match socket.ws.send(Message::Ping(Vec::<u8>::new().into())) {
            Ok(()) => {}

            Err(error) if is_would_block(&error) => {}

            Err(error) => {
                tracing::debug!(
                    "Could not ping WebSocket client: {error}"
                );

                return false;
            }
        }
    }

    true
}

fn note_invalid(socket: &mut Socket, what: &str) -> bool {
    socket.invalid += 1;

    tracing::warn!(
        "Panel client #{} ({}): ignored an invalid frame ({what}), {}/{MAX_INVALID_FRAMES}",
        socket.id,
        socket.peer,
        socket.invalid,
    );

    socket.invalid >= MAX_INVALID_FRAMES
}

fn on_data(socket: &mut Socket, data: &[u8]) -> Outcome {
    let message = match ClientMessage::decode(data) {
        Ok(message) => message,

        Err(error) => {
            return if note_invalid(socket, &format!("undecodable: {error}")) {
                Outcome::Close
            } else {
                Outcome::Continue
            };
        }
    };

    socket.invalid = 0;

    match message {
        ClientMessage::Hello { version } => {
            on_hello(socket, version)
        }

        ClientMessage::Request { id, request } => {
            on_request(socket, id, request)
        }
    }
}

fn on_hello(socket: &mut Socket, version: u32) -> Outcome {
    if version != PROTOCOL_VERSION {
        tracing::warn!(
            "Panel client #{} ({}) refused: it speaks protocol v{version}, the engine v{PROTOCOL_VERSION}",
            socket.id,
            socket.peer,
        );

        let _ = send_message(
            &mut socket.ws,
            &ServerMessage::Handshake(Err(format!(
                "Unsupported protocol version: {version}"
            ))),
        );

        return Outcome::Close;
    }

    let library = match data::library::init_dev() {
        Ok(library) => library,

        Err(error) => {
            tracing::error!(
                "Could not load dev library: {error}"
            );

            let _ = send_message(
                &mut socket.ws,
                &ServerMessage::Handshake(Err(format!(
                    "Could not load the library: {error}"
                ))),
            );

            return Outcome::Close;
        }
    };

    let sent = send_message(
        &mut socket.ws,
        &ServerMessage::Handshake(Ok(PROTOCOL_VERSION)),
    )
    .and_then(|()| {
        send_message(
            &mut socket.ws,
            &ServerMessage::Library(library),
        )
    })
    .and_then(|()| {
        send_message(
            &mut socket.ws,
            &ServerMessage::Registry(data::registry::get()),
        )
    });

    if sent.is_err() {
        return Outcome::Close;
    }

    socket.ready = true;

    tracing::info!(
        "Panel client #{} connected from {} (protocol v{version})",
        socket.id,
        socket.peer,
    );

    Outcome::Continue
}

fn on_request(
    socket: &mut Socket,
    id: u64,
    request: Request,
) -> Outcome {
    if !socket.ready {
        tracing::warn!(
            "Request received before the handshake, rejected"
        );

        socket.replies.push(ServerMessage::Response {
            id,
            result: Err("Handshake required".to_string()),
        });

        return Outcome::Continue;
    }

    let is_push = matches!(request, Request::Push);

    match request::handle(request) {
        Ok(event) => {
            if is_push {
                tracing::info!(
                    "Panel client #{} ({}) pushed the dev library to live",
                    socket.id,
                    socket.peer,
                );
            }

            socket.replies.push(ServerMessage::Response {
                id,
                result: Ok(()),
            });

            Outcome::Event(event)
        }

        Err(error) => {
            socket.replies.push(ServerMessage::Response {
                id,
                result: Err(error),
            });

            Outcome::Continue
        }
    }
}

fn is_would_block(error: &WsError) -> bool {
    matches!(
        error,
        WsError::Io(error) if error.kind() == ErrorKind::WouldBlock
    )
}

fn send_bytes(
    ws: &mut WebSocket<TcpStream>,
    bytes: Bytes,
) -> Result<(), ()> {
    match ws.send(Message::Binary(bytes)) {
        Ok(()) => Ok(()),

        Err(error) if is_would_block(&error) => Ok(()),

        Err(WsError::WriteBufferFull(_)) => {
            tracing::warn!(
                "Dropped a WebSocket client that is too slow"
            );

            Err(())
        }

        Err(error) => {
            tracing::debug!(
                "Could not send message: {error}"
            );

            Err(())
        }
    }
}

fn send_message(
    ws: &mut WebSocket<TcpStream>,
    message: &ServerMessage,
) -> Result<(), ()> {
    let bytes = message.encode().map_err(|error| {
        tracing::error!(
            "Could not encode ServerMessage: {error}"
        );
    })?;

    send_bytes(ws, Bytes::from(bytes))
}

pub fn broadcast_event(
    state: &mut State,
    event: Event,
) {
    let bytes = match (ServerMessage::Event { event }).encode() {
        Ok(bytes) => Bytes::from(bytes),

        Err(error) => {
            tracing::error!(
                "Could not encode event: {error}"
            );
            return;
        }
    };

    state.connections.retain_mut(|connection| {
        let Connection::WebSocket(socket) = connection else {
            return true;
        };

        if !socket.ready {
            return true;
        }

        send_bytes(&mut socket.ws, bytes.clone()).is_ok()
    });
}

pub fn send_replies(state: &mut State) {
    state.connections.retain_mut(|connection| {
        let Connection::WebSocket(socket) = connection else {
            return true;
        };

        for reply in std::mem::take(&mut socket.replies) {
            if send_message(&mut socket.ws, &reply).is_err() {
                return false;
            }
        }

        match socket.ws.flush() {
            Ok(()) => true,

            Err(error) if is_would_block(&error) => true,

            Err(error) => {
                tracing::debug!(
                    "Could not flush WebSocket: {error}"
                );

                false
            }
        }
    });
}