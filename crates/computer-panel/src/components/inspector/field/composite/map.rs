use computer_model::{Field as Definition, FieldNode, Node, Value};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use super::{assemble, count, edited};
use crate::components::inspector::field::{
    Field,
    default::defaults,
    frame::{Card, Collapse},
};

type Pair = (Vec<Value>, Vec<Value>);

#[component]
pub fn MapEditor(
    title: String,
    def: Vec<Definition>,
    values: Vec<Value>,
    onchange: EventHandler<Vec<Value>>,
) -> Element {
    let node = FieldNode::new(&def, 0);

    let named = |index: usize, name: String| {
        let mut subtree = node
            .children()
            .nth(index)
            .map(|child| child.subtree().to_vec())
            .unwrap_or_default();

        if let Some(field) = subtree.first_mut() {
            field.name = name;
        }

        subtree
    };

    let key_def = named(0, tid!("inspector-key"));
    let value_def = named(1, tid!("inspector-value"));

    let current = Node::new(&values, 0);

    let pairs: Vec<Pair> = current
        .pairs()
        .filter(|_| matches!(current.value(), Some(Value::Map(_))))
        .map(|(key, value)| (key.subtree().to_vec(), value.subtree().to_vec()))
        .collect();

    let pairs = use_memo(use_reactive!(|pairs| pairs));
    let template: Pair = (defaults(&key_def), defaults(&value_def));
    let emit = move |pairs: Vec<Pair>| {
        onchange.call(assemble(
            Value::Map(count(pairs.len())),
            pairs
                .into_iter()
                .flat_map(|(key, value)| [key, value])
                .collect(),
        ));
    };

    let rows: Vec<_> = pairs
        .read()
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, (pair_key, pair_value))| {
            rsx! {
                Card {
                    key: "{index}",
                    title: format!("{title} #{}", index + 1),
                    removable: true,

                    onremove: move |_| {
                        emit(
                            edited(
                                pairs,
                                |next| {
                                    if index < next.len() {
                                        next.remove(index);
                                    }
                                },
                            ),
                        );
                    },

                    Field {
                        def: key_def.clone(),
                        values: pair_key,
                        onchange: move |subtree: Vec<Value>| {
                            emit(
                                edited(
                                    pairs,
                                    |next| {
                                        if let Some(pair) = next.get_mut(index) {
                                            pair.0 = subtree;
                                        }
                                    },
                                ),
                            );
                        },
                    }

                    Field {
                        def: value_def.clone(),
                        values: pair_value,
                        onchange: move |subtree: Vec<Value>| {
                            emit(
                                edited(
                                    pairs,
                                    |next| {
                                        if let Some(pair) = next.get_mut(index) {
                                            pair.1 = subtree;
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
            count: Some(pairs.read().len()),
            addable: true,
            onadd: move |_| {
                emit(edited(pairs, |next| next.push(template.clone())));
            },
            {rows.into_iter()}
        }
    }
}
