use std::{cmp::PartialOrd, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::value::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub kind: FieldKind,
    pub flags: Vec<(String, String)>,
}

impl FieldDefinition {
    pub fn new(name: String, kind: FieldKind) -> Self {
        Self {
            name,
            kind,
            flags: Vec::new(),
        }
    }

    pub fn flag(&self, key: &str) -> Option<&str> {
        self.flags
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }

    pub fn has_flag(&self, key: &str) -> bool {
        self.flag(key).is_some_and(|v| v != "false")
    }

    pub fn bound<T: FromStr>(&self, key: &str) -> Option<T> {
        self.flag(key)?.parse().ok()
    }

    pub fn clamp<T: PartialOrd>(&self, value: T, min: Option<T>, max: Option<T>) -> T {
        let value = match min {
            Some(min) if value < min => min,
            _ => value,
        };

        match max {
            Some(max) if value > max => max,
            _ => value,
        }
    }

    pub fn default_value(&self) -> Value {
        match &self.kind {
            FieldKind::String => Value::String(String::new()),
            FieldKind::Int => Value::Int(self.clamp(0_i64, self.bound("min"), self.bound("max"))),
            FieldKind::Float => {
                Value::Float(self.clamp(0.0_f64, self.bound("min"), self.bound("max")))
            }
            FieldKind::Bool => Value::Bool(false),
            FieldKind::Reference => Value::Null,
            FieldKind::Enum(options) => Value::Enum(options.first().cloned().unwrap_or_default()),
            FieldKind::Struct(fields) => Value::Struct(
                fields
                    .iter()
                    .map(|field| (field.name.clone(), field.default_value()))
                    .collect(),
            ),
            FieldKind::List(_) => Value::List(Vec::new()),
            FieldKind::Map { .. } => Value::Map(Vec::new()),
            FieldKind::Variant(variants) => {
                variants.first().map_or(Value::Null, Variant::default_value)
            }
        }
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
impl Variant {
    pub fn default_value(&self) -> Value {
        Value::Variant {
            name: self.name.clone(),
            fields: self
                .fields
                .iter()
                .map(|field| (field.name.clone(), field.default_value()))
                .collect(),
        }
    }
}
