use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Text(String),
    Number(Number),
    Boolean(bool),
    Enum(String),
    Reference(String),
    Struct(BTreeMap<String, Value>),
    List(Vec<Value>),
}

impl Value {
    pub const NULL: Self = Self::Null;

    #[inline]
    pub fn text(value: impl Into<String>) -> Self { Self::Text(value.into()) }
    #[inline]
    pub const fn integer(value: i64) -> Self { Self::Number(Number::Integer(value)) }
    #[inline]
    pub const fn float(value: f64) -> Self { Self::Number(Number::Float(value)) }
    #[inline]
    pub const fn boolean(value: bool) -> Self { Self::Boolean(value) }
    #[inline]
    pub fn enumeration(value: impl Into<String>) -> Self { Self::Enum(value.into()) }
    #[inline]
    pub fn reference(value: impl Into<String>) -> Self { Self::Reference(value.into()) }
    #[inline]
    pub fn structure(values: BTreeMap<String, Self>) -> Self { Self::Struct(values) }
    #[inline]
    pub fn list(values: Vec<Self>) -> Self { Self::List(values) }

    #[inline]
    pub const fn is_null(&self) -> bool { matches!(self, Self::Null) }
    #[inline]
    pub fn as_text(&self) -> Option<&str> { if let Self::Text(value) = self { Some(value) } else { None } }
    #[inline]
    pub const fn as_number(&self) -> Option<Number> { if let Self::Number(value) = self { Some(*value) } else { None } }
    #[inline]
    pub const fn as_boolean(&self) -> Option<bool> { if let Self::Boolean(value) = self { Some(*value) } else { None } }
    #[inline]
    pub fn as_enum(&self) -> Option<&str> { if let Self::Enum(value) = self { Some(value) } else { None } }
    #[inline]
    pub fn as_reference(&self) -> Option<&str> { if let Self::Reference(value) = self { Some(value) } else { None } }
    #[inline]
    pub fn as_struct(&self) -> Option<&BTreeMap<String, Self>> { if let Self::Struct(value) = self { Some(value) } else { None } }
    #[inline]
    pub fn as_struct_mut(&mut self) -> Option<&mut BTreeMap<String, Self>> { if let Self::Struct(value) = self { Some(value) } else { None } }
    #[inline]
    pub fn as_list(&self) -> Option<&[Self]> { if let Self::List(value) = self { Some(value) } else { None } }
    #[inline]
    pub fn as_list_mut(&mut self) -> Option<&mut Vec<Self>> { if let Self::List(value) = self { Some(value) } else { None } }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64),
}
