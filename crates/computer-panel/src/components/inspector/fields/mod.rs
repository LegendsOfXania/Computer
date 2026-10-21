mod float;
mod int;
mod text;
mod toggle;

use computer_model::{field::FieldKind, key::EntryKey, value::Value};
use dioxus::prelude::*;

use crate::{
    components::inspector::fields::{float::Float, int::Int, text::Text, toggle::Toggle},
    state::app::AppState,
};

#[component]
pub fn Fields(entry: EntryKey) -> Element {
    let state = use_context::<AppState>();
    let entries = state.entries.read();
    let registry = state.registry.read();

    let Some(e) = entries.get(&entry) else {
        return rsx! {};
    };

    let Some(definition) = registry.get(&e.kind) else {
        return rsx! {};
    };

    rsx! {
        dl { class: "window-body window-fields",
            for f in definition.fields.iter() {
                dt { "{f.name}" }
                dd {
                    Field {
                        kind: f.kind.clone(),
                        value: e.fields.get(&f.name).cloned(),
                    }
                }
            }
        }
    }
}

#[component]
fn Field(kind: FieldKind, value: Option<Value>) -> Element {
    match (kind, value) {
        (FieldKind::String, Some(Value::String(v))) => rsx! { Text { value: v } },
        (FieldKind::Int, Some(Value::Int(v))) => rsx! { Int { value: v } },
        (FieldKind::Float, Some(Value::Float(v))) => rsx! { Float { value: v as f64 } },
        (FieldKind::Bool, Some(Value::Bool(v))) => rsx! { Toggle { value: v } },
        _ => rsx! { span { "…" } },
    }
}
