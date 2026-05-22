use std::path::Path;

use pumpkin_plugin_api::{
    Context, Plugin, PluginMetadata,
    permission::{Permission, PermissionDefault},
    permissions::{FS_WRITE_DATA, NETWORK_OUTBOUND, NETWORK_TCP, NETWORK_TCP_BIND},
};
use tracing::*;

use crate::{command::init_command_tree, config::ComputerConfig};

mod command;
mod config;
mod files;
mod server;

struct ComputerPlugin;

impl Plugin for ComputerPlugin {
    fn new() -> Self {
        ComputerPlugin
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "computer".into(),
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

    fn on_load(&mut self, context: Context) -> pumpkin_plugin_api::Result<()> {
        ComputerConfig::init(&Path::new(&context.get_data_folder()));

        command::register_internal();

        context.register_permission(&Permission {
            node: "computer:computer".to_string(),
            description: "Access to the /computer command".to_string(),
            default: PermissionDefault::Allow,
            children: Vec::new(),
        })?;

        context.register_command(init_command_tree(), "computer:computer");

        Ok(())
    }

    fn on_unload(&mut self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        info!("Example plugin unloaded. Goodbye!");
        Ok(())
    }
}

pumpkin_plugin_api::register_plugin!(ComputerPlugin);
