use crate::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct EntryData {
    entry_type: String,
    fields: BTreeMap<String, Value>,
}

impl EntryData {
    pub fn new(entry_type: impl Into<String>, fields: BTreeMap<String, Value>) -> Self {
        Self {
            entry_type: entry_type.into(),
            fields,
        }
    }

    pub fn empty(entry_type: impl Into<String>) -> Self {
        Self::new(entry_type, BTreeMap::new())
    }

    pub fn entry_type(&self) -> &str {
        &self.entry_type
    }
    pub fn fields(&self) -> &BTreeMap<String, Value> {
        &self.fields
    }
    pub fn fields_mut(&mut self) -> &mut BTreeMap<String, Value> {
        &mut self.fields
    }
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.fields.get_mut(name)
    }
}
