use computer_model::{
    field::{FieldDefinition, FieldKind},
    value::Value,
};
use dioxus::prelude::*;
use dioxus_i18n::tid;

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
    let key_def = FieldDefinition::new(tid!("inspector-key").to_string(), key_kind);
    let value_def = FieldDefinition::new(tid!("inspector-value").to_string(), value_kind);
    let template = (key_def.default_value(), value_def.default_value());
    let for_add = pairs.clone();

    let rows = pairs
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, (pair_key, pair_value))| {
            let (for_key, for_value, for_remove) = (pairs.clone(), pairs.clone(), pairs.clone());

            rsx! {
                Card {
                    key: "{index}",
                    title: format!("{title} #{}", index + 1),
                    removable: true,

                    onremove: move |_| {
                        let mut next = for_remove.clone();
                        next.remove(index);
                        onchange.call(next);
                    },

                    Field {
                        def: key_def.clone(),
                        value: pair_key,
                        onchange: move |new_key: Value| {
                            let mut next = for_key.clone();
                            next[index].0 = new_key;
                            onchange.call(next);
                        },
                    }

                    Field {
                        def: value_def.clone(),
                        value: pair_value,
                        onchange: move |new_value: Value| {
                            let mut next = for_value.clone();
                            next[index].1 = new_value;
                            onchange.call(next);
                        },
                    }
                }
            }
        });

    let count = pairs.len();
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
