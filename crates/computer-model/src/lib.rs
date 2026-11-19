use serde::{Deserialize, Serialize};

pub mod entry;
pub mod field;
pub mod id;
pub mod page;
pub mod value;

#[cfg(feature = "engine")]
pub mod interaction;
#[cfg(feature = "engine")]
pub mod ipc;
#[cfg(feature = "panel")]
pub mod library;
#[cfg(feature = "panel")]
pub mod protocol;
pub mod wit;

pub use entry::{Entry, EntryDefinition};
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
