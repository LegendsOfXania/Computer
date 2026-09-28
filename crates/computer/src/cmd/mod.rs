use pumpkin_plugin_api::{Context, command::Command};

mod launch;

pub fn register(context: &Context) {
    let cmd = Command::new(&["computer".to_string(), "cp".to_string()], "The main command of the Computer plugin")
        .then(launch::node());

    context.register_command(cmd, "computer.use");
}