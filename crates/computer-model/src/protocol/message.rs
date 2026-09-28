use std::io::{Read, Write};

use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    protocol::{event::Event, request::Request},
    Library, Registry,
};

pub trait ProtocolMessage: Serialize + DeserializeOwned {
    fn encode(&self) -> Result<Vec<u8>, String> {
        let bytes = postcard::to_allocvec(self)
            .map_err(|e| e.to_string())?;

        let mut encoder =
            DeflateEncoder::new(Vec::new(), Compression::new(9));

        encoder
            .write_all(&bytes)
            .map_err(|e| e.to_string())?;

        encoder
            .finish()
            .map_err(|e| e.to_string())
    }

    fn decode(bytes: &[u8]) -> Result<Self, String> {
        let mut decoder = DeflateDecoder::new(bytes);
        let mut data = Vec::new();

        decoder
            .read_to_end(&mut data)
            .map_err(|e| e.to_string())?;

        postcard::from_bytes(&data)
            .map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Hello {
        version: u32,
    },
    
    Request {
        id: u64,
        request: Request,
    },
}

impl ProtocolMessage for ClientMessage {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Handshake(Result<u32, String>),

    Library(Library),
    Registry(Registry),

    Response {
        id: u64,
        result: Result<(), String>,
    },

    Event {
        event: Event,
    },
}

impl ProtocolMessage for ServerMessage {}