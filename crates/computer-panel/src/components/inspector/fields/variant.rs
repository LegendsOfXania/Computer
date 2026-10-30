use computer_model::{field::Variant, value::Value};
use dioxus::prelude::*;
use indexmap::IndexMap;

use crate::components::inspector::fields::fields::Fields;

#[component]
pub fn VariantEditor(
    variants: Vec<Variant>,
    value: Value,
    onchange: EventHandler<Value>,
) -> Element {
    let (name, values) = match value {
        Value::Variant { name, fields } if variants.iter().any(|variant| variant.name == name) => {
            (name, fields)
        }
        _ => match variants.first() {
            Some(variant) => {
                let fields = match variant.default_value() {
                    Value::Variant { fields, .. } => fields,
                    _ => IndexMap::new(),
                };

                (variant.name.clone(), fields)
            }
            None => return rsx! {},
        },
    };

    let fields = variants
        .iter()
        .find(|variant| variant.name == name)
        .map(|variant| variant.fields.clone())
        .unwrap_or_default();

    let pick = {
        let variants = variants.clone();

        move |event: FormEvent| {
            let picked = event.value();

            if let Some(variant) = variants.iter().find(|variant| variant.name == picked) {
                onchange.call(variant.default_value());
            }
        }
    };

    let update = {
        let name = name.clone();

        move |fields: IndexMap<String, Value>| {
            onchange.call(Value::Variant {
                name: name.clone(),
                fields,
            });
        }
    };

    rsx! {
        div { class: "field-variant",
            select { class: "field-input", onchange: pick,
                for variant in variants.iter() {
                    option {
                        key: "{variant.name}",
                        value: "{variant.name}",
                        selected: variant.name == name,
                        "{variant.name}"
                    }
                }
            }
            if !fields.is_empty() {
                Fields { fields, values, onchange: update }
            }
        }
    }
}
