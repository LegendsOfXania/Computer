use std::fmt::Display;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    entry::Entry,
    id::{EntryId, PageId},
    page::Page,
    protocol::event::Event,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Library {
    pub pages: IndexMap<PageId, Page>,
    pub entries: IndexMap<EntryId, Entry>,
}

impl Library {
    pub fn pages_of(&self, entry: EntryId) -> impl Iterator<Item = &Page> {
        self.pages
            .values()
            .filter(move |page| page.entries.contains(&entry))
    }

    #[must_use]
    pub fn page_of(&self, entry: EntryId) -> Option<&Page> {
        self.pages_of(entry).next()
    }

    pub fn apply(&mut self, event: &Event) -> Result<Vec<PageId>, String> {
        match event {
            Event::PageCreated { page } => {
                if self.pages.contains_key(&page.id) {
                    return Err(format!("Page {} already exists", page.id));
                }

                let page = Page {
                    entries: Vec::new(),
                    ..page.clone()
                };

                let id = page.id;

                self.pages.insert(id, page);

                Ok(vec![id])
            }

            Event::PageUpdated { page } => {
                let existing = self
                    .pages
                    .get_mut(&page.id)
                    .ok_or_else(|| missing("Page", page.id, "have been deleted"))?;

                existing.name.clone_from(&page.name);
                existing.kind = page.kind;
                existing.priority = page.priority;
                existing.chapter.clone_from(&page.chapter);

                Ok(vec![page.id])
            }

            Event::PageDeleted { page_id } => {
                let page = self
                    .pages
                    .shift_remove(page_id)
                    .ok_or_else(|| missing("Page", page_id, "already be deleted"))?;

                for entry_id in &page.entries {
                    self.entries.shift_remove(entry_id);
                }

                Ok(vec![*page_id])
            }

            Event::EntryCreated { entry, page_id } => {
                if self.entries.contains_key(&entry.id) {
                    return Err(format!("Entry {} already exists", entry.id));
                }

                let page = self
                    .pages
                    .get_mut(page_id)
                    .ok_or_else(|| missing("Page", page_id, "have been deleted"))?;

                if !page.entries.contains(&entry.id) {
                    page.entries.push(entry.id);
                }

                self.entries.insert(entry.id, entry.clone());

                Ok(vec![*page_id])
            }

            Event::EntryUpdated { entry } => {
                let existing = self
                    .entries
                    .get_mut(&entry.id)
                    .ok_or_else(|| missing("Entry", entry.id, "have been deleted"))?;

                existing.clone_from(entry);

                Ok(self.pages_of(entry.id).map(|page| page.id).collect())
            }

            Event::EntryDeleted { entry_id } => {
                self.entries
                    .shift_remove(entry_id)
                    .ok_or_else(|| missing("Entry", entry_id, "already be deleted"))?;

                Ok(self.detach(*entry_id))
            }

            Event::EntryMoved { entry_id, page_id } => {
                if !self.entries.contains_key(entry_id) {
                    return Err(missing("Entry", entry_id, "have been deleted"));
                }

                if !self.pages.contains_key(page_id) {
                    return Err(missing("Page", page_id, "have been deleted"));
                }

                let mut touched = self.detach(*entry_id);

                if let Some(page) = self.pages.get_mut(page_id) {
                    page.entries.push(*entry_id);
                }

                if !touched.contains(page_id) {
                    touched.push(*page_id);
                }

                Ok(touched)
            }

            Event::Pushed => Ok(Vec::new()),
        }
    }

    fn detach(&mut self, entry: EntryId) -> Vec<PageId> {
        let mut touched = Vec::new();

        for page in self.pages.values_mut() {
            if page.entries.contains(&entry) {
                page.entries.retain(|id| *id != entry);
                touched.push(page.id);
            }
        }

        touched
    }
}

fn missing(what: &str, id: impl Display, state: &str) -> String {
    format!("{what} {id} not found: it may {state}")
}