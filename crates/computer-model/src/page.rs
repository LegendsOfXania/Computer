use serde::{Deserialize, Serialize};

use crate::key::EntryKey;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub id: u64,
    pub name: String,
    pub kind: PageKind,
    pub priority: i32,
    pub chapter: Option<String>,
    pub entries: Vec<EntryKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PageKind {
    Sequence,
    Static,
}

pub(crate) mod pages {
    use indexmap::IndexMap;
    use serde::Deserializer;

    use super::*;

    pub(crate) use crate::list::serialize;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<IndexMap<u64, Page>, D::Error>
    where
        D: Deserializer<'de>,
    {
        crate::list::deserialize(deserializer, |page: &Page| page.id)
    }
}