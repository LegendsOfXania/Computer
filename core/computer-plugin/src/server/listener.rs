use std::net::{TcpListener, TcpStream};
use std::sync::OnceLock;

use tracing::{error, warn};

use super::{auth, http, ws};

static LISTENER: OnceLock<TcpListener> = OnceLock::new();

pub fn start(addr: &str) -> Result<(), String> {
    if is_running() {
        return Err("already started".into());
    }
    let listener = TcpListener::bind(addr).map_err(|e| e.to_string())?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    LISTENER.set(listener).map_err(|_| "set failed".into())
}

pub fn is_running() -> bool {
    LISTENER.get().is_some()
}

pub fn accept_pending() {
    let Some(listener) = LISTENER.get() else {
        return;
    };
    loop {
        match listener.accept() {
            Ok((stream, _)) => dispatch(stream),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => {
                error!("accept: {e}");
                break;
            }
        }
    }
}

fn dispatch(mut stream: TcpStream) {
    if stream.set_nonblocking(false).is_err() {
        return;
    }

    let raw = match http::read_raw(&mut stream) {
        Some(r) => r,
        None => return,
    };

    if http::is_websocket_upgrade(&raw) {
        let provided = http::extract_token(&raw);

        match provided {
            Some(t) if auth::validate(t) => {
                ws::register(stream);
            }
            Some(_) => {
                warn!("WS: token invalide ou expiré, connexion refusée");
                http::reject_unauthorized(&mut stream);
            }
            None => {
                warn!("WS: aucun token fourni, connexion refusée");
                http::reject_unauthorized(&mut stream);
            }
        }
    } else if let Some(req) = http::parse(&raw) {
        http::serve_asset(&mut stream, req.path);
    }
}
