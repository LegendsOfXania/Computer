use std::{
    collections::HashMap,
    io::{ErrorKind, Read, Write},
    net::TcpStream,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use computer_model::protocol::{event::Event, message::ServerMessage};
use tungstenite::WebSocket;

use crate::net::{
    assets::Asset,
    http,
    limits::{PENDING_TIMEOUT, TICKS_PER_SECOND, WRITE_STALL_TIMEOUT},
    srv::REQUEST_BUFFER,
    ws,
};

pub enum Connection {
    Pending {
        stream: TcpStream,
        buffer: Vec<u8>,
        

        age: u32,
    },

    WebSocket(Socket),

    Writing(Writing),
}



static NEXT_SOCKET_ID: AtomicU64 = AtomicU64::new(1);

pub struct Socket {
    pub ws: WebSocket<TcpStream>,
    pub id: u64,
    pub peer: String,
    pub ready: bool,
    pub replies: Vec<ServerMessage>,
    pub age: u32,    
    pub idle: u32,
    pub since_ping: u32,
    pub invalid: u32,
}

impl Socket {
    pub fn new(ws: WebSocket<TcpStream>, peer: String) -> Self {
        Self {
            ws,
            id: NEXT_SOCKET_ID.fetch_add(1, Ordering::Relaxed),
            peer,
            ready: false,
            replies: Vec::new(),
            age: 0,
            idle: 0,
            since_ping: 0,
            invalid: 0,
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        if self.ready {
            tracing::info!(
                "Panel client #{} ({}) disconnected after {}s",
                self.id,
                self.peer,
                self.age / TICKS_PER_SECOND,
            );
        }
    }
}

pub struct Writing {
    stream: TcpStream,
    header: Vec<u8>,
    header_sent: usize,
    body: Arc<[u8]>,
    body_sent: usize,
    stalled: u32,
}

impl Connection {
    pub fn pending(
        stream: TcpStream,
        buffer_size: usize,
    ) -> Self {
        Self::Pending {
            stream,
            buffer: Vec::with_capacity(buffer_size),
            age: 0,
        }
    }

    pub fn poll(
        self,
        assets: &HashMap<String, Asset>,
    ) -> (Option<Self>, Vec<Event>) {
        match self {
            Self::Pending { stream, buffer, age } => {
                if age >= PENDING_TIMEOUT {
                    tracing::debug!(
                        "Dropped a connection that never sent a complete request"
                    );

                    return (None, Vec::new());
                }

                (
                    poll_pending(stream, buffer, age + 1, assets),
                    Vec::new(),
                )
            }

            Self::Writing(writing) => {
                (
                    poll_writing(writing),
                    Vec::new(),
                )
            }

            Self::WebSocket(mut socket) => {
                let (alive, events) =
                    ws::poll(&mut socket);

                if alive {
                    (Some(Self::WebSocket(socket)), events)
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
    age: u32,
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

            Err(error)
                if error.kind() == ErrorKind::Interrupted =>
            {
                continue;
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
                age,
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
        stalled: 0,
    })
}

fn poll_writing(mut writing: Writing) -> Option<Connection> {
    let mut progressed = false;

    loop {
        let (data, sent) = if writing.header_sent < writing.header.len() {
            (&writing.header[writing.header_sent..], &mut writing.header_sent)
        } else if writing.body_sent < writing.body.len() {
            (&writing.body[writing.body_sent..], &mut writing.body_sent)
        } else {
            return None;
        };

        match writing.stream.write(data) {
            Ok(0) => return None,

            Ok(n) => {
                *sent += n;
                progressed = true;
            }

            Err(error)
                if error.kind() == ErrorKind::WouldBlock =>
            {
                if progressed {
                    writing.stalled = 0;
                } else {
                    writing.stalled += 1;
                }

                if writing.stalled >= WRITE_STALL_TIMEOUT {
                    tracing::debug!(
                        "Dropped a client that stopped reading"
                    );

                    return None;
                }

                return Some(Connection::Writing(writing));
            }

            Err(error)
                if error.kind() == ErrorKind::Interrupted => {}

            Err(error) => {
                tracing::debug!(
                    "HTTP write failed: {error}"
                );
                return None;
            }
        }
    }
}