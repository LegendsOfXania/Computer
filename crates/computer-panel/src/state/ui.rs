use computer_model::id::{EntryId, PageId};
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct UiState {
    pub opened_page: Signal<Option<PageId>>,
    pub opened_entry: Signal<Option<EntryId>>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            opened_page: Signal::new(None),
            opened_entry: Signal::new(None),
        }
    }
}
