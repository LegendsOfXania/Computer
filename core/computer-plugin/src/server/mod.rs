use std::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, OnceLock},
};
use tracing::{error, info};

pub mod http;
pub mod session;
pub mod ws;

static LISTENER: OnceLock<Mutex<TcpListener>> = OnceLock::new();

pub fn start(addr: String) -> Result<(), String> {
    if LISTENER.get().is_some() {
        return Err("Server is already running".into());
    }

    let listener = TcpListener::bind(&addr)
        .map_err(|e| format!("Failed to bind HTTP/WS listener on {addr}: {e}"))?;

    listener
        .set_nonblocking(true)
        .map_err(|e| format!("set_nonblocking failed: {e}"))?;

    info!("Computer HTTP/WS server listening on http://{addr}");

    LISTENER
        .set(Mutex::new(listener))
        .map_err(|_| "LISTENER already set".to_string())?;

    Ok(())
}

pub fn is_running() -> bool {
    LISTENER.get().is_some()
}

pub fn poll() {
    accept_pending();
    ws::poll_clients();
}

fn accept_pending() {
    let Some(listener) = LISTENER.get() else {
        return;
    };
    let Ok(guard) = listener.lock() else { return };

    loop {
        match guard.accept() {
            Ok((stream, addr)) => {
                if let Err(e) = stream.set_nonblocking(true) {
                    error!(reason = %e, %addr, "Could not set stream non-blocking, dropping");
                    continue;
                }
                handle_connection(stream);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => {
                error!(reason = %e, "HTTP accept error");
                break;
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let raw = match http::read_request(&mut stream) {
        Some(r) => r,
        None => return,
    };

    if http::is_ws_upgrade(&raw) {
        ws::accept_stream(stream);
        return;
    }

    let Some(req) = http::parse(&raw) else {
        http::respond(&mut stream, 400, "text/plain", b"Bad Request");
        return;
    };

    route_http(&mut stream, &req);
}

fn route_http(stream: &mut TcpStream, req: &http::Request<'_>) {
    match (req.method, req.path) {
        ("GET", "/health") => {
            http::respond(stream, 200, "application/json", b"{\"status\":\"ok\"}");
        }
        _ => {
            http::respond_json_error(stream, 404, "not found");
        }
    }
}
