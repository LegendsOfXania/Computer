use computer_model::{
    field::{FieldDefinition, FieldKind},
    value::Value,
};
use dioxus::prelude::*;

use crate::components::inspector::fields::control::Control;

mod control;
mod fields;
mod float;
mod int;
mod list;
mod map;
mod reference;
mod text;
mod variant;

#[component]
pub fn Field(def: FieldDefinition, value: Value, onchange: EventHandler<Value>) -> Element {
    let optional = def.has_flag("optional") && !matches!(def.kind, FieldKind::Reference);

    let enabled = !optional || value != Value::Null;
    let label = def.name.replace('_', " ");
    let for_toggle = def.clone();

    rsx! {
        div { class: "field",
            if !label.is_empty() {
                div { class: "field-head",
                    span { class: "field-label", "{label}" }
                    if optional {
                        input {
                            class: "toggle",
                            r#type: "checkbox",
                            checked: enabled,
                            onchange: move |event: FormEvent| {
                                onchange.call(if event.checked() {
                                    for_toggle.default_value()
                                } else {
                                    Value::Null
                                });
                            },
                        }
                    }
                }
            }
            if enabled {
                Control { def: def.clone(), value, onchange }
            }
        }
    }
}
