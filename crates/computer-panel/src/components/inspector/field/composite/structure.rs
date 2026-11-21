use computer_model::{Field as Definition, FieldNode, Node, Value};
use dioxus::prelude::*;

use super::{assemble, child_values, names};
use crate::components::inspector::field::{Field, frame::Section};

#[component]
pub fn StructEditor(
    title: String,
    def: Vec<Definition>,
    values: Vec<Value>,
    onchange: EventHandler<Vec<Value>>,
) -> Element {
    let defs: Vec<_> = FieldNode::new(&def, 0)
        .children()
        .map(|child| child.subtree().to_vec())
        .collect();

    let children = child_values(&defs, Node::new(&values, 0));
    let names = names(&defs);

    rsx! {
        Section { title,
            FieldGroup {
                defs,
                values: children,
                onchange: move |children: Vec<Vec<Value>>| {
                    onchange.call(assemble(Value::Struct(names.clone()), children));
                },
            }
        }
    }
}

#[component]
pub fn FieldGroup(
    defs: Vec<Vec<Definition>>,
    values: Vec<Vec<Value>>,
    onchange: EventHandler<Vec<Vec<Value>>>,
) -> Element {
    let rows = defs
        .into_iter()
        .zip(values.clone())
        .enumerate()
        .map(|(index, (def, value))| {
            let name = def
                .first()
                .map(|field| field.name.clone())
                .unwrap_or_default();
            let all = values.clone();

            rsx! {
                Field {
                    key: "{name}",
                    def,
                    values: value,
                    onchange: move |subtree: Vec<Value>| {
                        let mut next = all.clone();

                        if let Some(slot) = next.get_mut(index) {
                            *slot = subtree;
                        }

                        onchange.call(next);
                    },
                }
            }
        });

    rsx! {
        div { class: "field-group", {rows} }
    }
}
