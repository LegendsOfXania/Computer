use computer_model::Registry;
use pumpkin_plugin_api::Result;

use crate::data;

pub fn handle(sender: String, registry: Registry) -> Result<()> {
    match data::registry::extend(registry) {
        Ok(()) => tracing::info!("Registered extension from {sender}"),
        Err(err) => tracing::error!("Could not register extension from {sender}: {err}"),
    }

    Ok(())
}