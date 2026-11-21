use computer_model::{Field as Definition, FieldNode, Node, Value};
use dioxus::prelude::*;

use super::{assemble, count, edited};
use crate::components::inspector::field::{
    Field,
    default::defaults,
    frame::{Card, Collapse},
};

#[component]
pub fn ListEditor(
    title: String,
    def: Vec<Definition>,
    values: Vec<Value>,
    onchange: EventHandler<Vec<Value>>,
) -> Element {
    let mut inner = FieldNode::new(&def, 0)
        .children()
        .next()
        .map(|child| child.subtree().to_vec())
        .unwrap_or_default();

    if let Some(field) = inner.first_mut() {
        field.name.clear();
    }

    let node = Node::new(&values, 0);

    let items: Vec<Vec<Value>> = if matches!(node.value(), Some(Value::List(_))) {
        node.children()
            .map(|child| child.subtree().to_vec())
            .collect()
    } else {
        Vec::new()
    };

    let items = use_memo(use_reactive!(|items| items));
    let template = defaults(&inner);
    let emit = move |items: Vec<Vec<Value>>| {
        onchange.call(assemble(Value::List(count(items.len())), items));
    };

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
                        emit(
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
                        emit(
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
                        def: inner.clone(),
                        values: item,
                        onchange: move |subtree: Vec<Value>| {
                            emit(
                                edited(
                                    items,
                                    |next| {
                                        if let Some(slot) = next.get_mut(index) {
                                            *slot = subtree;
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
                emit(edited(items, |next| next.push(template.clone())));
            },
            {rows.into_iter()}
        }
    }
}
