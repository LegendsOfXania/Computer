use computer_model::Registry;
use pumpkin_plugin_api::Result;

use crate::data;

pub fn handle(sender: String, registry: Registry) -> Result<()> {
    data::registry::set(sender.clone(), registry)?;

    tracing::info!("Got the entries of {sender}");

    Ok(())
}
