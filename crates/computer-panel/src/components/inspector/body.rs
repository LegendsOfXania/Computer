use computer_model::{Entry, EntryDefinition, value::Value};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::components::{
    inspector::fields::Field,
    utils::{btn::Btn, icon::Icon},
};

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

    rsx! {
        div { class: "inspector-head",
            span { class: "inspector-icon", style: "background: {color};",
                Icon { name: definition.icon.clone() }
            }
            div { class: "inspector-title",
                p { class: "inspector-kind", "{entry.kind}" }
                p { class: "inspector-id", "{id}" }
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
