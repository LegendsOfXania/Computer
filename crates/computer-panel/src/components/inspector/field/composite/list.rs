use computer_model::{
    field::{FieldDefinition, FieldKind},
    value::Value,
};
use dioxus::prelude::*;

use crate::components::inspector::field::{
    Field,
    frame::{Card, Collapse},
};

#[component]
pub fn ListEditor(
    title: String,
    inner: FieldKind,
    items: Vec<Value>,
    onchange: EventHandler<Vec<Value>>,
) -> Element {
    let def = FieldDefinition::new(String::new(), inner);
    let template = def.default_value();
    let for_add = items.clone();

    let rows = items.iter().cloned().enumerate().map(|(index, item)| {
        let (for_change, for_duplicate, for_remove) = (items.clone(), items.clone(), items.clone());

        rsx! {
            Card {
                key: "{index}",
                title: format!("{title} #{}", index + 1),
                duplicable: true,
                removable: true,

                onduplicate: move |_| {
                    let mut next = for_duplicate.clone();
                    let item = next[index].clone();
                    next.insert(index + 1, item);
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

    let count = items.len();
    let add = move |_| {
        let mut next = for_add.clone();
        next.push(template.clone());
        onchange.call(next);
    };

    rsx! {
        Collapse {
            title: title.clone(),
            count: Some(count),
            addable: true,
            onadd: add,
            {rows}
        }
    }
}
