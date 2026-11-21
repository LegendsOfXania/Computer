use computer_model::{Field as Definition, FieldKind, Value};
use dioxus::prelude::*;

use crate::components::inspector::field::{
    control::control, default::defaults, input::ToggleField,
};

mod composite;
mod control;
mod default;
mod frame;
mod input;

#[component]
pub fn Field(
    def: Vec<Definition>,
    values: Vec<Value>,
    onchange: EventHandler<Vec<Value>>,
) -> Element {
    let Some(field) = def.first() else {
        return rsx! {};
    };

    let optional = field.has_flag("optional") && !matches!(field.kind, FieldKind::Reference);
    let composite = matches!(
        field.kind,
        FieldKind::Struct(_) | FieldKind::List | FieldKind::Map | FieldKind::Variant(_)
    );

    let enabled = !optional || !matches!(values.first(), None | Some(Value::Null));

    let default = optional.then(|| defaults(&def));

    let values = if values.is_empty() {
        defaults(&def)
    } else {
        values
    };

    let label = field.name.replace('_', " ");
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
                                    _ => vec![Value::Null],
                                });
                            },
                        }
                    }
                }
            }
            if enabled {
                {control(&def, values, onchange, title)}
            }
        }
    }
}
