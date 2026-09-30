use computer_model::Registry;
use pumpkin_plugin_api::Result;

use crate::data;

pub fn handle(sender: String, registry: Registry) -> Result<()> {
    data::registry::extend(registry)?;

    tracing::info!("Registered extension from {}", sender);

    Ok(())
}