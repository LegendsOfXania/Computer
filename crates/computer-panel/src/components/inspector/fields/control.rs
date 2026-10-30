use computer_model::{
    field::{FieldDefinition, FieldKind},
    value::Value,
};
use dioxus::prelude::*;
use indexmap::IndexMap;

use crate::components::inspector::fields::{
    fields::Fields, float::FloatField, int::IntField, list::ListField, map::MapEditor,
    reference::ReferenceField, text::TextField, variant::VariantEditor,
};

#[component]
pub fn Control(def: FieldDefinition, value: Value, onchange: EventHandler<Value>) -> Element {
    match &def.kind {
        FieldKind::String => {
            let text = match value {
                Value::String(text) => text,
                _ => String::new(),
            };

            rsx! {
                TextField {
                    value: text,
                    multiline: def.has_flag("multiline"),
                    onchange: move |text: String| onchange.call(Value::String(text)),
                }
            }
        }

        FieldKind::Int => {
            let number = match value {
                Value::Int(number) => number,
                _ => 0,
            };

            rsx! {
                IntField {
                    value: number,
                    min: def.bound("min"),
                    max: def.bound("max"),
                    onchange: move |number: i64| onchange.call(Value::Int(number)),
                }
            }
        }

        FieldKind::Float => {
            let number = match value {
                Value::Float(number) => number,
                _ => 0.0,
            };

            rsx! {
                FloatField {
                    value: number,
                    min: def.bound("min"),
                    max: def.bound("max"),
                    onchange: move |number: f64| onchange.call(Value::Float(number)),
                }
            }
        }

        FieldKind::Bool => rsx! {
            input {
                class: "toggle",
                r#type: "checkbox",
                checked: matches!(value, Value::Bool(true)),
                onchange: move |event: FormEvent| onchange.call(Value::Bool(event.checked())),
            }
        },

        FieldKind::Enum(options) => {
            let current = match value {
                Value::Enum(name) => name,
                _ => options.first().cloned().unwrap_or_default(),
            };

            rsx! {
                select {
                    class: "field-input",
                    onchange: move |event: FormEvent| onchange.call(Value::Enum(event.value())),
                    for option in options.iter() {
                        option {
                            key: "{option}",
                            value: "{option}",
                            selected: *option == current,
                            "{option}"
                        }
                    }
                }
            }
        }

        FieldKind::Reference => rsx! {
            ReferenceField { value }
        },

        FieldKind::Struct(fields) => {
            let values = match value {
                Value::Struct(values) => values,
                _ => IndexMap::new(),
            };

            rsx! {
                Fields {
                    fields: fields.clone(),
                    values,
                    onchange: move |values: IndexMap<String, Value>| {
                        onchange.call(Value::Struct(values));
                    },
                }
            }
        }

        FieldKind::List(inner) => {
            let items = match value {
                Value::List(items) => items,
                _ => Vec::new(),
            };

            rsx! {
                ListField {
                    inner: (**inner).clone(),
                    items,
                    onchange: move |items: Vec<Value>| onchange.call(Value::List(items)),
                }
            }
        }

        FieldKind::Map {
            key: map_key,
            value: map_value,
        } => {
            let pairs = match value {
                Value::Map(pairs) => pairs,
                _ => Vec::new(),
            };

            rsx! {
                MapEditor {
                    key_kind: (**map_key).clone(),
                    value_kind: (**map_value).clone(),
                    pairs,
                    onchange: move |pairs: Vec<(Value, Value)>| onchange.call(Value::Map(pairs)),
                }
            }
        }

        FieldKind::Variant(variants) => rsx! {
            VariantEditor { variants: variants.clone(), value, onchange }
        },
    }
}
