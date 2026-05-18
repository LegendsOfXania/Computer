use crate::{config::ComputerConfig, server};
use computer_api::command::{ComputerCommand, register_command};
use pumpkin_plugin_api::{
    Result, Server,
    command::{CommandError, CommandNode, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    scheduler::SchedulerExt,
    text::TextComponent,
};
use tracing::error;

pub struct PanelExecutor;

impl CommandHandler for PanelExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        pumpkin_server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let config = ComputerConfig::get();

        if !config.panel_active() {
            sender.send_message(TextComponent::text("The panel is disabled on this server."));
            return Ok(0);
        }

        if !server::is_running() {
            if let Err(reason) = server::start(config.panel_addr()) {
                error!(%reason, "Failed to start Computer HTTP/WS server");
                sender.send_message(TextComponent::text(
                    "Could not start the panel server. Check server logs for details.",
                ));
                return Ok(0);
            }

            pumpkin_server.schedule_repeating_task(0, 1, |_| {
                server::poll();
            });
        }

        let token = server::session::create_token();
        let url = format!("http://{}?token={}", config.panel_addr(), token);

        let msg = TextComponent::text("Click the link to open the panel: ");
        let link = TextComponent::text(&url);
        link.click_open_url(&url);
        msg.add_child(link);
        sender.send_message(msg);

        Ok(1)
    }
}

impl ComputerCommand for PanelExecutor {
    fn register() {
        register_command(|| CommandNode::literal("panel").execute(PanelExecutor));
    }
}
