use computer_model::{Entry, value::Value};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::state::app::AppState;

#[component]
pub fn ReferenceField(value: Value) -> Element {
    let app = use_context::<AppState>();

    let Value::Reference(id) = value else {
        return rsx! {
            div { class: "field-ref empty", {tid!("inspector-reference-none")} }
        };
    };

    match app.with_entry(&id, |e| (label_of(e), e.kind.clone())) {
        Some((name, kind)) => rsx! {
            div { class: "field-ref",
                span { "{name}" }
                small { "{kind}" }
            }
        },
        None => {
            let id = id.as_str().to_string();

            rsx! {
                div { class: "field-ref missing",
                    span { "{id}" }
                    small { {tid!("inspector-reference-missing")} }
                }
            }
        }
    }
}

fn label_of(entry: &Entry) -> String {
    match entry.fields.get("name") {
        Some(Value::String(name)) if !name.is_empty() => name.clone(),
        _ => entry.kind.clone(),
    }
}
