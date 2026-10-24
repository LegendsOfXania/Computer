use std::collections::HashSet;

use computer_model::{EntryDefinition, Registry};
use parking_lot::Mutex;

static REGISTRY: Mutex<Vec<(String, Vec<EntryDefinition>)>> = Mutex::new(Vec::new());

pub fn get() -> Registry {
    Registry::new(
        REGISTRY
            .lock()
            .iter()
            .flat_map(|(_, definitions)| definitions.clone())
            .collect(),
    )
}

pub fn set(sender: String, registry: Registry) -> Result<(), String> {
    let mut batch = HashSet::new();

    if let Some(definition) = registry.0.iter().find(|d| !batch.insert(d.kind.as_str())) {
        return Err(format!("kind registered twice: {}", definition.kind));
    }

    let mut current = REGISTRY.lock();

    let taken: HashSet<&str> = current
        .iter()
        .filter(|(owner, _)| *owner != sender)
        .flat_map(|(_, definitions)| definitions.iter().map(|d| d.kind.as_str()))
        .collect();

    let duplicates: Vec<&str> = registry
        .0
        .iter()
        .map(|d| d.kind.as_str())
        .filter(|kind| taken.contains(kind))
        .collect();

    if !duplicates.is_empty() {
        return Err(format!(
            "kinds already registered by another extension: {}",
            duplicates.join(", ")
        ));
    }

    current.retain(|(owner, _)| *owner != sender);
    current.push((sender, registry.0));

    Ok(())
}
