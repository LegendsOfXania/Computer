use computer_model::{Entry, EntryDefinition, value::Value};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::components::{inspector::fields::Field, utils::btn::Btn};

#[component]
pub fn Body(
    entry: Entry,
    definition: EntryDefinition,
    edit: Callback<(String, Value)>,
    close: EventHandler<()>,
) -> Element {
    let color = Some(definition.color.as_str())
        .filter(|color| !color.is_empty())
        .unwrap_or("var(--surface-2)")
        .to_string();

    let id = entry.id.as_str().to_string();

    let name = entry
        .fields
        .get("name")
        .and_then(|value| match value {
            Value::String(name) if !name.is_empty() => Some(name.as_str()),
            _ => None,
        })
        .unwrap_or(&entry.kind);

    rsx! {
        div { class: "inspector-head",
            div { class: "inspector-title",
                p { class: "inspector-name", style: "color: {color};", "{name}" }
                div { class: "inspector-meta",
                    span { class: "inspector-kind", style: "color: {color};", "{entry.kind}" }
                    span { class: "inspector-id", "{id}" }
                }
            }

            Btn {
                icon: "close",
                tooltip: tid!("inspector-close"),
                onclick: move |_| close.call(()),
            }
        }

        div { class: "inspector-fields",
            for field in definition.fields.iter() {
                Field {
                    key: "{id}-{field.name}",
                    def: field.clone(),
                    value: entry.fields.get(&field.name).cloned().unwrap_or_else(|| field.default_value()),
                    onchange: {
                        let name = field.name.clone();
                        move |value: Value| edit.call((name.clone(), value))
                    },
                }
            }
        }
    }
}
