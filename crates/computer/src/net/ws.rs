use std::{
    io::{ErrorKind, Write},
    net::TcpStream,
};

use computer_model::protocol::{
    event::Event,
    message::{ClientMessage, ProtocolMessage, ServerMessage},
    PROTOCOL_VERSION,
};

use tungstenite::{
    handshake::derive_accept_key,
    protocol::Role,
    Error as WsError,
    Message,
    WebSocket,
};

use crate::{data, net::{
    connection::Connection,
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

pub fn poll(
    ws: &mut WebSocket<TcpStream>,
) -> (bool, Vec<Event>) {
    let mut events = Vec::new();

    loop {
        match ws.read() {
            Ok(Message::Binary(bytes)) => {
                match on_data(ws, &bytes) {
                    Ok(Some(event)) => {
                        events.push(event);
                    }

                    Ok(None) => {}

                    Err(()) => {
                        return (false, events);
                    }
                }
            }

            Ok(Message::Text(_)) => {
                tracing::warn!(
                    "Received text frame, ignored"
                );
            }

            Ok(Message::Ping(_))
            | Ok(Message::Pong(_))
            | Ok(Message::Frame(_)) => {}

            Ok(Message::Close(_)) => {
                return (false, events);
            }

            Err(WsError::Io(error))
                if error.kind() == ErrorKind::WouldBlock =>
            {
                return (true, events);
            }

            Err(error) => {
                tracing::warn!(
                    "WebSocket error: {error}"
                );

                return (false, events);
            }
        }
    }
}

fn on_data(
    ws: &mut WebSocket<TcpStream>,
    data: &[u8],
) -> Result<Option<Event>, ()> {
    let message = ClientMessage::decode(data)
        .map_err(|error| {
            tracing::error!(
                "Could not decode ClientMessage: {error}"
            );
        })?;

    match message {
        ClientMessage::Hello { version } => {
            if version != PROTOCOL_VERSION {
                send_message(
                    ws,
                    ServerMessage::Handshake(Err(format!(
                        "Unsupported protocol version: {version}"
                    ))),
                )?;

                return Err(());
            }

            send_message(
                ws,
                ServerMessage::Handshake(
                    Ok(PROTOCOL_VERSION),
                ),
            )?;

            let library = data::library::init_dev()
                    .map_err(|err| {
                        tracing::error!(
                            "Could not load dev library: {err}"
                        );
                    })?;

            send_message(
                ws,
                ServerMessage::Library(library),
            )?;

            let registry = data::registry::get();

            send_message(
                ws,
                ServerMessage::Registry(registry),
            )?;


            Ok(None)
        }

        ClientMessage::Request { id, request } => {
            request::handle(id, request)
        }
    }
}

fn send_message(
    ws: &mut WebSocket<TcpStream>,
    message: ServerMessage,
) -> Result<(), ()> {
    let bytes = message.encode().map_err(|error| {
        tracing::error!(
            "Could not encode ServerMessage: {error}"
        );
    })?;

    ws.send(Message::Binary(bytes.into()))
        .map_err(|error| {
            tracing::warn!(
                "Could not send ServerMessage: {error}"
            );
        })?;

    Ok(())
}

pub fn broadcast_event(
    state: &mut State,
    event: Event,
) {
    let message =
        match (ServerMessage::Event { event }).encode() {
            Ok(bytes) => bytes,

            Err(error) => {
                tracing::error!(
                    "Could not encode event: {error}"
                );
                return;
            }
        };

    state.connections.retain_mut(|connection| {
        let Connection::WebSocket(ws) = connection else {
            return true;
        };

        match ws.send(Message::Binary(message.clone().into())) {
            Ok(()) => true,

            Err(error) => {
                tracing::debug!(
                    "Could not send event: {error}"
                );
                false
            }
        }
    });
}