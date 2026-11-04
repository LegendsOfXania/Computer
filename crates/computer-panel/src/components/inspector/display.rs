use computer_model::{Entry, EntryDefinition, value::Value};

pub fn label(entry: &Entry) -> &str {
    match entry.fields.get("name") {
        Some(Value::String(name)) if !name.is_empty() => name,
        _ => &entry.kind,
    }
}

pub fn color(definition: &EntryDefinition) -> &str {
    if definition.color.is_empty() {
        "var(--surface-2)"
    } else {
        &definition.color
    }
}
