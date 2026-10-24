use std::{
    collections::HashSet,
    sync::{Mutex, MutexGuard},
};

use computer_model::{EntryDefinition, Registry, ipc::ExtensionMessage};

static LOCAL: Mutex<Option<Registry>> = Mutex::new(None);

fn local() -> MutexGuard<'static, Option<Registry>> {
    LOCAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn validate(definitions: &[EntryDefinition]) -> Result<(), String> {
    let mut kinds = HashSet::new();

    for definition in definitions {
        if !kinds.insert(definition.kind.as_str()) {
            return Err(format!("entry `{}` is registered twice", definition.kind));
        }

        let mut names = HashSet::new();

        if let Some(field) = definition
            .fields
            .iter()
            .find(|field| !names.insert(field.name.as_str()))
        {
            return Err(format!(
                "entry `{}` has the field `{}` twice (inherited from its base?)",
                definition.kind, field.name
            ));
        }
    }

    Ok(())
}

pub fn register_entries(entries: &[fn() -> EntryDefinition]) -> Result<(), String> {
    if local().is_some() {
        return Err(
            "entries are already registered: list every entry in a single `register`".into(),
        );
    }

    let definitions: Vec<_> = entries.iter().map(|entry| entry()).collect();
    validate(&definitions)?;

    *local() = Some(Registry::new(definitions));

    Ok(())
}

pub fn answer() -> Result<ExtensionMessage, String> {
    let registry = local().clone().ok_or("no entries registered yet")?;

    Ok(ExtensionMessage::Registry(registry))
}
