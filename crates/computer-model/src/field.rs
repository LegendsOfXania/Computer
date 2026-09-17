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
    Enum,
    Struct(Vec<FieldDefinition>),
    List(Box<FieldKind>),
    Map {
        key: Box<FieldKind>,
        value: Box<FieldKind>,
    },
    Variant(Vec<Case>)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Case {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
}
