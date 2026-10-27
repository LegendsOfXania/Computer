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
