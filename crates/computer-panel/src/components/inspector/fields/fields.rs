use computer_model::{field::FieldDefinition, value::Value};
use dioxus::prelude::*;
use indexmap::IndexMap;

use crate::components::inspector::fields::Field;

#[component]
pub fn Fields(
    fields: Vec<FieldDefinition>,
    values: IndexMap<String, Value>,
    onchange: EventHandler<IndexMap<String, Value>>,
) -> Element {
    rsx! {
        div { class: "field-group",
            for field in fields.iter() {
                Field {
                    key: "{field.name}",
                    def: field.clone(),
                    value: values.get(&field.name).cloned().unwrap_or_else(|| field.default_value()),
                    onchange: {
                        let (values, name) = (values.clone(), field.name.clone());

                        move |value: Value| {
                            let mut next = values.clone();
                            next.insert(name.clone(), value);
                            onchange.call(next);
                        }
                    },
                }
            }
        }
    }
}
