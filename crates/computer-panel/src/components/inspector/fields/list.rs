use computer_model::{
    field::{FieldDefinition, FieldKind},
    value::Value,
};
use dioxus::prelude::*;

use crate::components::inspector::fields::{Field, card::Card, collapse::Collapse};

#[component]
pub fn ListField(
    title: String,
    inner: FieldKind,
    items: Vec<Value>,
    onchange: EventHandler<Vec<Value>>,
) -> Element {
    let def = FieldDefinition::new(String::new(), inner);

    let rows = items.iter().cloned().enumerate().map(|(index, item)| {
        let (for_change, for_remove, for_duplicate) = (items.clone(), items.clone(), items.clone());

        rsx! {
            Card {
                key: "{index}",
                title: format!("{title} #{}", index + 1),
                duplicable: true,
                onduplicate: move |_| {
                    let mut next = for_duplicate.clone();
                    next.insert(index + 1, for_duplicate[index].clone());
                    onchange.call(next);
                },
                onremove: move |_| {
                    let mut next = for_remove.clone();
                    next.remove(index);
                    onchange.call(next);
                },

                Field {
                    def: def.clone(),
                    value: item,
                    onchange: move |value: Value| {
                        let mut next = for_change.clone();
                        next[index] = value;
                        onchange.call(next);
                    },
                }
            }
        }
    });

    let add = {
        let items = items.clone();
        let def = def.clone();

        move |_| {
            let mut next = items.clone();
            next.push(def.default_value());
            onchange.call(next);
        }
    };

    rsx! {
        Collapse {
            title: title.clone(),
            count: Some(items.len()),
            addable: true,
            onadd: add,
            {rows}
        }
    }
}
