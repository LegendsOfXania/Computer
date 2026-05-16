use computer_api::command::{ComputerCommand, register_command};
use pumpkin_plugin_api::{
    Result, Server,
    command::{CommandError, CommandNode, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    text::TextComponent,
};

pub struct PanelExecutor;

impl CommandHandler for PanelExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        sender.send_message(TextComponent::text("Panel command run succesfully!"));
        Ok(1)
    }
}

impl ComputerCommand for PanelExecutor {
    fn register() {
        register_command(|| CommandNode::literal("panel").execute(PanelExecutor));
    }
}
