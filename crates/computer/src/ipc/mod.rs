mod registry;

use computer_model::ipc::{ExtensionMessage, IpcMessage};
use pumpkin_plugin_api::Result;

pub fn handle(sender: String, message: Vec<u8>) -> Result<Vec<u8>> {
    let message = ExtensionMessage::decode(&message)
        .map_err(|error| error.to_string())?;

    match message {
        ExtensionMessage::Registry(registry) => {
            registry::handle(sender, registry)?;
        }
    }

    Ok(Vec::new())
}