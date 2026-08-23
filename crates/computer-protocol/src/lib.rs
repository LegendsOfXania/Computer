use model::{EntryData, PageType, Value};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Connect {
        token: String,
        protocol_version: u32,
    },

    CreateEntry {
        page_id: String,
        entry_id: String,
        data: EntryData,
    },

    CreatePage {
        id: String,
        name: String,
        page_type: PageType,
        priority: u32,
    },

    ClosePage {
        page_id: String,
    },

    DeleteEntry {
        page_id: String,
        entry_id: String,
    },

    DeletePage {
        id: String,
    },

    EditEntry {
        page_id: String,
        entry_id: String,
        field: String,
        value: Value,
    },

    EditPage {
        page: PageInfo,
    },

    OpenPage {
        id: String,
    },

    Publish,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    ConnectionResult {
        protocol_version: u32,
        result: ConnectionResult,
    },

    EntryCreated {
        page_id: String,
        entry_id: String,
        data: EntryData,
    },

    EntryDeleted {
        page_id: String,
        entry_id: String,
    },

    EntryEdited {
        page_id: String,
        entry_id: String,
        field: String,
        value: Value,
    },

    PageContent {
        page_id: String,
        content: String,
    },

    PageCreated {
        page: PageInfo,
    },

    PageDeleted {
        id: String,
    },

    PageEdited {
        page: PageInfo,
    },

    PageTree {
        pages: Vec<PageInfo>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionResult {
    Connected,
    Error {
        message: String,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageInfo {
    pub id: String,
    pub name: String,
    pub page_type: PageType,
    pub priority: u32,
}

impl PageInfo {
    #[inline]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        page_type: PageType,
        priority: u32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            page_type,
            priority,
        }
    }
}