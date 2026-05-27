use computer_api::command::{ComputerCommand, register_command};
use pumpkin_plugin_api::{
    Result, Server,
    command::{CommandError, CommandNode, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    java_dialog::{ActionButton, DialogBody, DialogType},
    java_dialogs::{Action, Dialog},
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

        let token = server::generate_token();
        let url = format!("http://{}/?token={}", config.panel_addr(), token);

        if let Some(java) = sender.as_player().and_then(|player| player.as_java()) {
            let dialog = Dialog {
                title: TextComponent::text("Computer Panel"),
                type_: DialogType::MultiAction,
                body: vec![
                    DialogBody::PlainMessage(TextComponent::text(
                        "You are trying to open the Computer Panel. Click on the link below to access it.",
                    )),
                    DialogBody::PlainMessage(TextComponent::text(
                        "The link will expire in 1 minute.",
                    )),
                ],
                inputs: vec![],
                buttons: vec![ActionButton {
                    text: TextComponent::text("Open Panel"),
                    tooltip: None,
                    width: None,
                    action: Action::OpenUrl(url),
                }],
                links: vec![],
                after_action: None,
                can_close_with_escape: true,
                external_title: None,
            };

            java.show_dialog(dialog);
            Ok(1)
        } else {
            let msg = TextComponent::text(
                "You are trying to open the Computer Panel. Click on the link below to access it. ",
            );
            let expiry_msg = TextComponent::text("The link will expire in 1 minute. ");
            expiry_msg.color_named(pumpkin_plugin_api::common::NamedColor::Gold);
            msg.add_child(expiry_msg);

            let link_msg = TextComponent::text(&url);
            link_msg.click_open_url(&url);
            msg.add_child(link_msg);

            sender.send_message(msg);
            Ok(1)
        }
    }
}

impl ComputerCommand for PanelCommand {
    fn register() {
        register_command(|| CommandNode::literal("panel").execute(PanelCommand));
        register_command(|| CommandNode::literal("p").execute(PanelCommand));
    }
}
