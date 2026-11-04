use computer_model::value::Value;
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::{
    components::{inspector::display, utils::icon::Icon},
    state::app::AppState,
};

#[component]
pub fn ReferenceField(value: Value) -> Element {
    let app = use_context::<AppState>();

    let Value::Reference(id) = value else {
        return rsx! {
            div { class: "field-ref empty", {tid!("inspector-reference-none")} }
        };
    };

    let found = app.with_entry(&id, |entry| {
        let registry = app.registry.read();
        let definition = registry.get(&entry.kind);

        (
            display::label(entry).to_string(),
            entry.kind.clone(),
            definition
                .map_or("var(--surface-2)", display::color)
                .to_string(),
            definition
                .map(|definition| definition.icon.clone())
                .unwrap_or_default(),
        )
    });

    match found {
        Some((name, kind, color, icon)) => rsx! {
            div { class: "field-ref", style: "background: {color};",
                Icon { name: icon }
                div { class: "field-ref-text",
                    span { "{name}" }
                    small { "{kind}" }
                }
            }
        },
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
