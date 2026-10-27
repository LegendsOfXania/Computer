use serde::{Deserialize, Serialize};

use crate::{
    entry::Entry,
    id::{EntryId, PageId},
    page::Page,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    CreatePage { page: Page },

    UpdatePage { page: Page },

    DeletePage { page_id: PageId },

    CreateEntry { entry: Entry },

    UpdateEntry { entry: Entry },

    DeleteEntry { entry_id: EntryId },

    MoveEntry { entry_id: EntryId, page_id: PageId },

    Push,
}
