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
    #[inline]
    pub fn global() -> &'static Self { &LIBRARY }
    #[inline]
    pub fn new() -> Self { Self::default() }

    pub fn add_page(&self, page: Page) {
        let page = Arc::new(page);
        let previous = self.pages.write().expect("Library pages lock poisoned").insert(page.id().to_owned(), Arc::clone(&page));
        if let Some(previous) = previous { self.entries.remove_page(&previous); }
        self.entries.add_page(&page);
    }

    pub fn remove_page(&self, id: &str) -> Option<Arc<Page>> {
        let page = self.pages.write().expect("Library pages lock poisoned").remove(id)?;
        self.entries.remove_page(&page);
        Some(page)
    }

    #[inline]
    pub fn page(&self, id: &str) -> Option<Arc<Page>> { self.pages.read().expect("Library pages lock poisoned").get(id).cloned() }
    #[inline]
    pub fn pages(&self) -> Vec<Arc<Page>> { self.pages.read().expect("Library pages lock poisoned").values().cloned().collect()}
    #[inline]
    pub fn contains_page(&self, id: &str) -> bool { self.pages.read().expect("Library pages lock poisoned").contains_key(id) }
    #[inline]
    pub fn len(&self) -> usize { self.pages.read().expect("Library pages lock poisoned").len() }
    #[inline]
    pub fn is_empty(&self) -> bool { self.pages.read().expect("Library pages lock poisoned").is_empty() }
    #[inline]
    pub fn entries(&self) -> &EntryStore { &self.entries }
    #[inline]
    pub fn find<E>(&self, key: &EntryKey) -> Option<Arc<E>> where E: Entry { self.entries.find(key) }
    pub fn clear(&self) {
        self.pages.write().expect("Library pages lock poisoned").clear();
        self.entries.clear();
    }
}
