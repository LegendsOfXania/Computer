use crate::{config::ComputerConfig, server};
use computer_api::command::{ComputerCommand, register_command};
use pumpkin_plugin_api::{
    Result, Server,
    command::{CommandError, CommandNode, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    text::TextComponent,
};
use std::str::FromStr;
use uuid::Uuid;

pub struct PanelExecutor;

impl CommandHandler for PanelExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let config = ComputerConfig::get();

        if let Some(player) = sender.as_player() {
            if !config.panel_active() {
                sender.send_message(TextComponent::text("The panel is disabled on this server."));
                return Ok(0);
            }

            if !server::is_running() {
                server::start(config.http_addr(), config.ws_addr());
            }

            let uuid = Uuid::from_str(&player.get_id()).unwrap_or(Uuid::nil());
            let token = server::session::create_token(uuid);

            player.send_system_message(
                TextComponent::text(&format!(
                    "Open the panel: http://{}?token={}",
                    config.http_addr(),
                    token
                )),
                false,
            );
        }

        Ok(1)
    }
}

impl ComputerCommand for PanelExecutor {
    fn register() {
        register_command(|| CommandNode::literal("panel").execute(PanelExecutor));
    }
}
