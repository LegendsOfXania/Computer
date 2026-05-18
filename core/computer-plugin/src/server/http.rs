use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};
use tracing::{error, info};

pub fn listen(addr: &str) {
    let listener = match TcpListener::bind(addr) {
        Ok(listener) => listener,
        Err(err) => {
            error!(reason = %err, "Failed to bind HTTP server on {}", addr);
            return;
        }
    };
    info!("Computer HTTP server listening on http://{}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle(stream));
            }
            Err(err) => {
                error!(reason = %err, "Failed to accept HTTP connection");
            }
        }
    }
}

fn handle(mut stream: TcpStream) {
    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return,
    };

    let request = String::from_utf8_lossy(&buf[..n]);
    let first_line = request.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();

    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("").split('?').next().unwrap_or("");

    match (method, path) {
        ("GET", "/health") => respond(&mut stream, 200, "text/plain", b"ok"),
        _ => respond(&mut stream, 404, "text/plain", b"Not Found"),
    }
}

fn respond(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) {
    let status_text = match status {
        200 => "OK",
        404 => "Not Found",
        _ => "Internal Server Error",
    };

    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        status_text,
        content_type,
        body.len(),
    );

    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
}
