mod list;
mod map;
mod structure;
mod variant;

use computer_model::{Field as Definition, Node, Value};
use dioxus::prelude::*;

use super::default::defaults;

pub use list::ListEditor;
pub use map::MapEditor;
pub use structure::{FieldGroup, StructEditor};
pub use variant::VariantEditor;

fn edited<T: Clone + PartialEq + 'static>(
    items: Memo<Vec<T>>,
    edit: impl FnOnce(&mut Vec<T>),
) -> Vec<T> {
    let mut next = items.peek().clone();

    edit(&mut next);

    next
}

fn count(items: usize) -> u32 {
    u32::try_from(items).unwrap_or(u32::MAX)
}

fn assemble(head: Value, children: Vec<Vec<Value>>) -> Vec<Value> {
    std::iter::once(head)
        .chain(children.into_iter().flatten())
        .collect()
}

fn names(defs: &[Vec<Definition>]) -> Vec<String> {
    defs.iter()
        .filter_map(|def| def.first())
        .map(|field| field.name.clone())
        .collect()
}

fn child_values(defs: &[Vec<Definition>], parent: Node) -> Vec<Vec<Value>> {
    defs.iter()
        .map(|def| {
            def.first()
                .and_then(|field| parent.get(&field.name))
                .map_or_else(|| defaults(def), |node| node.subtree().to_vec())
        })
        .collect()
}
