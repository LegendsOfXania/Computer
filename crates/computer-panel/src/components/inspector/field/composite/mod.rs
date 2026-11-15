mod list;
mod map;
mod structure;
mod variant;

use dioxus::prelude::*;

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
