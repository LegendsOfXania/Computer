use computer_model::{
    id::{EntryId, Identifier},
    value::Value,
};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::{
    components::utils::icon::Icon,
    search::{ENTRY, NEW_ENTRY},
    state::{app::AppState, ui::UiState},
};

#[component]
pub fn ReferenceField(value: Value, onchange: EventHandler<Value>) -> Element {
    let (app, mut ui) = (use_context::<AppState>(), use_context::<UiState>());

    let pick = use_callback(move |id: EntryId| onchange.call(Value::Reference(id)));
    let open = move |_| ui.set_search(Some((ENTRY | NEW_ENTRY, pick)));

    let Value::Reference(id) = value else {
        return rsx! {
            div { class: "field-ref empty", onclick: open, {tid!("inspector-reference-none")} }
        };
    };

    let found = app.with_entry(&id, |entry| {
        let registry = app.registry.read();
        let definition = registry.get(&entry.kind);

        (
            entry.label().to_string(),
            entry.kind.clone(),
            definition
                .map(|definition| definition.color.clone())
                .unwrap_or_default(),
            definition
                .map(|definition| definition.icon.clone())
                .unwrap_or_default(),
        )
    });

    match found {
        Some((name, kind, color, icon)) => rsx! {
            div { class: "field-ref", style: "background: {color};", onclick: open,
                Icon { name: icon }
                div { class: "field-ref-text",
                    span { "{name}" }
                    small { "{kind}" }
                }
            }
        },
        None => {
            let id = id.into_string();

            rsx! {
                div { class: "field-ref missing", onclick: open,
                    div { class: "field-ref-text",
                        span { "{id}" }
                        small { {tid!("inspector-reference-missing")} }
                    }
                }
            }
        }
    }
}
