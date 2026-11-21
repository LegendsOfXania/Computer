use serde::{Deserialize, Serialize};

pub mod id;
pub mod page;

#[cfg(feature = "engine")]
pub mod interaction;
#[cfg(feature = "engine")]
pub mod ipc;
#[cfg(feature = "panel")]
pub mod library;
#[cfg(feature = "panel")]
pub mod protocol;
pub mod wit;

pub use wit::{Entry, EntryDefinition, Field, FieldKind, FieldNode, Node, Value};
#[cfg(feature = "panel")]
pub use library::Library;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry(pub Vec<EntryDefinition>);

impl Registry {
    #[must_use]
    pub fn new(entries: Vec<EntryDefinition>) -> Self {
        Self(entries)
    }
}