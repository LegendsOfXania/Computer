use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use pumpkin_plugin_api::{scheduler::SchedulerExt, Context};
use sha1::{Digest, Sha1};

const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

const BINARY: u8 = 0x2;
const CLOSE: u8 = 0x8;
const PING: u8 = 0x9;
const PONG: u8 = 0xA;

struct Connection {
    stream: TcpStream,
    handshake: bool,
    buffer: Vec<u8>,
    closed: bool,
}

static SERVER: Mutex<Option<TcpListener>> = Mutex::new(None);
static CONNECTIONS: Mutex<Vec<Connection>> = Mutex::new(Vec::new());

pub fn start(context: &Context, port: u16) -> Result<(), String> {
    let listener = TcpListener::bind(("0.0.0.0", port)).map_err(|e| e.to_string())?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;

    *SERVER.lock().unwrap() = Some(listener);
    context.schedule_repeating_task(0, 1, |_| poll());

    Ok(())
}

fn poll() {
    accept();

    let mut connections = CONNECTIONS.lock().unwrap();

    for connection in &mut *connections {
        connection.poll();
    }

    connections.retain(|connection| !connection.closed);
}

fn accept() {
    let server = SERVER.lock().unwrap();
    let Some(listener) = server.as_ref() else {
        return;
    };

    loop {
        let Ok((stream, _)) = listener.accept() else {
            break;
        };

        if stream.set_nonblocking(true).is_ok() {
            CONNECTIONS.lock().unwrap().push(Connection {
                stream,
                handshake: false,
                buffer: Vec::new(),
                closed: false,
            });
        }
    }
}

impl Connection {
    fn poll(&mut self) {
        let mut bytes = [0; 4096];

        loop {
            match self.stream.read(&mut bytes) {
                Ok(0) | Err(_) => {
                    self.closed = true;
                    return;
                }
                Ok(n) => self.buffer.extend_from_slice(&bytes[..n]),
            }

            if self.buffer.len() < bytes.len() {
                break;
            }
        }

        if !self.handshake {
            self.handshake();
            return;
        }

        while let Some((used, frame)) = decode(&self.buffer) {
            self.buffer.drain(..used);

            match frame {
                Frame::Data(data) => self.on_data(data),
                Frame::Ping(data) => self.send(PONG, &data),
                Frame::Close => {
                    self.send(CLOSE, &[]);
                    self.closed = true;
                    return;
                }
                Frame::Ignored => {}
            }
        }
    }

    fn handshake(&mut self) {
        let Some(end) = find(&self.buffer, b"\r\n\r\n") else {
            return;
        };

        let request = String::from_utf8_lossy(&self.buffer[..end]);
        let key = request.lines().find_map(|line| {
            let (name, value) = line.split_once(':')?;
            (name.eq_ignore_ascii_case("sec-websocket-key")).then_some(value.trim())
        });

        let Some(key) = key else {
            self.closed = true;
            return;
        };

        let hash = Sha1::digest(format!("{key}{WS_GUID}").as_bytes());
        let accept = BASE64.encode(hash);

        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {accept}\r\n\r\n"
        );

        if self.stream.write_all(response.as_bytes()).is_err() {
            self.closed = true;
            return;
        }

        self.buffer.drain(..end + 4);
        self.handshake = true;
    }

    fn on_data(&mut self, data: Vec<u8>) {
        tracing::info!("panel -> {} bytes", data.len());
        // TODO: decode computer-model message.
    }

    fn send(&mut self, opcode: u8, data: &[u8]) {
        let len = data.len();

        let mut header = Vec::with_capacity(10);
        header.push(0x80 | opcode);

        match len {
            0..=125 => header.push(len as u8),
            126..=65535 => {
                header.push(126);
                header.extend_from_slice(&(len as u16).to_be_bytes());
            }
            _ => {
                header.push(127);
                header.extend_from_slice(&(len as u64).to_be_bytes());
            }
        }

        if self.stream.write_all(&header).is_err()
            || self.stream.write_all(data).is_err()
        {
            self.closed = true;
        }
    }

    #[allow(dead_code)]
    pub fn send_binary(&mut self, data: &[u8]) {
        self.send(BINARY, data);
    }
}

enum Frame {
    Data(Vec<u8>),
    Ping(Vec<u8>),
    Close,
    Ignored,
}

fn decode(buf: &[u8]) -> Option<(usize, Frame)> {
    if buf.len() < 2 {
        return None;
    }

    let opcode = buf[0] & 0x0F;
    let masked = buf[1] & 0x80 != 0;
    let mut len = (buf[1] & 0x7F) as usize;
    let mut pos = 2;

    match len {
        126 => {
            if buf.len() < pos + 2 {
                return None;
            }
            len = u16::from_be_bytes(buf[pos..pos + 2].try_into().ok()?) as usize;
            pos += 2;
        }
        127 => {
            if buf.len() < pos + 8 {
                return None;
            }
            len = usize::try_from(u64::from_be_bytes(buf[pos..pos + 8].try_into().ok()?)).ok()?;
            pos += 8;
        }
        _ => {}
    }

    let mask = if masked {
        if buf.len() < pos + 4 {
            return None;
        }

        let mask: [u8; 4] = buf[pos..pos + 4].try_into().ok()?;
        pos += 4;
        Some(mask)
    } else {
        None
    };

    if buf.len() < pos + len {
        return None;
    }

    let mut data = buf[pos..pos + len].to_vec();

    if let Some(mask) = mask {
        for (i, byte) in data.iter_mut().enumerate() {
            *byte ^= mask[i & 3];
        }
    }

    let frame = match opcode {
        BINARY => Frame::Data(data),
        PING => Frame::Ping(data),
        CLOSE => Frame::Close,
        _ => Frame::Ignored,
    };

    Some((pos + len, frame))
}

fn find(data: &[u8], pattern: &[u8]) -> Option<usize> {
    data.windows(pattern.len()).position(|window| window == pattern)
}