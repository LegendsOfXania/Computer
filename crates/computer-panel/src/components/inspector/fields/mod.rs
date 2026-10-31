use computer_model::{
    field::{FieldDefinition, FieldKind},
    value::Value,
};
use dioxus::prelude::*;

use crate::components::inspector::fields::control::Control;

mod card;
mod collapse;
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
    let composite = matches!(
        def.kind,
        FieldKind::Struct(_) | FieldKind::List(_) | FieldKind::Map { .. }
    );

    let enabled = !optional || value != Value::Null;
    let label = def.name.replace('_', " ");
    let for_toggle = def.clone();

    let show_head = !label.is_empty() && (!composite || optional);
    let title = if composite && !optional {
        label.clone()
    } else {
        String::new()
    };

    rsx! {
        div { class: "field",
            if show_head {
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
                Control { def: def.clone(), value, onchange, title }
            }
        }
    }
}
