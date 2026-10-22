mod float;
mod int;
mod text;
mod toggle;

use crate::{
    components::{
        inspector::fields::{float::Float, int::Int, text::Text, toggle::Toggle},
        utils::toast::ERR,
    },
    state::app::AppState,
    ws::Client,
};
use computer_model::{field::FieldKind, key::EntryKey, value::Value};
use dioxus::prelude::*;
use dioxus_i18n::tid;
use dioxus_primitives::toast::{ToastOptions, use_toast};

#[component]
pub fn Fields(entry: EntryKey) -> Element {
    let state = use_context::<AppState>();
    let client = use_context::<Client>();
    let toast = use_toast();

    let entries = state.entries.read();
    let registry = state.registry.read();

    let Some(e) = entries.get(&entry) else {
        return rsx! {};
    };
    let Some(def) = registry.get(&e.kind) else {
        return rsx! {};
    };

    rsx! {
        dl { class: "window-body window-fields",
            for f in def.fields.iter() {
                dt { "{f.name}" }
                dd {
                    Field {
                        kind: f.kind.clone(),
                        value: e.fields.get(&f.name).cloned(),
                        onchange: {
                            let name = f.name.clone();
                            let client = client.clone();
                            move |value: Value| {
                                let name = name.clone();
                                let client = client.clone();
                                spawn(async move {
                                    let result = client
                                        .edit_entry(&state, entry, |e| {
                                            e.fields.insert(name, value);
                                        })
                                        .await;
                                    if let Err(err) = result { toast.error(
                                        tid!("toast-push-failed"),
                                        ToastOptions::new().description(err).duration(ERR),
                                    )}
                                });
                            }
                        },
                    }
                }
            }
        }
    }
}

fn blank(kind: &FieldKind) -> Option<Value> {
    match kind {
        FieldKind::String => Some(Value::String(String::new())),
        FieldKind::Int => Some(Value::Int(0)),
        FieldKind::Float => Some(Value::Float(0.0)),
        FieldKind::Bool => Some(Value::Bool(false)),
        _ => None,
    }
}

#[component]
fn Field(kind: FieldKind, value: Option<Value>, onchange: EventHandler<Value>) -> Element {
    let value = value.or_else(|| blank(&kind));

    match (kind, value) {
        (FieldKind::String, Some(Value::String(v))) => rsx! {
            Text { value: v, onchange: move |v| onchange.call(Value::String(v)) }
        },
        (FieldKind::Int, Some(Value::Int(v))) => rsx! {
            Int { value: v, onchange: move |v| onchange.call(Value::Int(v)) }
        },
        (FieldKind::Float, Some(Value::Float(v))) => rsx! {
            Float { value: v as f64, onchange: move |v| onchange.call(Value::Float(v as _)) }
        },
        (FieldKind::Bool, Some(Value::Bool(v))) => rsx! {
            Toggle { value: v, onchange: move |v| onchange.call(Value::Bool(v)) }
        },
        _ => rsx! { span { "…" } },
    }
}
