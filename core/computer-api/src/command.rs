use std::sync::{Mutex, OnceLock};

use pumpkin_plugin_api::command::CommandNode;

pub type ComputerCommandFactory = fn() -> CommandNode;

static REGISTRY: OnceLock<Mutex<Vec<ComputerCommandFactory>>> = OnceLock::new();

fn registry() -> &'static Mutex<Vec<ComputerCommandFactory>> {
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn register_command(factory: ComputerCommandFactory) {
    registry()
        .lock()
        .expect("Computer command registry is not initialized")
        .push(factory);
}

pub fn get_commands() -> Vec<ComputerCommandFactory> {
    registry()
        .lock()
        .expect("Computer command registry is not initialized")
        .clone()
}

pub trait ComputerCommand {
    fn register();
}
