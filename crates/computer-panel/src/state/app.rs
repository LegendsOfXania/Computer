use indexmap::IndexMap;
use std::collections::HashMap;

use computer_model::{
    entry::{Entry, EntryDefinition},
    key::EntryKey,
    page::Page,
    protocol::{event::Event, message::ServerMessage},
};
use dioxus::signals::{ReadableExt, Signal, WritableExt};

#[derive(Clone, Copy)]
pub struct AppState {
    pub pages: Signal<IndexMap<u64, Page>>,
    pub entries: Signal<IndexMap<EntryKey, Entry>>,
    pub registry: Signal<HashMap<String, EntryDefinition>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            pages: Signal::new(IndexMap::new()),
            entries: Signal::new(IndexMap::new()),
            registry: Signal::new(HashMap::new()),
        }
    }

    pub fn with_page<P>(&self, id: u64, f: impl FnOnce(&Page) -> P) -> Option<P> {
        self.pages.read().get(&id).map(f)
    }

    pub fn with_entry<E>(&self, key: EntryKey, f: impl FnOnce(&Entry) -> E) -> Option<E> {
        self.entries.read().get(&key).map(f)
    }

    pub fn apply(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Library(lib) => {
                self.pages.set(lib.pages);
                self.entries.set(lib.entries);
            }

            ServerMessage::Registry(registry) => {
                self.registry
                    .set(registry.0.into_iter().map(|d| (d.kind.clone(), d)).collect());
            }

            ServerMessage::Event { event } => self.apply_event(event),

            _ => {}
        }
    }

    

    

    

    

    pub fn apply_event(&mut self, event: Event) {
        match event {
            Event::PageCreated { page } => {
                self.pages.write().insert(page.id, page);
            }

            Event::PageUpdated { page } => {
                if let Some(existing) = self.pages.write().get_mut(&page.id) {
                    *existing = page;
                }
            }

            Event::PageDeleted { page_id } => {
                let removed = self.pages.write().shift_remove(&page_id);

                if let Some(page) = removed {
                    let mut entries = self.entries.write();

                    for key in &page.entries {
                        entries.shift_remove(key);
                    }
                }
            }

            Event::EntryCreated { entry } => {
                let key = entry.key;

                self.entries.write().insert(key, entry);

                if let Some(page) = self.pages.write().get_mut(&key.page_id()) {
                    if !page.entries.contains(&key) {
                        page.entries.push(key);
                    }
                }
            }

            Event::EntryUpdated { entry } => {
                if let Some(existing) = self.entries.write().get_mut(&entry.key) {
                    *existing = entry;
                }
            }

            Event::EntryDeleted { key } => {
                self.entries.write().shift_remove(&key);

                for page in self.pages.write().values_mut() {
                    page.entries.retain(|k| *k != key);
                }
            }

            Event::EntryMoved { key, page_id } => {
                let mut pages = self.pages.write();

                if !pages.contains_key(&page_id) {
                    return;
                }

                for page in pages.values_mut() {
                    page.entries.retain(|k| *k != key);
                }

                if let Some(page) = pages.get_mut(&page_id) {
                    page.entries.push(key);
                }
            }

            Event::Pushed => {}
        }
    }
}
