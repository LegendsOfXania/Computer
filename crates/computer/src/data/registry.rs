use std::sync::Mutex;

use computer_model::Registry;

static REGISTRY: Mutex<Registry> = Mutex::new(Registry(Vec::new()));

pub fn get() -> Result<Registry, String> {
    REGISTRY
        .lock()
        .map_err(|error| error.to_string())
        .map(|registry| registry.clone())
}

pub fn extend(registry: Registry) -> Result<(), String> {
    REGISTRY
        .lock()
        .map_err(|error| error.to_string())?
        .extend(registry);

    Ok(())
}