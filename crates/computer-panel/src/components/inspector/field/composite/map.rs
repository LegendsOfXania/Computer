use computer_model::{
    field::{FieldDefinition, FieldKind},
    value::Value,
};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use super::edited;
use crate::components::inspector::field::{
    Field,
    frame::{Card, Collapse},
};

#[component]
pub fn MapEditor(
    title: String,
    key_kind: FieldKind,
    value_kind: FieldKind,
    pairs: Vec<(Value, Value)>,
    onchange: EventHandler<Vec<(Value, Value)>>,
) -> Element {
    let pairs = use_memo(use_reactive!(|pairs| pairs));
    let template = (Value::from(&key_kind), Value::from(&value_kind));
    let key_def = FieldDefinition::new(tid!("inspector-key"), key_kind);
    let value_def = FieldDefinition::new(tid!("inspector-value"), value_kind);

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
                        onchange
                            .call(
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
                        value: pair_key,
                        onchange: move |value: Value| {
                            onchange
                                .call(
                                    edited(
                                        pairs,
                                        |next| {
                                            if let Some(pair) = next.get_mut(index) {
                                                pair.0 = value;
                                            }
                                        },
                                    ),
                                );
                        },
                    }

                    Field {
                        def: value_def.clone(),
                        value: pair_value,
                        onchange: move |value: Value| {
                            onchange
                                .call(
                                    edited(
                                        pairs,
                                        |next| {
                                            if let Some(pair) = next.get_mut(index) {
                                                pair.1 = value;
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
                onchange.call(edited(pairs, |next| next.push(template.clone())));
            },
            {rows.into_iter()}
        }
    }
}
