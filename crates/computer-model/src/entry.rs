use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{field::FieldDefinition, id::EntryId, value::Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntryDefinition {
    pub kind: String,
    pub tags: Vec<String>,
    pub fields: Vec<FieldDefinition>,
    pub icon: String,
    pub color: String,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub id: EntryId,
    pub kind: String,
    pub fields: IndexMap<String, Value>,
    pub version: u32,
}

impl Entry {
    pub fn label(&self) -> &str {
        match self.fields.get("name") {
            Some(Value::String(name)) if !name.is_empty() => name,
            _ => &self.kind,
        }
    }
}
