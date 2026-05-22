use crate::config::ComputerConfig;
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
        let config = ComputerConfig::get();

        if !config.panel_active() {
            sender.send_message(TextComponent::text("The panel is disabled on this server."));
            return Ok(0);
        }

        // TODO: implement panel opening logic

        Ok(1)
    }
}

impl ComputerCommand for PanelExecutor {
    fn register() {
        register_command(|| CommandNode::literal("panel").execute(PanelExecutor));
        register_command(|| CommandNode::literal("p").execute(PanelExecutor));
    }
}
