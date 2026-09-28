use std::{
    collections::HashMap,
    io::{ErrorKind, Read, Write},
    net::TcpStream,
    sync::Arc,
};

use computer_model::protocol::event::Event;
use tungstenite::WebSocket;

use crate::net::{
    assets::Asset,
    http,
    srv::REQUEST_BUFFER,
    ws,
};

pub enum Connection {
    Pending {
        stream: TcpStream,
        buffer: Vec<u8>,
    },

    WebSocket(WebSocket<TcpStream>),

    Writing(Writing),
}

pub struct Writing {
    stream: TcpStream,
    header: Vec<u8>,
    header_sent: usize,
    body: Arc<[u8]>,
    body_sent: usize,
}

impl Connection {
    pub fn pending(
        stream: TcpStream,
        buffer_size: usize,
    ) -> Self {
        Self::Pending {
            stream,
            buffer: Vec::with_capacity(buffer_size),
        }
    }

    pub fn poll(
        self,
        assets: &HashMap<String, Asset>,
    ) -> (Option<Self>, Vec<Event>) {
        match self {
            Self::Pending { stream, buffer } => {
                (
                    poll_pending(stream, buffer, assets),
                    Vec::new(),
                )
            }

            Self::Writing(writing) => {
                (
                    poll_writing(writing),
                    Vec::new(),
                )
            }

            Self::WebSocket(mut ws) => {
                let (alive, events) =
                    ws::poll(&mut ws);

                if alive {
                    (Some(Self::WebSocket(ws)), events)
                } else {
                    (None, events)
                }
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

                if http::parse(&buffer)
                    .is_ok_and(|request| request.is_some())
                {
                    break;
                }
            }

            Err(error)
                if error.kind() == ErrorKind::WouldBlock =>
            {
                break;
            }

            Err(error) => {
                tracing::debug!(
                    "HTTP read failed: {error}"
                );
                return None;
            }
        }
    }

    let request = match http::parse(&buffer) {
        Ok(Some(request)) => request,

        Ok(None) => {
            if buffer.len() >= REQUEST_BUFFER {
                tracing::warn!(
                    "Request headers exceeded {REQUEST_BUFFER} bytes"
                );

                return None;
            }

            return Some(Connection::Pending {
                stream,
                buffer,
            });
        }

        Err(()) => {
            tracing::warn!("Could not parse HTTP request");
            return None;
        }
    };

    if let Some(key) = request.ws_key {
        return ws::upgrade(stream, buffer, key);
    }

    Some(serve_file(
        stream,
        assets,
        &request.path,
    ))
}

fn serve_file(
    stream: TcpStream,
    assets: &HashMap<String, Asset>,
    request_path: &str,
) -> Connection {
    let (status, reason, content_type, body) =
        match http::resolve(assets, request_path) {
            Some(asset) => (
                200,
                "OK",
                asset.content_type,
                asset.body.clone(),
            ),

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
            match writing
                .stream
                .write(&writing.header[writing.header_sent..])
            {
                Ok(0) => return None,

                Ok(n) => {
                    writing.header_sent += n;
                }

                Err(error)
                    if error.kind() == ErrorKind::WouldBlock =>
                {
                    return Some(Connection::Writing(writing));
                }

                Err(error) => {
                    tracing::debug!(
                        "HTTP write failed: {error}"
                    );
                    return None;
                }
            }

            continue;
        }

        if writing.body_sent < writing.body.len() {
            match writing
                .stream
                .write(&writing.body[writing.body_sent..])
            {
                Ok(0) => return None,

                Ok(n) => {
                    writing.body_sent += n;
                }

                Err(error)
                    if error.kind() == ErrorKind::WouldBlock =>
                {
                    return Some(Connection::Writing(writing));
                }

                Err(error) => {
                    tracing::debug!(
                        "HTTP body write failed: {error}"
                    );
                    return None;
                }
            }

            continue;
        }

        return None;
    }
}