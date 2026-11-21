use computer_model::{Field as Definition, FieldNode, Node, Value};
use dioxus::prelude::*;

use super::{assemble, child_values, names};
use crate::components::inspector::field::{default::defaults, frame::Section, input::EnumField};

use super::FieldGroup;

type Case = (String, Vec<Vec<Definition>>);

fn with_case(name: &str, defs: &[Vec<Definition>], children: Vec<Vec<Value>>) -> Vec<Value> {
    assemble(Value::Variant((name.to_owned(), names(defs))), children)
}

#[component]
pub fn VariantEditor(
    title: String,
    def: Vec<Definition>,
    values: Vec<Value>,
    onchange: EventHandler<Vec<Value>>,
) -> Element {
    let cases: Vec<Case> = FieldNode::new(&def, 0)
        .cases()
        .map(|(name, fields)| {
            (
                name.to_owned(),
                fields
                    .iter()
                    .map(|field| field.subtree().to_vec())
                    .collect(),
            )
        })
        .collect();

    let node = Node::new(&values, 0);

    let current = match node.value() {
        Some(Value::Variant((name, _))) => Some(name.as_str()),
        _ => None,
    };

    let Some((name, defs)) = current
        .and_then(|current| cases.iter().find(|(name, _)| name == current))
        .or_else(|| cases.first())
    else {
        return rsx! {};
    };

    let children = if current == Some(name.as_str()) {
        child_values(defs, node)
    } else {
        defs.iter().map(|def| defaults(def)).collect()
    };

    let options: Vec<String> = cases.iter().map(|(name, _)| name.clone()).collect();

    let starts: Vec<(String, Vec<Value>)> = cases
        .iter()
        .map(|(name, defs)| {
            let children = defs.iter().map(|def| defaults(def)).collect();

            (name.clone(), with_case(name, defs, children))
        })
        .collect();

    let pick = move |picked: String| {
        if let Some((_, start)) = starts.iter().find(|(name, _)| name == &picked) {
            onchange.call(start.clone());
        }
    };

    let update = {
        let (name, defs) = (name.clone(), defs.clone());

        move |children: Vec<Vec<Value>>| onchange.call(with_case(&name, &defs, children))
    };

    rsx! {
        Section { title,
            div { class: "field-variant",
                EnumField {
                    icon: "git-branch",
                    options,
                    value: name.clone(),
                    onchange: pick,
                }

                if !defs.is_empty() {
                    FieldGroup {
                        defs: defs.clone(),
                        values: children,
                        onchange: update,
                    }
                }
            }
        }
    }
}
