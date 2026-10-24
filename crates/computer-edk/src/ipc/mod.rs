pub mod registry;

use computer_model::ipc::{ComputerMessage, IpcMessage};

const COMPUTER: &str = "computer";

pub fn handle_ipc(sender: &str, message: &[u8]) -> Result<Vec<u8>, String> {
    if sender != COMPUTER {
        return Err(format!("unexpected sender `{sender}`"));
    }

    let answer = match ComputerMessage::decode(message)? {
        ComputerMessage::RequestRegistry => registry::answer()?,
    };

    answer.encode()
}
