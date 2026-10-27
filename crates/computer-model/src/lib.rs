use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    id::{EntryId, PageId},
    page::Page,
};

pub mod entry;
pub mod field;
pub mod id;
pub mod interaction;
pub mod ipc;
pub mod page;
pub mod protocol;
pub mod value;

pub use entry::{Entry, EntryDefinition};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Library {
    pub pages: IndexMap<PageId, Page>,
    pub entries: IndexMap<EntryId, Entry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry(pub Vec<EntryDefinition>);

impl Registry {
    pub fn new(entries: Vec<EntryDefinition>) -> Self {
        Self(entries)
    }
}
