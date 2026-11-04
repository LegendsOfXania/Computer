use computer_model::{
    field::{FieldDefinition, FieldKind},
    value::Value,
};
use dioxus::prelude::*;

use crate::components::inspector::field::{control::control, input::ToggleField};

mod composite;
mod control;
mod frame;
mod input;
mod ops;

#[component]
pub fn Field(def: FieldDefinition, value: Value, onchange: EventHandler<Value>) -> Element {
    let optional = def.has_flag("optional") && !matches!(def.kind, FieldKind::Reference);
    let composite = matches!(
        def.kind,
        FieldKind::Struct(_) | FieldKind::List(_) | FieldKind::Map { .. } | FieldKind::Variant(_)
    );

    let enabled = !optional || value != Value::Null;

    let default = optional.then(|| def.default_value());

    let label = def.name.replace('_', " ");
    let (head, title) = if composite && !optional {
        (String::new(), label)
    } else {
        (label, String::new())
    };

    rsx! {
        div { class: "field",
            if !head.is_empty() {
                div { class: "field-head",
                    span { class: "field-label", "{head}" }
                    if optional {
                        ToggleField {
                            value: enabled,
                            onchange: move |on: bool| {
                                onchange.call(match (&default, on) {
                                    (Some(default), true) => default.clone(),
                                    _ => Value::Null,
                                });
                            },
                        }
                    }
                }
            }
            if enabled {
                {control(&def, value, onchange, title)}
            }
        }
    }
}
