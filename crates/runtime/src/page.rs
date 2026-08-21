use crate::Entry;

use model::PageData;

use std::{
    any::Any,
    collections::HashMap,
    fmt::Debug,
    sync::Arc,
};

pub(crate) trait StoredEntry: Debug + Send + Sync {
    fn as_entry(&self) -> Arc<dyn Entry>;

    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub(crate) struct TypedEntry<E: Entry> {
    pub entry: Arc<E>,
}

impl<E: Entry> TypedEntry<E> {
    #[inline]
    fn get(&self) -> Arc<E> {
        Arc::clone(&self.entry)
    }
}

impl<E: Entry> StoredEntry for TypedEntry<E> {
    #[inline]
    fn as_entry(&self) -> Arc<dyn Entry> {
        self.entry.clone()
    }

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Page {
    id: String,
    data: PageData,
    entries: HashMap<String, Arc<dyn StoredEntry>>,
}

impl Page {
    #[inline]
    pub fn new(
        id: impl Into<String>,
        data: PageData,
    ) -> Self {
        Self {
            id: id.into(),
            data,
            entries: HashMap::new(),
        }
    }

    #[inline]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[inline]
    pub fn data(&self) -> &PageData {
        &self.data
    }

    pub fn insert<E>(
        &mut self,
        entry: E,
    ) -> Option<Arc<dyn Entry>>
    where
        E: Entry,
    {
        let id = entry.id().to_owned();

        self.entries
            .insert(
                id,
                Arc::new(TypedEntry {
                    entry: Arc::new(entry),
                }),
            )
            .map(|entry| entry.as_entry())
    }

    #[inline]
    pub fn get(
        &self,
        id: &str,
    ) -> Option<Arc<dyn Entry>> {
        self.entries
            .get(id)
            .map(|entry| entry.as_entry())
    }

    #[inline]
    pub fn get_typed<E>(
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
            .map(TypedEntry::get)
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&str, Arc<dyn Entry>)> + '_ {
        self.entries
            .iter()
            .map(|(id, entry)| {
                (
                    id.as_str(),
                    entry.as_entry(),
                )
            })
    }

    pub(crate) fn stored_entries(
        &self,
    ) -> impl Iterator<
        Item = (&str, &Arc<dyn StoredEntry>),
    > {
        self.entries
            .iter()
            .map(|(id, entry)| {
                (
                    id.as_str(),
                    entry,
                )
            })
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