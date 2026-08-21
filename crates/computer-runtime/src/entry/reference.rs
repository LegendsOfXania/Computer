use super::{Entry, EntryKey};
use crate::Library;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Clone)]
pub struct Ref<E: Entry> {
    key: Option<EntryKey>,
    marker: PhantomData<fn() -> E>,
}
impl<E: Entry> Ref<E> {
    pub fn new(page_id: impl Into<String>, entry_id: impl Into<String>) -> Self {
        Self::from_key(EntryKey::new(page_id, entry_id))
    }
    pub fn from_key(key: EntryKey) -> Self {
        Self {
            key: Some(key),
            marker: PhantomData,
        }
    }
    pub fn empty() -> Self {
        Self {
            key: None,
            marker: PhantomData,
        }
    }
    pub fn is_set(&self) -> bool {
        self.key.is_some()
    }
    pub fn key(&self) -> Option<&EntryKey> {
        self.key.as_ref()
    }
    pub fn resolve(&self, library: &Library) -> Option<Arc<E>> {
        library.typed_entry(self.key.as_ref()?)
    }
    pub fn get(&self) -> Option<Arc<E>> {
        self.resolve(Library::global())
    }
}
impl<E: Entry> PartialEq for Ref<E> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<E: Entry> Eq for Ref<E> {}
impl<E: Entry> Hash for Ref<E> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}
impl<E: Entry> fmt::Debug for Ref<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ref")
            .field("type", &std::any::type_name::<E>())
            .field("key", &self.key)
            .finish()
    }
}
