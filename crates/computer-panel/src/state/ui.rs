use computer_model::{
    EntryDefinition,
    id::{EntryId, PageId},
};
use dioxus::prelude::*;

use crate::state::app::AppState;

#[derive(Clone, Copy, PartialEq)]
pub enum Picked {
    Entry(EntryId),
    Page(PageId),
}

#[derive(Clone, PartialEq)]
pub struct SearchRequest {
    pub scope: u8,
    pub accepts: Option<EntryDefinition>,
    pub pick: Callback<Picked>,
}

#[derive(Clone, Copy, Default)]
pub struct UiState {
    opened_page: Signal<Option<PageId>>,
    opened_entry: Signal<Option<EntryId>>,
    search: Signal<Option<SearchRequest>>,
}

impl UiState {
    #[must_use]
    pub fn get_opened_page(&self) -> Option<PageId> {
        *self.opened_page.read()
    }

    pub fn set_opened_page(&mut self, page: Option<PageId>) {
        self.opened_page.set(page);
        self.opened_entry.set(None);
    }

    #[must_use]
    pub fn get_opened_entry(&self) -> Option<EntryId> {
        *self.opened_entry.read()
    }

    pub fn set_opened_entry(&mut self, app: &AppState, entry: Option<EntryId>) {
        let Some(entry) = entry else {
            self.opened_entry.set(None);
            return;
        };

        let page = app.library.read().page_of(entry).map(|page| page.id);

        if let Some(page) = page
            && self.get_opened_page() != Some(page)
        {
            self.opened_page.set(Some(page));
        }

        self.opened_entry.set(Some(entry));
    }

    #[must_use]
    pub fn get_search(&self) -> Option<SearchRequest> {
        self.search.read().clone()
    }

    pub fn set_search(&mut self, search: Option<SearchRequest>) {
        self.search.set(search);
    }
}
