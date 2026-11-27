mod generated {
    include!(concat!(env!("OUT_DIR"), "/ipc.rs"));
    include!(concat!(env!("OUT_DIR"), "/types.rs"));
}

mod entry;
mod field;
mod tree;
mod value;

#[cfg(feature = "extension")]
pub use generated::{Interaction, Trigger, Fire};

pub use generated::{Entry, EntryDefinition, Field, FieldKind, Value};

pub use tree::{FieldNode, Node};