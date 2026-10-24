mod registry;

use computer_model::ipc::{ComputerMessage, ExtensionMessage, IpcMessage};
use pumpkin_plugin_api::{Result, ipc::send_ipc_message};

pub fn handle(sender: String, message: Vec<u8>) -> Result<Vec<u8>> {
    let message = ExtensionMessage::decode(&message).map_err(|error| error.to_string())?;

    match message {
        ExtensionMessage::Registry(registry) => {
            registry::handle(sender, registry)?;
        }
    }

    Ok(Vec::new())
}

pub fn pull(extension: &str) {
    let request = match ComputerMessage::RequestRegistry.encode() {
        Ok(request) => request,
        Err(error) => return tracing::error!("Could not encode the registry request: {error}"),
    };

    match send_ipc_message(extension, &request) {
        Ok(Ok(response)) => {
            if let Err(error) = handle(extension.to_owned(), response) {
                tracing::error!("Invalid registry from {extension}: {error}");
            }
        }
        Ok(Err(reason)) => tracing::warn!("{extension} did not give its registry: {reason}"),
        Err(()) => tracing::warn!(
            "{extension} is not loaded (yet): it will register itself if it loads after Computer"
        ),
    }
}
