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