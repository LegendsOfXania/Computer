use std::rc::Rc;

use computer_model::{
    field::{FieldDefinition, FieldKind},
    value::Value,
};
use dioxus::prelude::*;

use crate::components::inspector::field::{
    Field,
    frame::{Card, Collapse},
    ops,
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
    let items = Rc::new(items);

    let rows = items.iter().cloned().enumerate().map(|(index, item)| {
        let (for_change, for_duplicate, for_remove) = (items.clone(), items.clone(), items.clone());

        rsx! {
            Card {
                key: "{index}",
                title: format!("{title} #{}", index + 1),
                duplicable: true,
                removable: true,
                onduplicate: move |_| onchange.call(ops::duplicate(&for_duplicate, index)),
                onremove: move |_| onchange.call(ops::remove(&for_remove, index)),

                Field {
                    def: def.clone(),
                    value: item,
                    onchange: move |value: Value| {
                        onchange.call(ops::update(&for_change, index, |slot| *slot = value));
                    },
                }
            }
        }
    });

    let count = items.len();
    let add = {
        let items = items.clone();

        move |_| onchange.call(ops::push(&items, template.clone()))
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
