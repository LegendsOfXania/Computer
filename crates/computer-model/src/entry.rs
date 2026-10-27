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
