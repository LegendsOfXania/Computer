use serde::{Deserialize, Serialize};

use crate::{
    entry::EntryDefinition,
    id::{EntryId, PageId},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub id: PageId,
    pub kind: PageKind,
    pub name: String,
    pub priority: i32,
    pub chapter: Option<String>,
    pub entries: Vec<EntryId>,
}

#[cfg(feature = "panel")]
impl Page {
    pub fn new(name: String, kind: PageKind) -> Result<Self, String> {
        Ok(Self {
            id: PageId::generate()?,
            kind,
            name,
            priority: 0,
            chapter: None,
            entries: Vec::new(),
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageKind {
    #[default]
    Sequence,
    Static,
}

impl PageKind {
    pub const ALL: [Self; 2] = [Self::Sequence, Self::Static];

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Sequence => "sequence",
            Self::Static => "static",
        }
    }

    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Sequence => "article",
            Self::Static => "bulletlist",
        }
    }

    #[must_use]
    pub fn accepts(&self, definition: &EntryDefinition) -> bool {
        definition
            .tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(self.as_str()))
    }
}
