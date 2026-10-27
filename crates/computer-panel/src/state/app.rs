use indexmap::IndexMap;
use std::collections::HashMap;

use computer_model::{
    entry::{Entry, EntryDefinition},
    id::{EntryId, PageId},
    page::Page,
    protocol::{event::Event, message::ServerMessage},
};
use dioxus::signals::{ReadableExt, Signal, WritableExt};

#[derive(Clone, Copy)]
pub struct AppState {
    pub pages: Signal<IndexMap<PageId, Page>>,
    pub entries: Signal<IndexMap<EntryId, Entry>>,
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

    pub fn with_page<P>(&self, id: &PageId, f: impl FnOnce(&Page) -> P) -> Option<P> {
        self.pages.read().get(id).map(f)
    }

    pub fn with_entry<E>(&self, id: &EntryId, f: impl FnOnce(&Entry) -> E) -> Option<E> {
        self.entries.read().get(id).map(f)
    }

    pub fn apply(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Library(lib) => {
                self.pages.set(lib.pages);
                self.entries.set(lib.entries);
            }

            ServerMessage::Registry(registry) => {
                self.registry.set(
                    registry
                        .0
                        .into_iter()
                        .map(|definition| (definition.kind.clone(), definition))
                        .collect(),
                );
            }

            ServerMessage::Event { event } => self.apply_event(event),

            _ => {}
        }
    }

    pub fn apply_event(&mut self, event: Event) {
        match event {
            Event::PageCreated { page } => {
                let id = page.id.clone();
                self.pages.write().insert(id, page);
            }

            Event::PageUpdated { page } => {
                let id = page.id.clone();

                if let Some(existing) = self.pages.write().get_mut(&id) {
                    *existing = page;
                }
            }

            Event::PageDeleted { page_id } => {
                let removed = self.pages.write().shift_remove(&page_id);

                if let Some(page) = removed {
                    let mut entries = self.entries.write();

                    for id in &page.entries {
                        entries.shift_remove(id);
                    }
                }
            }

            Event::EntryCreated { entry, page_id } => {
                let id = entry.id.clone();
                let mut pages = self.pages.write();

                if !pages.contains_key(&page_id) {
                    return;
                }

                self.entries.write().insert(id.clone(), entry);

                if let Some(page) = pages.get_mut(&page_id) {
                    if !page.entries.contains(&id) {
                        page.entries.push(id);
                    }
                }
            }

            Event::EntryUpdated { entry } => {
                let id = entry.id.clone();

                if let Some(existing) = self.entries.write().get_mut(&id) {
                    *existing = entry;
                }
            }

            Event::EntryDeleted { entry_id } => {
                self.entries.write().shift_remove(&entry_id);

                for page in self.pages.write().values_mut() {
                    page.entries.retain(|id| id != &entry_id);
                }
            }

            Event::EntryMoved { entry_id, page_id } => {
                let mut pages = self.pages.write();

                if !pages.contains_key(&page_id) {
                    return;
                }

                for page in pages.values_mut() {
                    page.entries.retain(|id| id != &entry_id);
                }

                if let Some(page) = pages.get_mut(&page_id) {
                    page.entries.push(entry_id);
                }
            }

            Event::Pushed => {}
        }
    }
}
