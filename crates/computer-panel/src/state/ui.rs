use computer_model::key::EntryKey;
use dioxus::signals::Signal;

#[derive(Clone, Copy)]
pub struct UiState {
    pub opened_page: Signal<Option<u64>>,
    pub opened_entry: Signal<Option<EntryKey>>,
}

impl UiState {
    pub fn new() -> Self {
        Self { 
            opened_page: Signal::new(None), 
            opened_entry: Signal::new(None) 
        }
    }
}