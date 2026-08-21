use crate::{Entry, EntryKey, EntryStore, Page};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

#[derive(Debug, Default)]
pub struct Library {
    pages: RwLock<HashMap<String, Arc<Page>>>,
    entries: EntryStore,
}
static LIBRARY: LazyLock<Library> = LazyLock::new(Library::new);
impl Library {
    pub fn global() -> &'static Self {
        &LIBRARY
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, page: Page) -> Option<Arc<Page>> {
        let page = Arc::new(page);
        let previous = self
            .pages
            .write()
            .expect("Library pages lock poisoned")
            .insert(page.id().to_owned(), Arc::clone(&page));
        if let Some(previous) = &previous {
            self.entries.remove_page(previous);
        }
        self.entries.add_page(&page);
        previous
    }

    pub fn remove(&self, id: &str) -> Option<Arc<Page>> {
        let page = self
            .pages
            .write()
            .expect("Library pages lock poisoned")
            .remove(id)?;
        self.entries.remove_page(&page);
        Some(page)
    }

    pub fn page(&self, id: &str) -> Option<Arc<Page>> {
        self.pages
            .read()
            .expect("Library pages lock poisoned")
            .get(id)
            .cloned()
    }

    pub fn pages(&self) -> Vec<Arc<Page>> {
        self.pages
            .read()
            .expect("Library pages lock poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.pages
            .read()
            .expect("Library pages lock poisoned")
            .contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.pages
            .read()
            .expect("Library pages lock poisoned")
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.pages
            .read()
            .expect("Library pages lock poisoned")
            .is_empty()
    }

    pub fn entries(&self) -> &EntryStore {
        &self.entries
    }

    pub fn typed_entry<E: Entry>(&self, key: &EntryKey) -> Option<Arc<E>> {
        self.entries.get_typed(key)
    }

    pub fn clear(&self) {
        self.pages
            .write()
            .expect("Library pages lock poisoned")
            .clear();
        self.entries.clear();
    }
}
