use serde::{Deserialize, Serialize};

use crate::{
    Entry,
    id::{EntryId, PageId},
    page::Page,
    protocol::request::Request,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    PageCreated { page: Page },

    PageUpdated { page: Page },

    PageDeleted { page_id: PageId },

    EntryCreated { entry: Entry, page_id: PageId },

    EntryUpdated { entry: Entry },

    EntryDeleted { entry_id: EntryId },

    EntryMoved { entry_id: EntryId, page_id: PageId },

    Pushed,
}

impl From<Request> for Event {
    fn from(request: Request) -> Self {
        match request {
            Request::CreatePage { page } => Self::PageCreated { page },
            Request::UpdatePage { page } => Self::PageUpdated { page },
            Request::DeletePage { page_id } => Self::PageDeleted { page_id },
            Request::CreateEntry { entry, page_id } => Self::EntryCreated { entry, page_id },
            Request::UpdateEntry { entry } => Self::EntryUpdated { entry },
            Request::DeleteEntry { entry_id } => Self::EntryDeleted { entry_id },
            Request::MoveEntry { entry_id, page_id } => Self::EntryMoved { entry_id, page_id },
            Request::Push => Self::Pushed,
        }
    }
}
