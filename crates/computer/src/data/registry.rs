use std::collections::HashSet;

use computer_model::Registry;
use parking_lot::Mutex;

static REGISTRY: Mutex<Registry> = Mutex::new(Registry(Vec::new()));

pub fn get() -> Registry {
    REGISTRY.lock().clone()
}

pub fn extend(registry: Registry) -> Result<(), String> {
    let mut current = REGISTRY.lock();

    let mut seen: HashSet<&str> = current.0.iter().map(|d| d.kind.as_str()).collect();

    let duplicates: Vec<&str> = registry
        .0
        .iter()
        .map(|d| d.kind.as_str())
        .filter(|kind| !seen.insert(kind))
        .collect();

    if !duplicates.is_empty() {
        return Err(format!("duplicate kinds: {}", duplicates.join(", ")));
    }

    current.extend(registry);

    Ok(())
}