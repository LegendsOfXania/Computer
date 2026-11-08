use computer_model::id::{EntryId, PageId};
use dioxus::prelude::*;

use crate::state::app::AppState;

#[derive(Clone, Copy)]
pub struct UiState {
    opened_page: Signal<Option<PageId>>,
    opened_entry: Signal<Option<EntryId>>,
    search: Signal<Option<(u8, Callback<EntryId>)>>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            opened_page: Signal::new(None),
            opened_entry: Signal::new(None),
            search: Signal::new(None),
        }
    }

    pub fn get_opened_page(&self) -> Option<PageId> {
        self.opened_page.read().clone()
    }

    pub fn set_opened_page(&mut self, page: Option<PageId>) {
        self.opened_page.set(page);
        self.opened_entry.set(None);
    }

    pub fn get_opened_entry(&self) -> Option<EntryId> {
        self.opened_entry.read().clone()
    }

    pub fn set_opened_entry(&mut self, app: &AppState, entry: Option<EntryId>) {
        let Some(entry) = entry else {
            self.opened_entry.set(None);
            return;
        };

        let page = app
            .pages
            .read()
            .iter()
            .find(|(_, page)| page.entries.contains(&entry))
            .map(|(id, _)| id.clone());

        if let Some(page) = page {
            if self.get_opened_page().as_ref() != Some(&page) {
                self.opened_page.set(Some(page));
            }
        }

        self.opened_entry.set(Some(entry));
    }

    pub fn get_search(&self) -> Option<(u8, Callback<EntryId>)> {
        self.search.read().clone()
    }

    pub fn set_search(&mut self, search: Option<(u8, Callback<EntryId>)>) {
        self.search.set(search);
    }
}
