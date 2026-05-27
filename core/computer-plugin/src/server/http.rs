use std::io::{Read, Write};
use std::net::TcpStream;

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/panel/"]
struct Panel;

pub struct Request<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub raw: &'a str,
}

pub fn read_raw(stream: &mut TcpStream) -> Option<String> {
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf).ok()?;
    if n == 0 {
        return None;
    }
    Some(String::from_utf8_lossy(&buf[..n]).into_owned())
}

pub fn is_websocket_upgrade(raw: &str) -> bool {
    let lower = raw.to_ascii_lowercase();
    lower.contains("upgrade: websocket")
}

pub fn parse<'a>(raw: &'a str) -> Option<Request<'a>> {
    let line = raw.lines().next()?;
    let mut parts = line.splitn(3, ' ');
    let method = parts.next()?;
    let full_path = parts.next()?;
    let path = full_path.split('?').next()?;
    Some(Request { method, path, raw })
}

pub fn extract_token<'a>(raw: &'a str) -> Option<&'a str> {
    let first_line = raw.lines().next()?;
    let query = first_line.split('?').nth(1)?;
    let query = query.split(' ').next()?;
    for param in query.split('&') {
        if let Some(value) = param.strip_prefix("token=") {
            return Some(value);
        }
    }
    None
}

pub fn serve_asset(stream: &mut TcpStream, path: &str) {
    let key = path.trim_start_matches('/');
    let key = if key.is_empty() { "index.html" } else { key };

    let file = Panel::get(key).or_else(|| Panel::get("index.html"));

    let Some(file) = file else {
        write_response(stream, 404, "text/plain", b"not found");
        return;
    };

    let mime = match key.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "application/javascript",
        "css" => "text/css",
        _ => "text/plain",
    };

    write_response(stream, 200, mime, file.data.as_ref());
}

pub fn reject_unauthorized(stream: &mut TcpStream) {
    let _ = write!(
        stream,
        "HTTP/1.1 401 Unauthorized\r\nContent-Type: text/plain\r\nContent-Length: 12\r\nConnection: close\r\n\r\nUnauthorized"
    );
}

fn write_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) {
    let status_text = if status == 200 { "OK" } else { "Not Found" };
    let _ = write!(
        stream,
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(body);
}
