// thanks you claude, I didn't want to write a mock

use computer_model::{
    Library, Registry, entry::Entry, key::EntryKey, page::{Page, PageKind}, protocol::{PROTOCOL_VERSION, message::{ClientMessage, ProtocolMessage, ServerMessage}},
};
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message as WsMessage;

fn sample_library() -> Library {
    Library {
        pages: vec![
            Page {
                id: 1,
                name: "Forêt".into(),
                kind: PageKind::Sequence,
                priority: 0,
                chapter: None,
                entries: vec![
                    EntryKey::new(1, 1),
                    EntryKey::new(1, 2),
                    EntryKey::new(1, 3),
                ],
            },
            Page {
                id: 2,
                name: "Village".into(),
                kind: PageKind::Sequence,
                priority: 1,
                chapter: Some("lieu".into()),
                entries: vec![
                    EntryKey::new(2, 1),
                    EntryKey::new(2, 2),
                ],
            },
            Page {
                id: 3,
                name: "Personnages".into(),
                kind: PageKind::Static,
                priority: 2,
                chapter: Some("NPCs".into()),
                entries: vec![
                    EntryKey::new(3, 1),
                    EntryKey::new(3, 2),
                    EntryKey::new(3, 3),
                    EntryKey::new(3, 4),
                    EntryKey::new(3, 5),
                    EntryKey::new(3, 6),
                    EntryKey::new(3, 7),
                    EntryKey::new(3, 8),
                ],
            },
            Page {
                id: 4,
                name: "Quêtes".into(),
                kind: PageKind::Sequence,
                priority: 3,
                chapter: Some("config.crotte".into()),
                entries: vec![
                    EntryKey::new(4, 1),
                    EntryKey::new(4, 2),
                ],
            },
            Page {
                id: 5,
                name: "Objets".into(),
                kind: PageKind::Static,
                priority: 4,
                chapter: Some("config.caca".into()),
                entries: vec![
                    EntryKey::new(5, 1),
                    EntryKey::new(5, 2),
                    EntryKey::new(5, 3),
                ],
            },
        ],
        entries: vec![
            Entry {
                key: EntryKey::new(1, 1),
                kind: "location".into(),
                version: 1,
                fields: Default::default(),
            },
            Entry {
                key: EntryKey::new(1, 2),
                kind: "location".into(),
                version: 1,
                fields: Default::default(),
            },
            Entry {
                key: EntryKey::new(1, 3),
                kind: "location".into(),
                version: 1,
                fields: Default::default(),
            },
            Entry {
                key: EntryKey::new(2, 1),
                kind: "location".into(),
                version: 1,
                fields: Default::default(),
            },
            Entry {
                key: EntryKey::new(2, 2),
                kind: "location".into(),
                version: 1,
                fields: Default::default(),
            },
            Entry {
                key: EntryKey::new(3, 1),
                kind: "npc".into(),
                version: 1,
                fields: Default::default(),
            },
            Entry {
                key: EntryKey::new(3, 2),
                kind: "npc".into(),
                version: 1,
                fields: Default::default(),
            },
            Entry {
                key: EntryKey::new(3, 3),
                kind: "npc".into(),
                version: 1,
                fields: Default::default(),
            },
            Entry {
                key: EntryKey::new(3, 4),
                kind: "npc".into(),
                version: 1,
                fields: Default::default(),
            },
            Entry {
                key: EntryKey::new(4, 1),
                kind: "quest".into(),
                version: 1,
                fields: Default::default(),
            },
            Entry {
                key: EntryKey::new(4, 2),
                kind: "quest".into(),
                version: 1,
                fields: Default::default(),
            },
            Entry {
                key: EntryKey::new(5, 1),
                kind: "item".into(),
                version: 1,
                fields: Default::default(),
            },
            Entry {
                key: EntryKey::new(5, 2),
                kind: "item".into(),
                version: 1,
                fields: Default::default(),
            },
            Entry {
                key: EntryKey::new(5, 3),
                kind: "item".into(),
                version: 1,
                fields: Default::default(),
            },
        ],
    }
}

fn sample_registry() -> Registry {
    Registry(vec![])
}

async fn handle_connection(stream: TcpStream) {
    println!("Nouvelle connexion : {:?}", stream.peer_addr().ok());

    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(err) => {
            eprintln!("Échec du handshake WebSocket : {err}");
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    while let Some(frame) = read.next().await {
        let frame = match frame {
            Ok(frame) => frame,
            Err(err) => {
                eprintln!("Erreur de lecture WS : {err}");
                break;
            }
        };

        let bytes = match frame {
            WsMessage::Binary(bytes) => bytes,
            WsMessage::Close(_) => break,
            _ => continue,
        };

        let client_message = match ClientMessage::decode(&bytes) {
            Ok(message) => message,
            Err(err) => {
                eprintln!("Décodage ClientMessage échoué : {err:?}");
                continue;
            }
        };

        match client_message {
            ClientMessage::Hello { version } => {
                println!("Hello reçu : version={version}");
                send(&mut write, &ServerMessage::Handshake(Ok(PROTOCOL_VERSION))).await;
                send(&mut write, &ServerMessage::Library(sample_library())).await;
                send(&mut write, &ServerMessage::Registry(sample_registry())).await;
            }
            ClientMessage::Request { id, request } => {
                println!("Request reçue (id={id}) : {request:?}");
                send(&mut write, &ServerMessage::Response { id, result: Ok(()) }).await;
            }
        }
    }
}

async fn send(
    write: &mut (impl SinkExt<WsMessage, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    message: &ServerMessage,
) {
    match message.encode() {
        Ok(bytes) => {
            if let Err(err) = write.send(WsMessage::Binary(bytes)).await {
                eprintln!("Envoi échoué : {err}");
            }
        }
        Err(err) => eprintln!("Encodage ServerMessage échoué : {err:?}"),
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8082")
        .await
        .expect("impossible d'écouter sur 127.0.0.1:8082");

    println!("Mock engine à l'écoute sur ws://127.0.0.1:8082");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}