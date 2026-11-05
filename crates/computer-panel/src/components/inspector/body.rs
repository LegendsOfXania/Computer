use computer_model::{Entry, EntryDefinition, value::Value};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::components::{inspector::field::Field, utils::btn::Btn};

#[component]
pub fn Body(
    entry: Entry,
    definition: EntryDefinition,
    revision: u32,
    edit: Callback<(String, Value)>,
    close: EventHandler<()>,
) -> Element {
    let name = &entry.label().to_string();
    let id = entry.id.as_str().to_string();
    let color = definition.color;

    let rows = definition.fields.into_iter().map(|def| {
        let row_key = format!("{id}-{revision}-{}", def.name);
        let field_name = def.name.clone();
        let value = entry
            .fields
            .get(&def.name)
            .cloned()
            .unwrap_or_else(|| def.default_value());

        rsx! {
            Field {
                key: "{row_key}",
                def,
                value,
                onchange: move |value: Value| edit.call((field_name.clone(), value)),
            }
        }
    });

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

        div { class: "inspector-fields", {rows} }
    }
}
