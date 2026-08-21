use crate::{Number, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Schema {
    Null,
    Integer,
    Float,
    Boolean,
    Text,
    Enumeration { values: Vec<String> },
    Reference(ReferenceSchema),
    Struct(StructSchema),
    List(Box<Schema>),
}

impl Schema {
    pub fn enumeration(values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Enumeration {
            values: values.into_iter().map(Into::into).collect(),
        }
    }
    pub fn reference() -> ReferenceSchema {
        ReferenceSchema::default()
    }
    pub fn structure(fields: impl IntoIterator<Item = Field>) -> Self {
        Self::Struct(StructSchema::new(fields))
    }
    pub fn list(element: Self) -> Self {
        Self::List(Box::new(element))
    }
    pub fn accepts(&self, value: &Value) -> bool {
        match (self, value) {
            (Self::Null, Value::Null)
            | (Self::Text, Value::Text(_))
            | (Self::Integer, Value::Number(Number::Integer(_)))
            | (Self::Float, Value::Number(Number::Float(_)))
            | (Self::Boolean, Value::Boolean(_))
            | (Self::Reference(_), Value::Reference(_)) => true,
            (Self::Enumeration { values }, Value::Enum(value)) => {
                values.iter().any(|candidate| candidate == value)
            }
            (Self::Struct(schema), Value::Struct(values)) => schema.accepts(values),
            (Self::List(schema), Value::List(values)) => {
                values.iter().all(|value| schema.accepts(value))
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReferenceSchema {
    entry_type: Option<String>,
    tags: Vec<String>,
}
impl ReferenceSchema {
    pub fn with_entry_type(mut self, entry_type: impl Into<String>) -> Self {
        self.entry_type = Some(entry_type.into());
        self
    }
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(Into::into));
        self
    }
    pub fn entry_type(&self) -> Option<&str> {
        self.entry_type.as_deref()
    }
    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}
impl From<ReferenceSchema> for Schema {
    fn from(value: ReferenceSchema) -> Self {
        Self::Reference(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    name: String,
    schema: Schema,
    required: bool,
}
impl Field {
    pub fn new(name: impl Into<String>, schema: impl Into<Schema>) -> Self {
        Self {
            name: name.into(),
            schema: schema.into(),
            required: true,
        }
    }
    pub fn optional(name: impl Into<String>, schema: impl Into<Schema>) -> Self {
        Self {
            name: name.into(),
            schema: schema.into(),
            required: false,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn schema(&self) -> &Schema {
        &self.schema
    }
    pub const fn is_required(&self) -> bool {
        self.required
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StructSchema {
    fields: Vec<Field>,
}
impl StructSchema {
    pub fn new(fields: impl IntoIterator<Item = Field>) -> Self {
        Self {
            fields: fields.into_iter().collect(),
        }
    }
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }
    pub fn get(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|field| field.name == name)
    }
    fn accepts(&self, values: &BTreeMap<String, Value>) -> bool {
        values.keys().all(|name| self.get(name).is_some())
            && self
                .fields
                .iter()
                .all(|field| match values.get(field.name()) {
                    Some(value) => field.schema().accepts(value),
                    None => !field.is_required(),
                })
    }
}

pub trait Fields {
    fn fields() -> Vec<Field>;
}
