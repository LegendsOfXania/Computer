use std::{
    fs,
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    sync::Mutex,
};

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use pumpkin_plugin_api::{
    scheduler::SchedulerExt,
    Server,
};
use sha1::{Digest, Sha1};

use crate::data;

const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
    upgraded: bool,
    closed: bool,
}

struct State {
    listener: TcpListener,
    connections: Vec<Connection>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

pub enum StartOutcome {
    Started,
    AlreadyRunning,
}

pub fn ensure_started(
    server: &Server,
    port: u16,
) -> Result<StartOutcome, String> {
    if STATE.lock().map_err(|e| e.to_string())?.is_some() {
        return Ok(StartOutcome::AlreadyRunning);
    }

    let listener =
        TcpListener::bind(("0.0.0.0", port)).map_err(|e| e.to_string())?;

    listener
        .set_nonblocking(true)
        .map_err(|e| e.to_string())?;

    server.schedule_repeating_task(0, 1, |_| poll());

    *STATE.lock().map_err(|e| e.to_string())? = Some(State {
        listener,
        connections: Vec::new(),
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

    accept(state);

    for connection in &mut state.connections {
        connection.poll();
    }

    state.connections.retain(|connection| !connection.closed);
}

fn accept(state: &mut State) {
    loop {
        match state.listener.accept() {
            Ok((stream, _)) => {
                if stream.set_nonblocking(true).is_ok() {
                    state.connections.push(Connection {
                        stream,
                        buffer: Vec::new(),
                        upgraded: false,
                        closed: false,
                    });
                }
            }

            Err(error) if error.kind() == ErrorKind::WouldBlock => break,
            Err(_) => break,
        }
    }
}

impl Connection {
    fn poll(&mut self) {
        if !self.read() {
            self.closed = true;
            return;
        }

        if !self.upgraded {
            self.http();
            return;
        }

        self.websocket();
    }

    fn read(&mut self) -> bool {
        let mut bytes = [0; 4096];

        loop {
            match self.stream.read(&mut bytes) {
                Ok(0) => return false,

                Ok(n) => {
                    self.buffer.extend_from_slice(&bytes[..n]);

                    if n < bytes.len() {
                        return true;
                    }
                }

                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    return true;
                }

                Err(_) => return false,
            }
        }
    }

    fn http(&mut self) {
        let Some(end) = find(&self.buffer, b"\r\n\r\n") else {
            return;
        };

        let request = String::from_utf8_lossy(&self.buffer[..end]);
        let mut lines = request.lines();

        let Some(path) = lines
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .map(str::to_owned)
        else {
            self.closed = true;
            return;
        };

        let key = lines.find_map(|line| {
            let (name, value) = line.split_once(':')?;

            name.eq_ignore_ascii_case("sec-websocket-key")
                .then(|| value.trim().to_owned())
        });

        self.buffer.drain(..end + 4);

        if let Some(key) = key {
            self.websocket_handshake(&key);
        } else {
            self.file(&path);
        }
    }

    fn websocket_handshake(&mut self, key: &str) {
        let hash =
            Sha1::digest(format!("{key}{WS_GUID}").as_bytes());

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

        self.upgraded = true;
    }

    fn file(&mut self, request_path: &str) {
        let Some(data_folder) = data::get_data_folder() else {
            self.send_http(404, "Not Found", "text/plain", b"");
            return;
        };

        let panel_dir = Path::new(data_folder).join("assets/panel");

        let Some(panel_dir) = panel_dir.canonicalize().ok() else {
            self.send_http(404, "Not Found", "text/plain", b"");
            return;
        };

        let request_path = request_path.trim_start_matches('/');
        let request_path = if request_path.is_empty() {
            "index.html"
        } else {
            request_path
        };

        let Some(path) = panel_dir.join(request_path).canonicalize().ok() else {
            self.send_http(404, "Not Found", "text/plain", b"");
            return;
        };

        if !path.starts_with(&panel_dir) {
            self.send_http(404, "Not Found", "text/plain", b"");
            return;
        }

        let Ok(body) = fs::read(&path) else {
            self.send_http(404, "Not Found", "text/plain", b"");
            return;
        };

        self.send_http(200, "OK", content_type(&path), &body);
    }

    fn send_http(
        &mut self,
        status: u16,
        reason: &str,
        content_type: &str,
        body: &[u8],
    ) {
        let response = format!(
            "HTTP/1.1 {status} {reason}\r\n\
             Content-Type: {content_type}\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\r\n",
            body.len()
        );

        if self.stream.write_all(response.as_bytes()).is_err()
            || self.stream.write_all(body).is_err()
        {
            self.closed = true;
            return;
        }

        self.closed = true;
    }

    fn websocket(&mut self) {
        while let Some((used, opcode, data)) = decode(&self.buffer) {
            self.buffer.drain(..used);

            match opcode {
                0x1 | 0x2 => self.on_data(data),

                0x8 => {
                    self.send(0x8, &[]);
                    self.closed = true;
                    return;
                }

                _ => {}
            }
        }
    }

    fn on_data(&mut self, data: Vec<u8>) {
        tracing::info!("panel -> {} bytes", data.len());

        // TODO: decode computer-model message.
    }

    fn send(&mut self, opcode: u8, data: &[u8]) {
        let len = data.len();

        let mut frame = Vec::with_capacity(10 + len);
        frame.push(0x80 | opcode);

        match len {
            0..=125 => frame.push(len as u8),

            126..=65535 => {
                frame.push(126);
                frame.extend_from_slice(&(len as u16).to_be_bytes());
            }

            _ => {
                frame.push(127);
                frame.extend_from_slice(&(len as u64).to_be_bytes());
            }
        }

        frame.extend_from_slice(data);

        if self.stream.write_all(&frame).is_err() {
            self.closed = true;
        }
    }
}

fn decode(buf: &[u8]) -> Option<(usize, u8, Vec<u8>)> {
    if buf.len() < 2 {
        return None;
    }

    let opcode = buf[0] & 0x0f;
    let masked = buf[1] & 0x80 != 0;

    let mut len = (buf[1] & 0x7f) as usize;
    let mut pos = 2;

    match len {
        126 => {
            if buf.len() < pos + 2 {
                return None;
            }

            len = u16::from_be_bytes(
                buf[pos..pos + 2].try_into().ok()?,
            ) as usize;

            pos += 2;
        }

        127 => {
            if buf.len() < pos + 8 {
                return None;
            }

            len = usize::try_from(u64::from_be_bytes(
                buf[pos..pos + 8].try_into().ok()?,
            ))
            .ok()?;

            pos += 8;
        }

        _ => {}
    }

    if !masked {
        return None;
    }

    if buf.len() < pos + 4 {
        return None;
    }

    let mask: [u8; 4] =
        buf[pos..pos + 4].try_into().ok()?;

    pos += 4;

    if buf.len() < pos + len {
        return None;
    }

    let mut data = buf[pos..pos + len].to_vec();

    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= mask[i & 3];
    }

    Some((pos + len, opcode, data))
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("wasm") => "application/wasm",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn find(data: &[u8], pattern: &[u8]) -> Option<usize> {
    data.windows(pattern.len())
        .position(|window| window == pattern)
}