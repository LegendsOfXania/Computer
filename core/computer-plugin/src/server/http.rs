use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub struct Request<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub query: &'a str,
}

pub fn parse<'a>(raw: &'a str) -> Option<Request<'a>> {
    let first_line = raw.lines().next()?;
    let mut parts = first_line.splitn(3, ' ');

    let method = parts.next()?;
    let full_path = parts.next()?;

    let (path, query) = match full_path.split_once('?') {
        Some((p, q)) => (p, q),
        None => (full_path, ""),
    };

    Some(Request {
        method,
        path,
        query,
    })
}

pub fn is_ws_upgrade(raw: &str) -> bool {
    let lower = raw.to_ascii_lowercase();
    lower.contains("upgrade: websocket") && lower.contains("connection:")
}

pub fn respond(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) {
    let status_text = status_text(status);
    let header = format!(
        "HTTP/1.1 {status} {status_text}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n",
        body.len(),
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
}

pub fn respond_json_error(stream: &mut TcpStream, status: u16, message: &str) {
    let body = format!(r#"{{"error":"{message}"}}"#);
    respond(stream, status, "application/json", body.as_bytes());
}

pub fn read_request(stream: &mut TcpStream) -> Option<String> {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).ok()?;
    if n == 0 {
        return None;
    }
    Some(String::from_utf8_lossy(&buf[..n]).into_owned())
}

fn status_text(code: u16) -> &'static str {
    match code {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Internal Server Error",
    }
}
