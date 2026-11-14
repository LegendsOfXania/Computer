use std::str::FromStr;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::value::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub kind: FieldKind,
    pub flags: Vec<(String, String)>,
}

impl FieldDefinition {
    #[must_use]
    pub fn new(name: String, kind: FieldKind) -> Self {
        Self {
            name,
            kind,
            flags: Vec::new(),
        }
    }

    #[must_use]
    pub fn flag(&self, key: &str) -> Option<&str> {
        self.flags
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }

    #[must_use]
    pub fn has_flag(&self, key: &str) -> bool {
        self.flag(key).is_some_and(|value| value != "false")
    }

    #[must_use]
    pub fn bound<T: FromStr>(&self, key: &str) -> Option<T> {
        self.flag(key)?.parse().ok()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldKind {
    String,
    Int,
    Float,
    Bool,
    Reference,
    Enum(Vec<String>),
    Struct(Vec<FieldDefinition>),
    List(Box<Self>),
    Map { key: Box<Self>, value: Box<Self> },
    Variant(Vec<Variant>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
}

pub(crate) fn defaults(fields: &[FieldDefinition]) -> IndexMap<String, Value> {
    fields
        .iter()
        .map(|field| (field.name.clone(), Value::from(&field.kind)))
        .collect()
}

impl From<&FieldKind> for Value {
    fn from(kind: &FieldKind) -> Self {
        match kind {
            FieldKind::String => Self::String(String::new()),
            FieldKind::Int => Self::Int(0),
            FieldKind::Float => Self::Float(0.0),
            FieldKind::Bool => Self::Bool(false),
            FieldKind::Reference => Self::Null,
            FieldKind::Enum(options) => Self::Enum(options.first().cloned().unwrap_or_default()),
            FieldKind::Struct(fields) => Self::Struct(defaults(fields)),
            FieldKind::List(_) => Self::List(Vec::new()),
            FieldKind::Map { .. } => Self::Map(Vec::new()),
            FieldKind::Variant(variants) => variants.first().map_or(Self::Null, Self::from),
        }
    }
}

impl From<&Variant> for Value {
    fn from(variant: &Variant) -> Self {
        Self::Variant {
            name: variant.name.clone(),
            fields: defaults(&variant.fields),
        }
    }
}