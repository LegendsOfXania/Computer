use pumpkin_plugin_api::{
    command::{CommandError, CommandNode, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    common::NamedColor,
    forms::SimpleFormBuilder,
    java_dialog::{
        Action, ActionButton, AfterAction, Dialog, DialogBody, DialogType,
    },
    text::TextComponent,
    Result, Server,
};

use crate::{
    data,
    net::srv::{self, StartOutcome},
    util::text::ComputerMessage,
};

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

        match srv::ensure_started(&server, &conf.panel.ip, conf.panel.port) {
            Ok(StartOutcome::Started | StartOutcome::AlreadyRunning) => {}
            Err(err) => {
                return Err(CommandError::CommandFailed(
                    TextComponent::text(&format!(
                        "Could not start the panel server: {err}"
                    ))
                    .color_named(NamedColor::Red),
                ));
            }
        }

        let url = format!("http://{}:{}", conf.panel.ip, conf.panel.port);

        let panel_url = TextComponent::text("Open ")
            .add_child(
                TextComponent::text(&url)
                    .underlined(true)
                    .click_open_url(&url),
            )
            .add_child(TextComponent::text(" in your browser to connect."));

        let Some(player) = sender.as_player() else {
            sender.send_plugin_message(TextComponent::text(
                "The Computer panel is ready.",
            ));
            sender.send_plugin_message(panel_url);

            return Ok(0);
        };

        if let Some(java) = player.as_java() {
            let dialog = Dialog {
                title: TextComponent::text("Computer Panel")
                    .color_named(NamedColor::Gold),

                type_: DialogType::Confirmation,

                body: vec![
                    DialogBody::PlainMessage(TextComponent::text(
                        "The Computer panel is ready.",
                    )),
                    DialogBody::PlainMessage(TextComponent::text(
                        "Open the panel in your browser to manage the server.",
                    )),
                ],

                inputs: vec![],

                buttons: vec![ActionButton {
                    text: TextComponent::text("Open Panel")
                        .color_named(NamedColor::Gold),

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
            return Ok(0);
        }

        if let Some(bedrock) = player.as_bedrock() {
            let form = SimpleFormBuilder::new(
                TextComponent::text("Computer Panel")
                    .color_named(NamedColor::Gold),
                TextComponent::text(
                    "The Computer panel is ready. Open it in your browser to manage the server.",
                ),
            )
            .button(
                TextComponent::text("Open Panel")
                    .color_named(NamedColor::Gold),
                None,
            )
            .button(
                TextComponent::text("Close")
                    .color_named(NamedColor::Gray),
                None,
            )
            .build();

            bedrock.open_form(form);

            return Ok(0);
        }

        sender.send_plugin_message(TextComponent::text(
            "The Computer panel is ready.",
        ));
        sender.send_plugin_message(panel_url);

        Ok(0)
    }
}