use std::io::{Read, Write};

use flate2::{Compression, read::DeflateDecoder, write::DeflateEncoder};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    Library, Registry,
    id::Token,
    protocol::{event::Event, request::Request},
};

const MAX_DECOMPRESSED_SIZE: u64 = 8 * 1024 * 1024;

pub trait ProtocolMessage: Serialize + DeserializeOwned {
    fn encode(&self) -> Result<Vec<u8>, String> {
        let bytes = postcard::to_allocvec(self).map_err(|e| e.to_string())?;

        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::fast());

        encoder.write_all(&bytes).map_err(|e| e.to_string())?;

        encoder.finish().map_err(|e| e.to_string())
    }

    fn decode(bytes: &[u8]) -> Result<Self, String> {
        let decoder = DeflateDecoder::new(bytes);
        let mut data = Vec::new();

        decoder
            .take(MAX_DECOMPRESSED_SIZE + 1)
            .read_to_end(&mut data)
            .map_err(|e| e.to_string())?;

        if data.len() as u64 > MAX_DECOMPRESSED_SIZE {
            return Err("Message exceeds the decompressed size limit".into());
        }

        postcard::from_bytes(&data).map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Hello { version: u32, token: Token },

    Request { id: u64, request: Request },
}

impl ProtocolMessage for ClientMessage {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Handshake(Result<u32, String>),

    Library(Library),
    Registry(Registry),

    Response { id: u64, result: Result<(), String> },

    Event { event: Event },
}

impl ProtocolMessage for ServerMessage {}
