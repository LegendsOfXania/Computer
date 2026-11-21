use computer_model::{Value, id::EntryId};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::{
    components::utils::icon::Icon,
    search::{ENTRY, NEW_ENTRY},
    state::{
        app::AppState,
        ui::{Picked, SearchRequest, UiState},
    },
};

#[component]
pub fn ReferenceField(value: Value, onchange: EventHandler<Value>) -> Element {
    let (app, mut ui) = (use_context::<AppState>(), use_context::<UiState>());

    let pick = use_callback(move |picked: Picked| {
        if let Picked::Entry(id) = picked {
            onchange.call(Value::from(id));
        }
    });

    let open = move |_| {
        ui.set_search(Some(SearchRequest {
            scope: ENTRY | NEW_ENTRY,
            accepts: None,
            pick,
        }));
    };

    let Value::Reference(id) = value else {
        return rsx! {
            div { class: "field-ref empty", onclick: open, {tid!("inspector-reference-none")} }
        };
    };

    let id = EntryId::from(id.as_str());

    let Some((name, kind)) =
        app.with_entry(id, |entry| (entry.label().to_string(), entry.kind.clone()))
    else {
        return rsx! {
            div { class: "field-ref missing", onclick: open,
                div { class: "field-ref-text",
                    span { "{id}" }
                    small { {tid!("inspector-reference-missing")} }
                }
            }
        };
    };

    let (icon, color) = app.look_of(&kind);

    rsx! {
        div { class: "field-ref", style: "background: {color};", onclick: open,
            Icon { name: icon }
            div { class: "field-ref-text",
                span { "{name}" }
                small { "{kind}" }
            }
        }
    }
}
