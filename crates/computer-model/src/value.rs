use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

use crate::id::EntryId;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
    #[default]
    Null,
}

impl Value {
    pub fn walk(&self, visit: &mut impl FnMut(&Self)) {
        visit(self);

        match self {
            Self::Struct(fields) | Self::Variant { fields, .. } => {
                fields.values().for_each(|value| value.walk(visit));
            }
            Self::List(items) => items.iter().for_each(|value| value.walk(visit)),
            Self::Map(pairs) => pairs.iter().for_each(|(key, value)| {
                key.walk(visit);
                value.walk(visit);
            }),
            _ => {}
        }
    }

    pub fn walk_mut(&mut self, visit: &mut impl FnMut(&mut Self)) {
        visit(self);

        match self {
            Self::Struct(fields) | Self::Variant { fields, .. } => {
                fields.values_mut().for_each(|value| value.walk_mut(visit));
            }
            Self::List(items) => items.iter_mut().for_each(|value| value.walk_mut(visit)),
            Self::Map(pairs) => pairs.iter_mut().for_each(|(key, value)| {
                key.walk_mut(visit);
                value.walk_mut(visit);
            }),
            _ => {}
        }
    }

    #[must_use]
    pub fn references(&self) -> IndexSet<EntryId> {
        let mut found = IndexSet::new();

        self.walk(&mut |value| {
            if let Self::Reference(id) = value {
                found.insert(*id);
            }
        });

        found
    }
}

macro_rules! convert {
    ($($ty:ty => $variant:ident),+ $(,)?) => {$(
        impl From<$ty> for Value {
            fn from(value: $ty) -> Self {
                Self::$variant(value)
            }
        }

        impl TryFrom<Value> for $ty {
            type Error = Value;

            fn try_from(value: Value) -> Result<Self, Value> {
                match value {
                    Value::$variant(inner) => Ok(inner),
                    other => Err(other),
                }
            }
        }
    )+};
}

convert!(
    i64 => Int,
    f64 => Float,
    bool => Bool,
    EntryId => Reference,
    Vec<Value> => List,
    Vec<(Value, Value)> => Map,
);

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl TryFrom<Value> for String {
    type Error = Value;

    fn try_from(value: Value) -> Result<Self, Value> {
        match value {
            Value::String(text) | Value::Enum(text) => Ok(text),
            other => Err(other),
        }
    }
}

impl From<IndexMap<String, Value>> for Value {
    fn from(fields: IndexMap<String, Value>) -> Self {
        Self::Struct(fields)
    }
}

impl TryFrom<Value> for IndexMap<String, Value> {
    type Error = Value;

    fn try_from(value: Value) -> Result<Self, Value> {
        match value {
            Value::Struct(fields) | Value::Variant { fields, .. } => Ok(fields),
            other => Err(other),
        }
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}