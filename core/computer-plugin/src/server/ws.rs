use std::net::TcpStream;
use std::sync::{Mutex, OnceLock};

use tracing::{info, warn};
use tungstenite::{HandshakeError, Message, WebSocket, accept};

struct Client {
    ws: WebSocket<TcpStream>,
}

impl Client {
    fn send_text(&mut self, text: &str) -> bool {
        self.ws.send(Message::Text(text.into())).is_ok()
    }
}

static CLIENTS: OnceLock<Mutex<Vec<Client>>> = OnceLock::new();

fn clients() -> &'static Mutex<Vec<Client>> {
    CLIENTS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn register(stream: TcpStream) {
    if stream.set_nonblocking(false).is_err() {
        return;
    }

    match accept(stream) {
        Ok(ws) => {
            if ws.get_ref().set_nonblocking(true).is_err() {
                return;
            }
            let mut client = Client { ws };
            client.send_text(r#"{"type":"ready"}"#);
            info!("WS: client connecté");
            clients().lock().unwrap().push(client);
        }
        Err(HandshakeError::Failure(e)) => warn!("WS handshake échoué: {e}"),
        Err(HandshakeError::Interrupted(_)) => {}
    }
}

pub fn poll_all() {
    clients().lock().unwrap().retain_mut(|client| {
        loop {
            match client.ws.read() {
                Ok(Message::Text(text)) => {
                    let _ = client.send_text(&text);
                }
                Ok(Message::Ping(data)) => {
                    let _ = client.ws.send(Message::Pong(data));
                }
                Ok(Message::Close(_)) => return false,
                Ok(_) => {}
                Err(tungstenite::Error::Io(ref e))
                    if e.kind() == std::io::ErrorKind::WouldBlock =>
                {
                    return true;
                }
                Err(_) => return false,
            }
        }
    });
}
