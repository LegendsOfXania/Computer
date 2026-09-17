use serde::{Deserialize, Serialize};

use crate::{entry::Entry, key::EntryKey, page::Page};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    PageCreated {
        page: Page,
    },

    PageUpdated {
        page: Page,
    },

    PageDeleted {
        page_id: u64,
    },

    EntryCreated {
        entry: Entry,
    },

    EntryUpdated {
        entry: Entry,
    },

    EntryDeleted {
        key: EntryKey,
    },

    EntryMoved {
        key: EntryKey,
        page_id: u64,
    },

    Published,
}