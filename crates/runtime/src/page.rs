use crate::Entry;
use model::PageData;

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

pub(crate) trait StoredEntry: Debug + Send + Sync {
    fn entry(&self) -> Arc<dyn Entry>;

    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub(crate) struct TypedEntry<E: Entry> {
    pub(crate) entry: Arc<E>,
}

impl<E: Entry> StoredEntry for TypedEntry<E> {
    fn entry(&self) -> Arc<dyn Entry> {
        self.entry.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Page {
    data: PageData,
    entries: HashMap<String, Arc<dyn StoredEntry>>,
}

impl Page {
    #[inline]
    pub fn new(data: PageData) -> Self {
        Self {
            data,
            entries: HashMap::new(),
        }
    }

    #[inline]
    pub fn data(&self) -> &PageData {
        &self.data
    }

    #[inline]
    pub fn id(&self) -> &str {
        self.data.id()
    }

    pub fn add_entry<E>(&mut self, entry: E)
    where
        E: Entry,
    {
        let id = entry.id().to_owned();

        self.entries.insert(
            id,
            Arc::new(TypedEntry {
                entry: Arc::new(entry),
            }),
        );
    }

    pub(crate) fn stored_entries(
        &self,
    ) -> impl Iterator<Item = (&str, &Arc<dyn StoredEntry>)> {
        self.entries
            .iter()
            .map(|(id, entry)| (id.as_str(), entry))
    }

    pub fn entries(
        &self,
    ) -> impl Iterator<Item = Arc<dyn Entry>> + '_ {
        self.entries
        .values()
        .map(|e| e.entry())
    }
    
    pub fn entry(
        &self,
        id: &str,
    ) -> Option<Arc<dyn Entry>> {
        self.entries
            .get(id)
            .map(|entry| entry.entry())
    }

    pub fn find<E>(
        &self,
        id: &str,
    ) -> Option<Arc<E>>
    where
        E: Entry,
    {
        self.entries
            .get(id)?
            .as_any()
            .downcast_ref::<TypedEntry<E>>()
            .map(|entry| Arc::clone(&entry.entry))
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}