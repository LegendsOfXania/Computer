use crate::Value;

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct EntryData {
    entry_type: String,
    fields: BTreeMap<String, Value>,
}

impl EntryData {
    #[inline]
    pub fn new(
        entry_type: impl Into<String>,
        fields: BTreeMap<String, Value>,
    ) -> Self {
        Self {
            entry_type: entry_type.into(),
            fields,
        }
    }

    #[inline]
    pub fn empty(entry_type: impl Into<String>) -> Self {
        Self::new(entry_type, BTreeMap::new())
    }

    #[inline]
    pub fn entry_type(&self) -> &str {
        &self.entry_type
    }

    #[inline]
    pub fn fields(&self) -> &BTreeMap<String, Value> {
        &self.fields
    }

    #[inline]
    pub fn fields_mut(&mut self) -> &mut BTreeMap<String, Value> {
        &mut self.fields
    }

    #[inline]
    pub fn field(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }
}
