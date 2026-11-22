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

#[cfg(feature = "extension")]
pub use exports::legends::cosmos::extension::{Interaction, Trigger};

#[cfg(feature = "extension")]
pub use legends::cosmos::engine::Fire;

pub use legends::cosmos::{
    entry::{Entry, EntryDefinition},
    field::{Field, FieldKind, Value},
};

pub use tree::{Cursor, FieldNode, Node, Tree};