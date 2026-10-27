pub mod registry;

use computer_model::ipc::{ComputerMessage, IpcMessage};

const COMPUTER: &str = "computer";

pub fn handle_ipc(sender: &str, message: &[u8]) -> Result<Vec<u8>, String> {
    if sender != COMPUTER {
        return Err(format!("Unexpected IPC sender `{sender}`"));
    }

    match ComputerMessage::decode(message)? {
        ComputerMessage::RequestRegistry => registry::answer()?.encode(),

        ComputerMessage::ActiveKinds(kinds) => {
            *registry::ACTIVE_KINDS
                .lock()
                .map_err(|_| "Active kinds lock poisoned".to_owned())? = kinds;

            Ok(Vec::new())
        }

        ComputerMessage::Trigger { .. } => {
            Err("Trigger handling has not been implemented yet".into())
        }

        ComputerMessage::Cancel { .. } => {
            Err("Cancel handling has not been implemented yet".into())
        }
    }
}
