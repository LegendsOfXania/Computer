use computer_api::command::{ComputerCommand, register_command};
use pumpkin_plugin_api::{
    Result, Server,
    command::{CommandError, CommandNode, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    scheduler::SchedulerExt,
    text::TextComponent,
};

use crate::{config::ComputerConfig, server};

pub struct PanelCommand;

impl CommandHandler for PanelCommand {
    fn handle(
        &self,
        sender: CommandSender,
        minecraft_server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let config = ComputerConfig::get();

        if !config.panel_active() {
            sender.send_message(TextComponent::text("Panel is disabled."));
            return Ok(0);
        }

        if !server::is_running() {
            if let Err(e) = server::start(&config.panel_addr()) {
                sender.send_message(TextComponent::text(&format!(
                    "Could not start the panel: {e}"
                )));
                return Ok(0);
            }

            minecraft_server.schedule_repeating_task(1, 1, move |_| {
                server::poll();
            });
        }

        let url = format!("http://{}", config.panel_addr());
        sender.send_message(TextComponent::text(&format!("Panel: {url}")));
        Ok(1)
    }
}

impl ComputerCommand for PanelCommand {
    fn register() {
        register_command(|| CommandNode::literal("panel").execute(PanelCommand));
        register_command(|| CommandNode::literal("p").execute(PanelCommand));
    }
}
