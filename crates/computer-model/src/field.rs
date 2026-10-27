use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub kind: FieldKind,
    pub flags: Vec<(String, String)>,
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
