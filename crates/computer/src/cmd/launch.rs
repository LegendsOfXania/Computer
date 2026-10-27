use pumpkin_plugin_api::{
    Result, Server,
    command::{CommandError, CommandNode, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    common::NamedColor,
    forms::SimpleFormBuilder,
    java_dialog::{Action, ActionButton, AfterAction, Dialog, DialogBody, DialogType},
    text::TextComponent,
};

use crate::{data, net::srv, util::text::ComputerMessage};

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
        let conf = data::conf::get_conf();

        if !conf.panel.enabled {
            sender.send_plugin_message(
                TextComponent::text(
                    "The panel is disabled. Contact an administrator to enable it in the configuration file.",
                )
                .color_named(NamedColor::Red),
            );

            return Ok(0);
        }

        let token = match srv::ensure_started(&server, &conf.panel.ip, conf.panel.port) {
            Ok((_, token)) => token,

            Err(error) => {
                return Err(CommandError::CommandFailed(
                    TextComponent::text(&format!("Could not start the panel server: {error}"))
                        .color_named(NamedColor::Red),
                ));
            }
        };

        let url = format!("http://{}:{}/#t={}", conf.panel.ip, conf.panel.port, token,);

        let panel_url = TextComponent::text("Open ")
            .add_child(
                TextComponent::text(&url)
                    .underlined(true)
                    .click_open_url(&url),
            )
            .add_child(TextComponent::text(" in your browser to connect."));

        let Some(player) = sender.as_player() else {
            sender.send_plugin_message(TextComponent::text("The Computer panel is ready."));
            sender.send_plugin_message(panel_url);

            register_session(token, sender)?;
            return Ok(0);
        };

        if let Some(java) = player.as_java() {
            let dialog = Dialog {
                title: TextComponent::text("Computer Panel").color_named(NamedColor::Gold),
                type_: DialogType::Confirmation,
                body: vec![
                    DialogBody::PlainMessage(TextComponent::text("The Computer panel is ready.")),
                    DialogBody::PlainMessage(TextComponent::text(
                        "Open the panel in your browser to manage the server.",
                    )),
                ],
                inputs: vec![],
                buttons: vec![ActionButton {
                    text: TextComponent::text("Open Panel").color_named(NamedColor::Gold),
                    tooltip: Some(TextComponent::text(
                        "Open the Computer panel in your browser",
                    )),
                    width: None,
                    action: Action::OpenUrl(url),
                }],
                links: vec![],
                after_action: Some(AfterAction::Pop),
                can_close_with_escape: true,
                external_title: None,
            };

            java.show_dialog(dialog);
            register_session(token, sender)?;
            return Ok(0);
        }

        if let Some(bedrock) = player.as_bedrock() {
            let form = SimpleFormBuilder::new(
                TextComponent::text("Computer Panel").color_named(NamedColor::Gold),
                TextComponent::text(
                    "The Computer panel is ready. Open it in your browser to manage the server.",
                ),
            )
            .button(
                TextComponent::text("Open Panel").color_named(NamedColor::Gold),
                None,
            )
            .button(
                TextComponent::text("Close").color_named(NamedColor::Gray),
                None,
            )
            .build();

            bedrock.open_form(form);
            sender.send_plugin_message(panel_url);

            register_session(token, sender)?;
            return Ok(0);
        }

        sender.send_plugin_message(TextComponent::text("The Computer panel is ready."));
        sender.send_plugin_message(panel_url);

        register_session(token, sender)?;

        Ok(0)
    }
}

fn register_session(token: String, sender: CommandSender) -> Result<(), CommandError> {
    match srv::register_session(token, sender) {
        Ok(()) => Ok(()),

        Err((error, _sender)) => Err(CommandError::CommandFailed(
            TextComponent::text(&format!("Could not create the panel session: {error}"))
                .color_named(NamedColor::Red),
        )),
    }
}
