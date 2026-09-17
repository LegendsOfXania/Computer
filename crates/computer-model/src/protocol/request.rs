use serde::{Deserialize, Serialize};

use crate::{entry::Entry, key::EntryKey, page::Page};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    CreatePage { 
        page: Page
    },
    UpdatePage {
        page: Page
    },
    DeletePage { 
        page_id: u64
    },

    CreateEntry {
        entry: Entry
    },
    UpdateEntry {
        entry: Entry
    },
    DeleteEntry {
        key: EntryKey
    },
    MoveEntry {
        key: EntryKey,
        page_id: u64
    },

    Publish,
}