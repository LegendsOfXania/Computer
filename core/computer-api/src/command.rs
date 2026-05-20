use std::sync::Mutex;

use pumpkin_plugin_api::command::CommandNode;

pub type ComputerCommandFactory = fn() -> CommandNode;

static REGISTRY: Mutex<Vec<ComputerCommandFactory>> = Mutex::new(Vec::new());

pub fn register_command(factory: ComputerCommandFactory) {
    REGISTRY
        .lock()
        .expect("Computer command registry is not initialized")
        .push(factory);
}

pub fn get_commands() -> Vec<ComputerCommandFactory> {
    REGISTRY
        .lock()
        .expect("Computer command registry is not initialized")
        .clone()
}

pub trait ComputerCommand {
    fn register();
}
