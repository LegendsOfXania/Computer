use std::{collections::HashSet, sync::Mutex};

use computer_model::{EntryDefinition, Registry, ipc::ExtensionMessage};

static LOCAL: Mutex<Option<Registry>> = Mutex::new(None);

pub static ACTIVE_KINDS: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn register_entries(entries: &[fn() -> EntryDefinition]) -> Result<(), String> {
    let definitions: Vec<_> = entries.iter().map(|entry| entry()).collect();
    let mut kinds = HashSet::new();

    for definition in &definitions {
        if !kinds.insert(definition.kind.as_str()) {
            return Err(format!(
                "Entry kind `{}` is registered more than once",
                definition.kind
            ));
        }

        let mut fields = HashSet::new();

        for field in &definition.fields {
            if !fields.insert(field.name.as_str()) {
                return Err(format!(
                    "Entry `{}` declares field `{}` more than once",
                    definition.kind, field.name
                ));
            }
        }
    }

    let mut registry = LOCAL
        .lock()
        .map_err(|_| "Entry registry lock poisoned".to_owned())?;

    if registry.is_some() {
        return Err("Entries have already been registered".into());
    }

    *registry = Some(Registry::new(definitions));

    Ok(())
}

pub fn answer() -> Result<ExtensionMessage, String> {
    let registry = LOCAL
        .lock()
        .map_err(|_| "Entry registry lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "No entries have been registered".to_owned())?;

    Ok(ExtensionMessage::Registry(registry))
}
