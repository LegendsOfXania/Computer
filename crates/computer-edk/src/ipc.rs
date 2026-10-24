use std::{
    collections::HashSet,
    sync::{Mutex, MutexGuard},
};

use computer_model::{
    EntryDefinition, Registry,
    ipc::{ComputerMessage, ExtensionMessage, IpcMessage},
};
use pumpkin_plugin_api::ipc::send_ipc_message;

const COMPUTER: &str = "computer";

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

pub fn register(entries: &[fn() -> EntryDefinition]) -> Result<(), String> {
    if local().is_some() {
        return Err(
            "entries are already registered: list every entry in a single `register`".into(),
        );
    }

    let definitions: Vec<_> = entries.iter().map(|entry| entry()).collect();
    validate(&definitions)?;

    let registry = Registry::new(definitions);
    let bytes = ExtensionMessage::Registry(registry.clone()).encode()?;

    *local() = Some(registry);

    match send_ipc_message(COMPUTER, &bytes) {
        Ok(Ok(_)) => {}
        Ok(Err(reason)) => {
            *local() = None;

            return Err(format!("Computer rejected the registry: {reason}"));
        }
        Err(()) => {
            tracing::warn!("Computer is not loaded (yet): the entries will be sent when it asks")
        }
    }

    Ok(())
}

pub fn handle_ipc(sender: &str, message: &[u8]) -> Result<Vec<u8>, String> {
    if sender != COMPUTER {
        return Err(format!("unexpected sender `{sender}`"));
    }

    match ComputerMessage::decode(message)? {
        ComputerMessage::RequestRegistry => {
            let registry = local().clone().ok_or("no entries registered yet")?;

            ExtensionMessage::Registry(registry).encode()
        }
    }
}
