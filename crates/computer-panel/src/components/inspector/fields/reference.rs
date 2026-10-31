use computer_model::{Entry, value::Value};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::{components::utils::icon::Icon, state::app::AppState};

/// Affichage seul : la modification viendra avec la recherche.
#[component]
pub fn ReferenceField(value: Value) -> Element {
    let app = use_context::<AppState>();

    let Value::Reference(id) = value else {
        return rsx! {
            div { class: "field-ref empty", {tid!("inspector-reference-none")} }
        };
    };

    match app.with_entry(&id, |e| (label_of(e), e.kind.clone())) {
        Some((name, kind)) => {
            let definition = app.registry.read().get(&kind).cloned();

            let color = definition
                .as_ref()
                .map(|definition| definition.color.as_str())
                .filter(|color| !color.is_empty())
                .unwrap_or("var(--surface-2)")
                .to_string();

            let icon = definition
                .map(|definition| definition.icon)
                .unwrap_or_default();

            rsx! {
                div { class: "field-ref", style: "background: {color};",
                    Icon { name: icon }
                    div { class: "field-ref-text",
                        span { "{name}" }
                        small { "{kind}" }
                    }
                }
            }
        }
        None => {
            let id = id.as_str().to_string();

            rsx! {
                div { class: "field-ref missing",
                    div { class: "field-ref-text",
                        span { "{id}" }
                        small { {tid!("inspector-reference-missing")} }
                    }
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
