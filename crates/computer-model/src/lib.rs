use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{key::EntryKey, page::Page};

pub mod ipc;
pub mod protocol;
pub mod entry;
pub mod field;
pub mod key;
pub mod page;
pub mod value;

pub use entry::{Entry, EntryDefinition};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Library {
    #[serde(with = "page::pages")]
    pub pages: IndexMap<u64, Page>,

    #[serde(with = "entry::entries")]
    pub entries: IndexMap<EntryKey, Entry>,
}

mod list {
    use std::hash::Hash;

use indexmap::IndexMap;
use serde::{Deserializer, Serializer};

use super::*;

    pub fn serialize<K, V, S>(map: &IndexMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        V: Serialize,
        S: Serializer,
    {
        serializer.collect_seq(map.values())
    }

    pub fn deserialize<'de, K, V, D>(
        deserializer: D,
        key: impl Fn(&V) -> K,
    ) -> Result<IndexMap<K, V>, D::Error>
    where
        K: Hash + Eq,
        V: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        let list = Vec::<V>::deserialize(deserializer)?;

        Ok(list.into_iter().map(|item| (key(&item), item)).collect())
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry(pub Vec<EntryDefinition>);

impl Registry {
    pub fn new(entries: Vec<EntryDefinition>) -> Self {
        Self(entries)
    }

    pub fn extend(&mut self, other:Registry) {
        self.0.extend(other.0);
    }
}
