use computer_model::{Field as Definition, FieldKind, FieldNode, Node, Value};
use dioxus::prelude::*;

use crate::components::inspector::field::{
    composite::{ListEditor, MapEditor, StructEditor, VariantEditor},
    input::{EnumField, NumberField, ReferenceField, TextField, ToggleField},
};

pub fn control(
    def: &[Definition],
    values: Vec<Value>,
    onchange: EventHandler<Vec<Value>>,
    title: String,
) -> Element {
    let Some(field) = FieldNode::new(def, 0).value() else {
        return rsx! {};
    };

    let value = Node::new(&values, 0);
    let single = move |value: Value| onchange.call(vec![value]);

    match &field.kind {
        FieldKind::String => rsx! {
            TextField {
                value: String::try_from(value).unwrap_or_default(),
                multiline: field.has_flag("multiline"),
                onchange: move |text: String| single(Value::String(text)),
            }
        },

        FieldKind::Int => rsx! {
            NumberField::<i64> {
                value: i64::try_from(value).unwrap_or_default(),
                min: field.bound("min"),
                max: field.bound("max"),
                onchange: move |number: i64| single(Value::Int(number)),
            }
        },

        FieldKind::Float => rsx! {
            NumberField::<f64> {
                value: f64::try_from(value).unwrap_or_default(),
                min: field.bound("min"),
                max: field.bound("max"),
                onchange: move |number: f64| single(Value::Float(number)),
            }
        },

        FieldKind::Bool => rsx! {
            ToggleField {
                value: matches!(value.value(), Some(Value::Bool(true))),
                onchange: move |on: bool| single(Value::Bool(on)),
            }
        },

        FieldKind::Enum(options) => {
            let current = match value.value() {
                Some(Value::Enum(name)) => name.clone(),
                _ => options.first().cloned().unwrap_or_default(),
            };

            rsx! {
                EnumField {
                    options: options.clone(),
                    value: current,
                    onchange: move |name: String| single(Value::Enum(name)),
                }
            }
        }

        FieldKind::Reference => rsx! {
            ReferenceField {
                value: value.value().cloned().unwrap_or_default(),
                onchange: move |value: Value| single(value),
            }
        },

        FieldKind::Struct(_) => rsx! {
            StructEditor { title, def: def.to_vec(), values, onchange }
        },

        FieldKind::List => rsx! {
            ListEditor { title, def: def.to_vec(), values, onchange }
        },

        FieldKind::Map => rsx! {
            MapEditor { title, def: def.to_vec(), values, onchange }
        },

        FieldKind::Variant(_) => rsx! {
            VariantEditor { title, def: def.to_vec(), values, onchange }
        },
    }
}
