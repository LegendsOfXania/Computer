use serde::{Deserialize, Serialize};

mod wit;
pub mod id;
#[cfg(feature = "panel")]
pub mod library;
#[cfg(feature = "panel")]
pub mod page;
#[cfg(feature = "panel")]
pub mod protocol;

#[cfg(feature = "extension")]
pub mod ipc;

#[cfg(feature = "panel")]
pub use library::Library;
pub use wit::{Entry, EntryDefinition, Field, FieldKind, FieldNode, Node, Value};

#[cfg(feature = "extension")]
pub use wit::{Fire, Interaction, Trigger};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry(pub Vec<EntryDefinition>);

impl Registry {
    #[must_use]
    pub fn new(entries: Vec<EntryDefinition>) -> Self {
        Self(entries)
    }
}