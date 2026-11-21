use computer_model::{Entry, EntryDefinition, Value};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::components::{
    inspector::{OPERATIONS, field::Field, operations::Operations},
    utils::btn::Btn,
};

#[component]
pub fn Body(
    entry: Entry,
    definition: EntryDefinition,
    revision: u32,
    edit: Callback<(String, Vec<Value>)>,
    close: EventHandler<()>,
) -> Element {
    let name = entry.label().to_string();
    let id = entry.id().as_str().to_string();
    let color = definition.color.clone();

    let rows = definition.members().map(|node| {
        let field_name = node
            .value()
            .map(|field| field.name.clone())
            .unwrap_or_default();
        let row_key = format!("{id}-{revision}-{field_name}");
        let def = node.subtree().to_vec();
        let values = entry
            .get(&field_name)
            .map(|value| value.subtree().to_vec())
            .unwrap_or_default();

        rsx! {
            Field {
                key: "{row_key}",
                def,
                values,
                onchange: move |subtree: Vec<Value>| edit.call((field_name.clone(), subtree)),
            }
        }
    });

    rsx! {
        div { class: "inspector-head",
            div { class: "inspector-title",
                p {
                    class: "inspector-name",
                    style: "color: {color};",
                    "{name}"
                }
                div { class: "inspector-meta",
                    span {
                        class: "inspector-kind",
                        style: "color: {color};",
                        "{entry.kind}"
                    }
                    span {
                        class: "inspector-id",
                        "{id}"
                    }
                }
            }

            Btn {
                icon: "close",
                tooltip: tid!("inspector-close"),
                onclick: move |_| close.call(()),
            }

            div { class: "inspector-menu",
                Btn {
                    icon: "menu",
                    tooltip: tid!("inspector-operations"),
                    class: "inspector-menu-trigger",
                    popover_target: Some(OPERATIONS.to_string()),
                }

                div {
                    id: OPERATIONS,
                    class: "inspector-operations-popup",
                    "popover": "auto",

                    Operations {
                        entry: entry.clone(),
                        definition: definition.clone(),
                    }
                }
            }
        }

        div { class: "inspector-fields", {rows} }
    }
}
