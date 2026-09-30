use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::Registry;

pub trait IpcMessage: Serialize + DeserializeOwned {
    fn encode(&self) -> Result<Vec<u8>, String> {
        postcard::to_allocvec(self).map_err(|err| err.to_string())
    }

    fn decode(bytes: &[u8]) -> Result<Self, String> {
        postcard::from_bytes(bytes).map_err(|err| err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtensionMessage {
    Registry(Registry)
}

impl IpcMessage for ExtensionMessage {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputerMessage {
    // there's nothing here for the moment
}

impl IpcMessage for ComputerMessage {}