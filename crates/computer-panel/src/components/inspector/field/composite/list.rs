use computer_model::{
    field::{FieldDefinition, FieldKind},
    value::Value,
};
use dioxus::prelude::*;

use super::edited;
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
    let items = use_memo(use_reactive!(|items| items));
    let template = Value::from(&inner);
    let def = FieldDefinition::new(String::new(), inner);

    let rows: Vec<_> = items
        .read()
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, item)| {
            rsx! {
                Card {
                    key: "{index}",
                    title: format!("{title} #{}", index + 1),
                    duplicable: true,
                    removable: true,

                    onduplicate: move |_| {
                        onchange
                            .call(
                                edited(
                                    items,
                                    |next| {
                                        if let Some(item) = next.get(index).cloned() {
                                            next.insert(index + 1, item);
                                        }
                                    },
                                ),
                            );
                    },

                    onremove: move |_| {
                        onchange
                            .call(
                                edited(
                                    items,
                                    |next| {
                                        if index < next.len() {
                                            next.remove(index);
                                        }
                                    },
                                ),
                            );
                    },

                    Field {
                        def: def.clone(),
                        value: item,
                        onchange: move |value: Value| {
                            onchange
                                .call(
                                    edited(
                                        items,
                                        |next| {
                                            if let Some(slot) = next.get_mut(index) {
                                                *slot = value;
                                            }
                                        },
                                    ),
                                );
                        },
                    }
                }
            }
        })
        .collect();

    rsx! {
        Collapse {
            title: title.clone(),
            count: Some(items.read().len()),
            addable: true,
            onadd: move |_| {
                onchange.call(edited(items, |next| next.push(template.clone())));
            },
            {rows.into_iter()}
        }
    }
}
