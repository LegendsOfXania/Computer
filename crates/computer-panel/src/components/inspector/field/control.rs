use computer_model::{
    field::{FieldDefinition, FieldKind},
    value::Value,
};
use dioxus::prelude::*;
use indexmap::IndexMap;

use crate::components::inspector::field::{
    composite::{ListEditor, MapEditor, StructEditor, VariantEditor},
    input::{EnumField, NumberField, ReferenceField, TextField, ToggleField},
};

pub fn control(
    def: &FieldDefinition,
    value: Value,
    onchange: EventHandler<Value>,
    title: String,
) -> Element {
    match &def.kind {
        FieldKind::String => rsx! {
            TextField {
                value: String::try_from(value).unwrap_or_default(),
                multiline: def.has_flag("multiline"),
                onchange: move |text: String| onchange.call(Value::String(text)),
            }
        },

        FieldKind::Int => rsx! {
            NumberField::<i64> {
                value: i64::try_from(value).unwrap_or_default(),
                min: def.bound("min"),
                max: def.bound("max"),
                onchange: move |number: i64| onchange.call(Value::Int(number)),
            }
        },

        FieldKind::Float => rsx! {
            NumberField::<f64> {
                value: f64::try_from(value).unwrap_or_default(),
                min: def.bound("min"),
                max: def.bound("max"),
                onchange: move |number: f64| onchange.call(Value::Float(number)),
            }
        },

        FieldKind::Bool => rsx! {
            ToggleField {
                value: matches!(value, Value::Bool(true)),
                onchange: move |on: bool| onchange.call(Value::Bool(on)),
            }
        },

        FieldKind::Enum(options) => {
            let current = match value {
                Value::Enum(name) => name,
                _ => options.first().cloned().unwrap_or_default(),
            };

            rsx! {
                EnumField {
                    options: options.clone(),
                    value: current,
                    onchange: move |name: String| onchange.call(Value::Enum(name)),
                }
            }
        }

        FieldKind::Reference => rsx! {
            ReferenceField { value, onchange }
        },

        FieldKind::Struct(fields) => rsx! {
            StructEditor {
                title,
                fields: fields.clone(),
                values: IndexMap::try_from(value).unwrap_or_default(),
                onchange: move |values: IndexMap<String, Value>| onchange.call(Value::Struct(values)),
            }
        },

        FieldKind::List(inner) => rsx! {
            ListEditor {
                title,
                inner: (**inner).clone(),
                items: Vec::<Value>::try_from(value).unwrap_or_default(),
                onchange: move |items: Vec<Value>| onchange.call(Value::List(items)),
            }
        },

        FieldKind::Map {
            key: map_key,
            value: map_value,
        } => rsx! {
            MapEditor {
                title,
                key_kind: (**map_key).clone(),
                value_kind: (**map_value).clone(),
                pairs: Vec::<(Value, Value)>::try_from(value).unwrap_or_default(),
                onchange: move |pairs: Vec<(Value, Value)>| onchange.call(Value::Map(pairs)),
            }
        },

        FieldKind::Variant(variants) => rsx! {
            VariantEditor { title, variants: variants.clone(), value, onchange }
        },
    }
}
