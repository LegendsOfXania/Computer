use std::rc::Rc;

use computer_model::{field::FieldDefinition, value::Value};
use dioxus::prelude::*;
use indexmap::IndexMap;

use crate::components::inspector::field::{Field, frame::Section, ops};

#[component]
pub fn StructEditor(
    title: String,
    fields: Vec<FieldDefinition>,
    values: IndexMap<String, Value>,
    onchange: EventHandler<IndexMap<String, Value>>,
) -> Element {
    rsx! {
        Section { title,
            FieldGroup { fields, values, onchange }
        }
    }
}

#[component]
pub fn FieldGroup(
    fields: Vec<FieldDefinition>,
    values: IndexMap<String, Value>,
    onchange: EventHandler<IndexMap<String, Value>>,
) -> Element {
    let values = Rc::new(values);

    let rows = fields.into_iter().map(|def| {
        let name = def.name.clone();
        let for_change = def.name.clone();
        let for_values = values.clone();
        let value = values
            .get(&name)
            .cloned()
            .unwrap_or_else(|| def.default_value());

        rsx! {
            Field {
                key: "{name}",
                def,
                value,
                onchange: move |value: Value| {
                    onchange.call(ops::insert(&for_values, for_change.clone(), value));
                },
            }
        }
    });

    rsx! {
        div { class: "field-group", {rows} }
    }
}
