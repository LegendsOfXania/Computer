use super::{Entry, EntryKey};
use crate::page::{Page, StoredEntry, TypedEntry};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Default)]
pub struct EntryStore {
    entries: RwLock<HashMap<EntryKey, Arc<dyn StoredEntry>>>,
}

impl EntryStore {
    #[inline]
    pub fn new() -> Self { Self::default() }

    pub fn insert<E>(&self, key: EntryKey, entry: E)
    where
        E: Entry,
    {
        self.insert_typed(key, Arc::new(entry));
    }

    pub fn insert_typed<E>(&self, key: EntryKey, entry: Arc<E>)
    where
        E: Entry,
    {
        self.entries
            .write()
            .expect("EntryStore lock poisoned")
            .insert(key, Arc::new(TypedEntry { entry }));
    }

    pub fn add_page(&self, page: &Page) {
        let page_id = page.id().to_owned();
        let mut entries = self.entries.write().expect("EntryStore lock poisoned");

        for (id, entry) in page.stored_entries() {
            entries.insert(EntryKey::new(&page_id, id), Arc::clone(entry));
        }
    }

    pub fn remove_page(&self, page: &Page) {
        let page_id = page.id().to_owned();
        let mut entries = self.entries.write().expect("EntryStore lock poisoned");

        for (id, entry) in page.stored_entries() {
            let key = EntryKey::new(&page_id, id);

            if entries.get(&key).is_some_and(|current| Arc::ptr_eq(current, entry)) {
                entries.remove(&key);
            }
        }
    }

    #[inline]
    pub fn remove(&self, key: &EntryKey) -> Option<Arc<dyn Entry>> {
        self.entries
            .write()
            .expect("EntryStore lock poisoned")
            .remove(key)
            .map(|entry| entry.entry())
    }

    pub fn find<E>(&self, key: &EntryKey) -> Option<Arc<E>>
    where
        E: Entry,
    {
        let entry = self
            .entries
            .read()
            .expect("EntryStore lock poisoned")
            .get(key)
            .cloned()?;

        entry
            .as_any()
            .downcast_ref::<TypedEntry<E>>()
            .map(|entry| Arc::clone(&entry.entry))
    }

    #[inline]
    pub fn get(&self, key: &EntryKey) -> Option<Arc<dyn Entry>> {
        self.entries
            .read()
            .expect("EntryStore lock poisoned")
            .get(key)
            .map(|entry| entry.entry())
    }

    #[inline]
    pub fn contains(&self, key: &EntryKey) -> bool {
        self.entries
            .read()
            .expect("EntryStore lock poisoned")
            .contains_key(key)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.read().expect("EntryStore lock poisoned").len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries
            .read()
            .expect("EntryStore lock poisoned")
            .is_empty()
    }

    pub fn clear(&self) {
        self.entries
            .write()
            .expect("EntryStore lock poisoned")
            .clear();
    }
}
