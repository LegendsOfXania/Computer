use serde::{Deserialize, Serialize};

use crate::{entry::{Entry, EntryDefinition}, page::Page};

pub mod protocol;
pub mod entry;
pub mod field;
pub mod key;
pub mod page;
pub mod value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub pages: Vec<Page>,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry(pub Vec<EntryDefinition>);