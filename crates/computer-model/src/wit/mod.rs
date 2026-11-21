wit_bindgen::generate!({
    path: "../../wit",
    world: "cosmos",
    additional_derives: [
        Clone,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
    ],
});

mod entry;
mod field;
mod tree;
mod value;

pub use exports::legends::cosmos::extension::Interaction;
pub use legends::cosmos::{
    entry::{Entry, EntryDefinition},
    field::{Field, FieldKind, Value},
};
pub use tree::{Cursor, FieldNode, Node, Tree};