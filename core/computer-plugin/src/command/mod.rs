use computer_api::command::{ComputerCommand, get_commands};
use pumpkin_plugin_api::command::Command;

pub mod panel;

pub fn register_internal() {
    panel::PanelCommand::register();
}

pub fn init_command_tree() -> Command {
    let aliases = ["computer".to_string(), "cp".to_string()];
    let description = "The main computer command";

    let command = Command::new(&aliases[..], description);

    for factory in get_commands() {
        command.then(factory());
    }

    command
}
