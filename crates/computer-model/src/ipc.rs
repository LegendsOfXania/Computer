use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::Registry;

pub const IPC_VERSION: u32 = 1;

pub trait IpcMessage: Serialize + DeserializeOwned {
    fn encode(&self) -> Result<Vec<u8>, String> {
        postcard::to_allocvec(&(IPC_VERSION, self)).map_err(|err| err.to_string())
    }

    fn decode(bytes: &[u8]) -> Result<Self, String> {
        let (version, payload) =
            postcard::take_from_bytes::<u32>(bytes).map_err(|err| err.to_string())?;

        if version != IPC_VERSION {
            return Err(format!(
                "IPC version mismatch: received v{version}, expected v{IPC_VERSION} \
                 (Computer and the extension are built against different versions)"
            ));
        }

        postcard::from_bytes(payload).map_err(|err| err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtensionMessage {
    Registry(Registry),
}

impl IpcMessage for ExtensionMessage {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputerMessage {
    RequestRegistry,
}

impl IpcMessage for ComputerMessage {}
