use rust_embed::Embed;
use serde::Serialize;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub struct Request<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub query: &'a str,
}

pub fn read_request(stream: &mut TcpStream) -> Option<String> {
    let mut buf = Vec::with_capacity(4096);
    let mut tmp = [0u8; 1024];

    loop {
        match stream.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => {
                buf.extend_from_slice(&tmp[..n]);
                if buf.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
                if buf.len() > 16_384 {
                    return None;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(_) => return None,
        }
    }

    if buf.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&buf).into_owned())
    }
}

pub fn parse(raw: &str) -> Option<Request<'_>> {
    let first_line = raw.lines().next()?;
    let mut parts = first_line.splitn(3, ' ');
    let method = parts.next()?;
    let full_path = parts.next()?;
    let (path, query) = full_path.split_once('?').unwrap_or((full_path, ""));
    Some(Request {
        method,
        path,
        query,
    })
}

pub fn is_ws_upgrade(raw: &str) -> bool {
    let mut has_upgrade_ws = false;
    let mut has_connection_upgrade = false;

    for line in raw.lines().skip(1) {
        if line.is_empty() {
            break;
        }
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("upgrade:") && lower.contains("websocket") {
            has_upgrade_ws = true;
        }
        if lower.starts_with("connection:") && lower.contains("upgrade") {
            has_connection_upgrade = true;
        }
        if has_upgrade_ws && has_connection_upgrade {
            return true;
        }
    }
    false
}

pub fn respond_asset<A: Embed>(stream: &mut TcpStream, path: &str) {
    let key = path.trim_start_matches('/');
    let key = if key.is_empty() { "index.html" } else { key };

    let (resolved_key, data) = match A::get(key) {
        Some(f) => (key, f),
        None => match A::get("index.html") {
            Some(f) => ("index.html", f),
            None => {
                respond(
                    stream,
                    404,
                    "text/plain",
                    b"Panel not built. Run `npm run build` first.",
                );
                return;
            }
        },
    };

    respond(stream, 200, mime_for(resolved_key), &data.data);
}

pub fn respond(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) {
    let header = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status_text(status),
        body.len(),
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
}

pub fn respond_json<T: Serialize>(stream: &mut TcpStream, status: u16, value: &T) {
    match serde_json::to_vec(value) {
        Ok(body) => respond(stream, status, "application/json", &body),
        Err(_) => respond(stream, 500, "text/plain", b"Internal Server Error"),
    }
}

fn mime_for(path: &str) -> &'static str {
    match path.rsplit_once('.').map(|(_, ext)| ext) {
        Some("html") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "application/javascript",
        Some("css") => "text/css",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
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
