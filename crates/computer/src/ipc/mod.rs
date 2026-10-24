mod registry;

use computer_model::ipc::{ComputerMessage, ExtensionMessage, IpcMessage};
use pumpkin_plugin_api::{
    Result, Server,
    events::{EventData, EventHandler, ServerLoadEvent},
    events_wit::ServerLoadType,
    ipc::send_ipc_message,
};

use crate::data;

pub fn handle(sender: String, message: Vec<u8>) -> Result<Vec<u8>> {
    let message = ExtensionMessage::decode(&message).map_err(|error| error.to_string())?;

    match message {
        ExtensionMessage::Registry(registry) => {
            registry::handle(sender, registry)?;
        }
    }

    Ok(Vec::new())
}

pub fn ask(extension: &str, message: ComputerMessage) -> Result<()> {
    let request = message.encode()?;

    let response = send_ipc_message(extension, &request)
        .map_err(|()| "this plugin is not loaded".to_string())?
        .map_err(|reason| format!("it did not answer: {reason}"))?;

    handle(extension.to_owned(), response)?;

    Ok(())
}

pub struct OnServerLoad;

impl EventHandler<ServerLoadEvent> for OnServerLoad {
    fn handle(
        &self,
        _server: Server,
        event: EventData<ServerLoadEvent>,
    ) -> EventData<ServerLoadEvent> {
        if matches!(event.load_type, ServerLoadType::Startup) {
            for extension in &data::conf::get_conf().extensions {
                if let Err(error) = ask(extension, ComputerMessage::RequestRegistry) {
                    tracing::warn!("No entries from {extension}: {error}");
                }
            }
        }

        event
    }
}
