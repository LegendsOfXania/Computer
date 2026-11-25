extern crate self as cosmos_sdk;

pub mod entry;
pub mod field;
pub mod primitives;

pub use computer_model;
pub use computer_model::{EntryDefinition, FieldKind, FieldNode, Node, Value, id};

pub use cosmos_sdk_macro::{Entry, Field, entry, tags, taggable};

pub use entry::{Entry, RegisteredEntry, Taggable};
pub use field::Field;

#[doc(hidden)]
pub mod __private {
    pub use crate::field::{define, define_struct, expect_struct, member};
}
