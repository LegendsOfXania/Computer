use pumpkin_plugin_api::{
    Context, Plugin, PluginMetadata,
    permissions::{FS_WRITE_DATA, NETWORK_OUTBOUND, NETWORK_TCP, NETWORK_TCP_BIND},
};
use tracing::*;

struct ComputerPlugin;

impl Plugin for ComputerPlugin {
    fn new() -> Self {
        ComputerPlugin
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Computer".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["Legends of Xania".into()],
            description: "A new page for player interactions".into(),
            dependencies: vec![],
            permissions: vec![
                NETWORK_TCP.into(),
                NETWORK_TCP_BIND.into(),
                NETWORK_OUTBOUND.into(),
                FS_WRITE_DATA.into(),
            ],
        }
    }

    fn on_load(&mut self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        info!("Hello from the example plugin!");
        Ok(())
    }

    fn on_unload(&mut self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        info!("Example plugin unloaded. Goodbye!");
        Ok(())
    }
}

pumpkin_plugin_api::register_plugin!(ComputerPlugin);
