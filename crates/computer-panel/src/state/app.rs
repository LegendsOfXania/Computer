use std::collections::HashMap;

use computer_model::{
    Entry, EntryDefinition, Library, Registry,
    id::{EntryId, PageId},
    page::Page,
    protocol::message::ServerMessage,
};
use dioxus::logger::tracing::warn;
use dioxus::signals::{ReadableExt, Signal, WritableExt};

use crate::state::ConnectionStatus;

#[derive(Clone, Copy, Default)]
pub struct AppState {
    pub library: Signal<Library>,
    pub registry: Signal<HashMap<String, EntryDefinition>>,
    pub status: Signal<ConnectionStatus>,
}

impl AppState {
    pub fn with_page<T>(&self, id: PageId, f: impl FnOnce(&Page) -> T) -> Option<T> {
        self.library.read().pages.get(&id).map(f)
    }

    pub fn with_entry<T>(&self, id: EntryId, f: impl FnOnce(&Entry) -> T) -> Option<T> {
        self.library.read().entries.get(&id).map(f)
    }

    #[must_use]
    pub fn definition(&self, kind: &str) -> Option<EntryDefinition> {
        self.registry.read().get(kind).cloned()
    }

    #[must_use]
    pub fn look_of(&self, kind: &str) -> (String, String) {
        self.registry
            .read()
            .get(kind)
            .map(|definition| (definition.icon.clone(), definition.color.clone()))
            .unwrap_or_default()
    }

    pub fn apply(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::Library(library) => self.library.set(library),

            ServerMessage::Registry(Registry(definitions)) => {
                self.registry.set(
                    definitions
                        .into_iter()
                        .map(|definition| (definition.kind.clone(), definition))
                        .collect(),
                );
            }

            ServerMessage::Event { event } => {
                if let Err(error) = self.library.write().apply(&event) {
                    warn!("Ignored an event that does not match the library: {error}");
                }
            }

            _ => {}
        }
    }
}
