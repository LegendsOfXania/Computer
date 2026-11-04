use std::rc::Rc;

use computer_model::{field::Variant, value::Value};
use dioxus::prelude::*;
use indexmap::IndexMap;

use super::FieldGroup;
use crate::components::inspector::field::{frame::Section, input::EnumField, ops::ValueExt};

#[component]
pub fn VariantEditor(
    title: String,
    variants: Vec<Variant>,
    value: Value,
    onchange: EventHandler<Value>,
) -> Element {
    let variants = Rc::new(variants);

    let active = match &value {
        Value::Variant { name, .. } => variants.iter().find(|variant| variant.name == *name),
        _ => None,
    }
    .or_else(|| variants.first());

    let Some(active) = active else {
        return rsx! {};
    };

    let values = match value {
        Value::Variant { name, fields } if name == active.name => fields,
        _ => active.default_value().into_fields(),
    };

    let name = active.name.clone();
    let options: Vec<String> = variants
        .iter()
        .map(|variant| variant.name.clone())
        .collect();

    let pick = {
        let variants = variants.clone();

        move |picked: String| {
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
        Section { title,
            div { class: "field-variant",
                EnumField {
                    icon: "git-branch",
                    options,
                    value: name,
                    onchange: pick,
                }

                if !active.fields.is_empty() {
                    FieldGroup {
                        fields: active.fields.clone(),
                        values,
                        onchange: update,
                    }
                }
            }
        }
    }
}
