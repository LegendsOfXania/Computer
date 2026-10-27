use serde::{Deserialize, Serialize};

use crate::{
    entry::Entry,
    id::{EntryId, PageId},
    page::Page,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    PageCreated { page: Page },

    PageUpdated { page: Page },

    PageDeleted { page_id: PageId },

    EntryCreated { entry: Entry },

    EntryUpdated { entry: Entry },

    EntryDeleted { entry_id: EntryId },

    EntryMoved { entry_id: EntryId, page_id: PageId },

    Pushed,
}
