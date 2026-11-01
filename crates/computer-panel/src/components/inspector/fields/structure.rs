use computer_model::{field::FieldDefinition, value::Value};
use dioxus::prelude::*;
use indexmap::IndexMap;

use crate::components::inspector::fields::{card::Card, collapse::Collapse, fields::Fields};

#[component]
pub fn StructEditor(
    title: String,
    fields: Vec<FieldDefinition>,
    values: IndexMap<String, Value>,
    onchange: EventHandler<IndexMap<String, Value>>,
) -> Element {
    let group = rsx! {
        Fields { fields, values, onchange }
    };

    if title.is_empty() {
        group
    } else {
        rsx! {
            Collapse { title,
                Card { {group} }
            }
        }
    }
}
