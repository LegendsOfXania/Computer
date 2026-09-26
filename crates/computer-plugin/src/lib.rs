mod data;
mod net;

use pumpkin_plugin_api::{Context, Plugin, PluginMetadata, Result};

struct ComputerPlugin;

impl Plugin for ComputerPlugin {
    fn new() -> Self {
        ComputerPlugin
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: env!("CARGO_PKG_NAME").into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: env!("CARGO_PKG_AUTHORS")
                .split(",")
                .map(|v| v.to_string())
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
        net::panel::ensure_panel_up_to_date();

        if let Err(err) = net::server::start(&context, 8080) {
            tracing::error!("Could not start the websocket server: {err}")
        }
        
        Ok(())
    }

    fn on_unload(&self, _context: Context) -> Result<()> {
        Ok(())
    }
}

pumpkin_plugin_api::register_plugin!(ComputerPlugin);