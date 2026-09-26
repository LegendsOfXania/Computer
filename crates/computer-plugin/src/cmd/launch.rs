use pumpkin_plugin_api::{
    Result, Server,
    command::{CommandError, CommandNode, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    text::TextComponent,
};

use crate::{data, net::server::{self, StartOutcome}};

pub fn node() -> CommandNode {
    CommandNode::literal("launch").execute(Launch)
}

struct Launch;

impl CommandHandler for Launch {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let Some(config) = data::conf::get() else {
            return Err(CommandError::CommandFailed(
                TextComponent::text("Failed to acces to the plugin's configuration."),
            ));
        };

        if !config.panel.enabled {
            sender.send_message(TextComponent::text("Panel is disabled in the configuration. Contact an administrator to enabled it."));
            return Ok(0);
        }

        match server::ensure_started(&server, &config.panel.ip, config.panel.port) {
            Ok(StartOutcome::Started) => {
                sender.send_message(TextComponent::text(&format!(
                    "Panel server started on {}:{}.",
                    config.panel.ip, config.panel.port
                )));
            }
            Ok(StartOutcome::AlreadyRunning) => {
                sender.send_message(TextComponent::text("Panel server is already running."));
            }
            Err(err) => {
                return Err(CommandError::CommandFailed(TextComponent::text(
                    &format!("Could not start the panel server: {err}"),
                )));
            }
        }

        Ok(0)
    }
}