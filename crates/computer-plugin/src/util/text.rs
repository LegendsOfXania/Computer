use pumpkin_plugin_api::{
    command::CommandSender,
    common::NamedColor,
    text::TextComponent,
};

pub trait ComputerMessage {
    fn send_plugin_message(&self, msg: TextComponent);
}

impl ComputerMessage for CommandSender {
    fn send_plugin_message(&self, msg: TextComponent) {
        let prefix = TextComponent::text("ᴄᴏᴍᴘᴜᴛᴇʀ")
            .color_named(NamedColor::Gold);

        let separator = TextComponent::text(" ◼ ")
            .color_named(NamedColor::DarkGray);

        self.send_message(prefix.add_child(separator).add_child(msg));
    }
}