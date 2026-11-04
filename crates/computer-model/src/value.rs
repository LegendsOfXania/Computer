use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::id::EntryId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Enum(String),
    Reference(EntryId),
    Struct(IndexMap<String, Self>),
    List(Vec<Self>),
    Map(Vec<(Self, Self)>),
    Variant {
        name: String,
        fields: IndexMap<String, Self>,
    },
    Null,
}

impl Value {
    pub fn into_text(self) -> String {
        match self {
            Self::String(value) | Self::Enum(value) => value,
            _ => String::new(),
        }
    }

    pub fn into_int(self) -> i64 {
        match self {
            Self::Int(value) => value,
            _ => 0,
        }
    }

    pub fn into_float(self) -> f64 {
        match self {
            Self::Float(value) => value,
            _ => 0.0,
        }
    }

    pub fn into_list(self) -> Vec<Self> {
        match self {
            Self::List(items) => items,
            _ => Vec::new(),
        }
    }

    pub fn into_map(self) -> Vec<(Self, Self)> {
        match self {
            Self::Map(pairs) => pairs,
            _ => Vec::new(),
        }
    }

    pub fn into_fields(self) -> IndexMap<String, Self> {
        match self {
            Self::Struct(fields) | Self::Variant { fields, .. } => fields,
            _ => IndexMap::new(),
        }
    }
}
