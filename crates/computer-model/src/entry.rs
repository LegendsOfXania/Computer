use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{field::FieldDefinition, key::EntryKey, value::Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntryDefinition {
    pub kind: String,
    pub version: u32,
    pub tags: Vec<String>,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub key: EntryKey,
    pub kind: String,
    pub version: u32,
    pub fields: IndexMap<String, Value>,
}