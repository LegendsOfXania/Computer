use std::net::{TcpListener, TcpStream};
use std::sync::OnceLock;
use tracing::{error, info};

pub mod assets;
pub mod http;
pub mod session;
pub mod ws;

static LISTENER: OnceLock<TcpListener> = OnceLock::new();

pub fn start(addr: &str) -> Result<(), String> {
    if LISTENER.get().is_some() {
        return Err("Server is already running".into());
    }

    let listener = TcpListener::bind(addr)
        .map_err(|e| format!("Failed to bind HTTP/WS listener on {addr}: {e}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("set_nonblocking failed: {e}"))?;

    info!("Computer HTTP/WS server listening on http://{addr}");
    LISTENER.set(listener).ok();
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

    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                if let Err(e) = stream.set_nonblocking(true) {
                    error!(%addr, reason = %e, "Could not set stream non-blocking, dropping");
                    continue;
                }
                handle_connection(stream);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => {
                error!(reason = %e, "Accept error");
                break;
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let Some(raw) = http::read_request(&mut stream) else {
        return;
    };

    if http::is_ws_upgrade(&raw) {
        ws::accept_stream(stream);
        return;
    }

    let Some(req) = http::parse(&raw) else {
        http::respond(&mut stream, 400, "text/plain", b"Bad Request");
        return;
    };

    if req.path.starts_with("/api/") {
        route_api(&mut stream, &req);
    } else {
        http::respond_asset::<assets::Panel>(&mut stream, req.path);
    }
}

fn route_api(stream: &mut TcpStream, req: &http::Request<'_>) {
    use serde_json::json;
    match (req.method, req.path) {
        ("GET", "/api/health") => {
            http::respond_json(stream, 200, &json!({ "status": "ok" }));
        }
        ("POST", "/api/auth/token") => {
            let token = session::create_token();
            http::respond_json(stream, 200, &json!({ "token": token }));
        }
        _ => {
            http::respond_json(stream, 404, &json!({ "error": "not found" }));
        }
    }
}
