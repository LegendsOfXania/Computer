use computer_model::{key::EntryKey, value::Value};
use dioxus::prelude::*;

use crate::state::AppState;

use super::layout::{NODE_H, NODE_W};

#[component]
pub fn EntryNode(
    entry_key: EntryKey,
    pos: (f64, f64),
    selected: bool,
    onselect: EventHandler<EntryKey>,
) -> Element {
    let app_state = use_context::<AppState>();
    let (x, y) = pos;

    let (name, kind) = app_state
        .entries
        .read()
        .get(&entry_key)
        .map(|entry| {
            let name = entry
                .fields
                .get("name")
                .and_then(|value| match value {
                    Value::String(name) => Some(name.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| entry_key.to_string());

            (name, entry.kind.to_string())
        })
        .unwrap_or_else(|| (entry_key.to_string(), String::new()));

    rsx! {
        div {
            class: if selected { "node selected" } else { "node" },
            style: "left: {x}px; top: {y}px; width: {NODE_W}px; height: {NODE_H}px;",

            onclick: move |e| {
                e.stop_propagation();
                onselect.call(entry_key);
            },

            span { class: "node-name", "{name}" }
            span { class: "node-kind", "{kind}" }
        }
    }
}