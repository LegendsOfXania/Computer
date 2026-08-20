use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Clone, Eq, Ord, PartialOrd)]
pub struct EntryKey {
    page_id: String,
    entry_id: String,
}

impl EntryKey {
    #[inline]
    pub fn new(page_id: impl Into<String>, entry_id: impl Into<String>) -> Self {
        Self { page_id: page_id.into(), entry_id: entry_id.into() }
    }
    #[inline]
    pub fn page_id(&self) -> &str { &self.page_id }
    #[inline]
    pub fn entry_id(&self) -> &str { &self.entry_id }

    pub fn parse(value: &str) -> Option<Self> {
        let (page_id, entry_id) = value.split_once(':')?;
        (!page_id.is_empty() && !entry_id.is_empty()).then(|| Self::new(page_id, entry_id))
    }
}

impl PartialEq for EntryKey {
    fn eq(&self, other: &Self) -> bool { self.page_id == other.page_id && self.entry_id == other.entry_id }
}
impl Hash for EntryKey {
    fn hash<H: Hasher>(&self, state: &mut H) { self.page_id.hash(state); self.entry_id.hash(state); }
}
impl fmt::Debug for EntryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.debug_tuple("EntryKey").field(&self.to_string()).finish() }
}
impl fmt::Display for EntryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}:{}", self.page_id, self.entry_id) }
}
