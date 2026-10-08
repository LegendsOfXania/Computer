mod cmd;
mod data;
mod ipc;
mod net;
mod util;

use pumpkin_plugin_api::{Context, Plugin, PluginMetadata, Result};

pub struct ComputerPlugin;

impl Plugin for ComputerPlugin {
    fn new() -> Self {
        Self
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: env!("CARGO_PKG_NAME").into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: env!("CARGO_PKG_AUTHORS")
                .split(',')
                .map(str::to_string)
                .collect(),
            description: env!("CARGO_PKG_DESCRIPTION").into(),
            dependencies: vec![],
            permissions: vec![
                "fs.read.data".into(),
                "fs.write.data".into(),
                "network.dns".into(),
                "network.outbound".into(),
                "network.tcp".into(),
                "network.tcp.bind".into(),
                "network.tcp.connect".into(),
                "http.outbound".into(),
            ],
        }
    }

    fn on_load(&self, context: Context) -> Result<()> {
        data::init_data_folder(context.get_data_folder());
        data::library::init_live();

        if data::conf::get_conf().panel.enabled {
            net::panel::ensure_panel_up_to_date();
        } else {
            tracing::info!("Panel is disable in the configuration. No check for update.")
        }


        cmd::register(&context);

        Ok(())
    }

    fn on_unload(&self, _context: Context) -> Result<()> {
        data::library::flush_dirty();

        Ok(())
    }

    fn handle_ipc_message(
        &self,
        sender: String,
        message: Vec<u8>,
    ) -> Result<Vec<u8>> {
        ipc::handle(sender, message)
    }
}

pumpkin_plugin_api::register_plugin!(ComputerPlugin);