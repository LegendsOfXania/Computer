use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Number(Number),
    Boolean(bool),
    Text(String),
    Enum(String),
    Reference(String),
    Struct(BTreeMap<String, Value>),
    List(Vec<Value>),
}

impl Value {
    pub const NULL: Self = Self::Null;
    pub const fn integer(value: i64) -> Self {
        Self::Number(Number::Integer(value))
    }
    pub const fn float(value: f64) -> Self {
        Self::Number(Number::Float(value))
    }
    pub const fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }
    pub fn enumeration(value: impl Into<String>) -> Self {
        Self::Enum(value.into())
    }
    pub fn reference(value: impl Into<String>) -> Self {
        Self::Reference(value.into())
    }
    pub fn structure(values: BTreeMap<String, Self>) -> Self {
        Self::Struct(values)
    }
    pub fn list(values: Vec<Self>) -> Self {
        Self::List(values)
    }

    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub const fn as_number(&self) -> Option<Number> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }
    pub const fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            _ => None,
        }
    }
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }
    pub fn as_enum(&self) -> Option<&str> {
        match self {
            Self::Enum(value) => Some(value),
            _ => None,
        }
    }
    pub fn as_reference(&self) -> Option<&str> {
        match self {
            Self::Reference(value) => Some(value),
            _ => None,
        }
    }
    pub fn as_struct(&self) -> Option<&BTreeMap<String, Self>> {
        match self {
            Self::Struct(value) => Some(value),
            _ => None,
        }
    }
    pub fn as_struct_mut(&mut self) -> Option<&mut BTreeMap<String, Self>> {
        match self {
            Self::Struct(value) => Some(value),
            _ => None,
        }
    }
    pub fn as_list(&self) -> Option<&[Self]> {
        match self {
            Self::List(value) => Some(value),
            _ => None,
        }
    }
    pub fn as_list_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            Self::List(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64),
}
