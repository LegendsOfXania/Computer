use std::io::{Read, Write};

use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{Library, Registry, protocol::{error::ProtocolError, event::Event, request::Request}};

pub trait ProtocolMessage: Serialize + DeserializeOwned  {
    fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let bytes = postcard::to_allocvec(self).map_err(|_| ProtocolError::Internal)?;
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(9));
        encoder.write_all(&bytes).map_err(|_| ProtocolError::Internal)?;
        encoder.finish().map_err(|_| ProtocolError::Internal)
    }

    fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let mut decoder = DeflateDecoder::new(bytes);
        let mut data = Vec::new();
        decoder.read_to_end(&mut data).map_err(|_| ProtocolError::Internal)?;
        postcard::from_bytes(&data).map_err(|_| ProtocolError::Internal)
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
        result: Result<(), ProtocolError> 
    },
    
    Event {
        event: Event
    },
}

impl ProtocolMessage for ServerMessage {}
