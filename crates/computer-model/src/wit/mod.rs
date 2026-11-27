include!(concat!(env!("OUT_DIR"), "/wit_bindings.rs"));
include!(concat!(env!("OUT_DIR"), "/variant_impls.rs"));

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

pub use tree::{FieldNode, Node};