use computer_model::{key::EntryKey, value::Value};
use dioxus::prelude::*;

use crate::components::btn::Btn;
use crate::state::app::AppState;

#[component]
pub fn Window(
    entry: EntryKey,
    x: i32,
    y: i32,
    z: i32,
    #[props(default)] active: bool,
    on_close: EventHandler<()>,
) -> Element {
    let state = use_context::<AppState>();
    let entries = state.entries.read();
    let Some(e) = entries.get(&entry) else {
        return rsx! {};
    };

    let label = match e.fields.get("name") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => e.kind.clone(),
    };

    rsx! {
        div {
            class: if active { "window active" } else { "window" },
            style: "left: {x}px; top: {y}px; z-index: {z};",

            div { class: "window-bar",
                input { class: "window-title",
                    value: "{label}",
                    //todo check !empty => error toast
                }
                Btn {
                    onclick: move |_| on_close.call(())
                }
            }

            dl { class: "window-body window-fields",
                for (_, value) in e.fields.iter() {
                    dd { "{show(value)}" }
                }
            }
        }
    }
}

fn show(v: &Value) -> String {
    match v {
        Value::String(s) | Value::Enum(s) => s.clone(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Reference(k) => format!("{}:{}", k.0, k.1),
        _ => "…".into(),
    }
}
