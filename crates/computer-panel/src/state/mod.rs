use std::collections::HashMap;

use computer_model::{entry::{Entry, EntryDefinition}, key::EntryKey, page::Page, protocol::{event::Event, message::ServerMessage}};
use dioxus::signals::{Signal, WritableExt};

pub mod connection;
pub mod ui;

#[derive(Clone, Copy)]
pub struct AppState {
    pub pages: Signal<HashMap<u64, Page>>,
    pub entries: Signal<HashMap<EntryKey, Entry>>,
    pub registry: Signal<HashMap<String, EntryDefinition>>,
}

impl AppState {
    pub fn new() -> Self {
        Self { 
            pages: Signal::new(HashMap::new()), 
            entries: Signal::new(HashMap::new()), 
            registry: Signal::new(HashMap::new()), 
        }
    }

    pub fn apply(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Library(lib) => {
                self.pages.set(lib.pages.into_iter().map(|p| (p.id, p)).collect());
                self.entries.set(lib.entries.into_iter().map(|e| (e.key, e)).collect());
            }

            ServerMessage::Registry(registry) => {
                self.registry.set(registry.0.into_iter().map(|d| (d.kind.clone(), d)).collect());
            }

            ServerMessage::Event { event } => self.apply_event(event),

            _ => {}
        }
    }

    pub fn apply_event(&mut self, event: Event) {
        match event {
            Event::PageCreated { page } | Event::PageUpdated { page } => {
                self.pages.write().insert(page.id, page);
            }

            Event::PageDeleted { page_id } => {
                self.pages.write().remove(&page_id);
            }

            Event::EntryCreated { entry } | Event::EntryUpdated { entry } => {
                self.entries.write().insert(entry.key, entry);
            }

            Event::EntryDeleted { key } => {
                self.entries.write().remove(&key);
            }

            Event::EntryMoved { key, page_id } => {
                for page in self.pages.write().values_mut() {
                    page.entries.retain(|k| *k != key);
                }
                if let Some(page) = self.pages.write().get_mut(&page_id) {
                    page.entries.push(key);
                }
            }

            Event::Published => {}
        }
    }
}