use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::key::EntryKey;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Enum(String),
    Reference(EntryKey),
    Struct(IndexMap<String, Self>),
    List(Vec<Self>),
    Map(Vec<(Self, Self)>),
    Variant {
        name: String,
        fields: IndexMap<String, Self>,
    },
}

impl Value {
    pub fn is_default_value(&self) -> bool {
        match self {
            Self::String(s) => s.is_empty(),
            Self::Int(i) => *i == 0,
            Self::Float(f) => *f == 0.0,
            Self::Bool(b) => !*b,
            Self::Reference(_) => false,
            Self::Enum(_) => false,
            Self::Struct(fields) => fields.is_empty(),
            Self::List(values) => values.is_empty(),
            Self::Map(pairs) => pairs.is_empty(),
            Self::Variant { fields, .. } => fields.is_empty(),
        }
    }
}