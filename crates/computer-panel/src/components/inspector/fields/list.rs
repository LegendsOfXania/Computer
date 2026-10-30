use computer_model::{
    field::{FieldDefinition, FieldKind},
    value::Value,
};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::components::{inspector::fields::Field, utils::btn::Btn};

#[component]
pub fn ListField(
    inner: FieldKind,
    items: Vec<Value>,
    onchange: EventHandler<Vec<Value>>,
) -> Element {
    let def = FieldDefinition::new(String::new(), inner);

    let rows = items.iter().cloned().enumerate().map(|(index, item)| {
        let (for_change, for_remove) = (items.clone(), items.clone());

        rsx! {
            div { class: "field-item", key: "{index}",
                div { class: "field-item-body",
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
                Btn {
                    icon: "trash",
                    tooltip: tid!("inspector-remove"),
                    onclick: move |_| {
                        let mut next = for_remove.clone();
                        next.remove(index);
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
        div { class: "field-list",
            {rows}
            Btn { icon: "plus", tooltip: tid!("inspector-add"), onclick: add }
        }
    }
}
