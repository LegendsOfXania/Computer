use serde::{Deserialize, Serialize};

use crate::id::{EntryId, PageId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub id: PageId,
    pub kind: PageKind,
    pub name: String,
    pub priority: i32,
    pub chapter: Option<String>,
    pub entries: Vec<EntryId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PageKind {
    Sequence,
    Static,
}

impl PageKind {
    pub const ALL: [Self; 2] = [Self::Sequence, Self::Static];

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Sequence => "sequence",
            Self::Static => "static",
        }
    }

    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Sequence => "article",
            Self::Static => "bulletlist",
        }
    }
}
