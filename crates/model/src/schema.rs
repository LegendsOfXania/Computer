use std::collections::BTreeMap;
use crate::{Number, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Schema {
    Null,
    Text,
    Integer,
    Float,
    Boolean,
    Enumeration { values: Vec<String> },
    Reference(ReferenceSchema),
    Struct(StructSchema),
    List(Box<Schema>),
}

impl Schema {
    #[inline]
    pub fn enumeration(values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Enumeration {
            values: values.into_iter().map(Into::into).collect(),
        }
    }

    #[inline]
    pub fn reference() -> ReferenceSchema {
        ReferenceSchema::default()
    }

    #[inline]
    pub fn structure(fields: impl IntoIterator<Item = Field>) -> Self {
        Self::Struct(StructSchema::new(fields))
    }

    #[inline]
    pub fn list(element: impl Into<Schema>) -> Self {
        Self::List(Box::new(element.into()))
    }

    pub fn accepts(&self, value: &Value) -> bool {
        match (self, value) {
            (Self::Null, Value::Null)
            | (Self::Text, Value::Text(_))
            | (Self::Integer, Value::Number(Number::Integer(_)))
            | (Self::Float, Value::Number(Number::Float(_)))
            | (Self::Boolean, Value::Boolean(_))
            | (Self::Reference(_), Value::Reference(_)) => true,

            (Self::Enumeration { values }, Value::Enum(val)) => values.contains(val),

            (Self::Struct(schema), Value::Struct(values)) => schema.accepts(values),

            (Self::List(element), Value::List(values)) => {
                values.iter().all(|val| element.accepts(val))
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
    #[inline]
    pub fn with_entry_type(mut self, entry_type: impl Into<String>) -> Self {
        self.entry_type = Some(entry_type.into());
        self
    }

    #[inline]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    #[inline]
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(Into::into));
        self
    }

    #[inline]
    pub fn entry_type(&self) -> Option<&str> {
        self.entry_type.as_deref()
    }

    #[inline]
    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}

impl From<ReferenceSchema> for Schema {
    #[inline]
    fn from(schema: ReferenceSchema) -> Self {
        Self::Reference(schema)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StructSchema {
    fields: Vec<Field>,
}

impl StructSchema {
    #[inline]
    pub fn new(fields: impl IntoIterator<Item = Field>) -> Self {
        Self {
            fields: fields.into_iter().collect(),
        }
    }

    #[inline]
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    #[inline]
    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }

    fn accepts(&self, values: &BTreeMap<String, Value>) -> bool {
        if values.keys().any(|k| self.field(k).is_none()) {
            return false;
        }

        self.fields.iter().all(|field| match values.get(field.name()) {
            Some(value) => field.schema().accepts(value),
            None => !field.is_required(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    name: String,
    schema: Schema,
    required: bool,
}

impl Field {
    #[inline]
    pub fn new(name: impl Into<String>, schema: impl Into<Schema>) -> Self {
        Self {
            name: name.into(),
            schema: schema.into(),
            required: true,
        }
    }

    #[inline]
    pub fn optional(name: impl Into<String>, schema: impl Into<Schema>) -> Self {
        Self {
            name: name.into(),
            schema: schema.into(),
            required: false,
        }
    }

    #[inline] pub fn name(&self) -> &str { &self.name }
    #[inline] pub fn schema(&self) -> &Schema { &self.schema }
    #[inline] pub const fn is_required(&self) -> bool { self.required }
}

pub trait Fields {
    fn fields() -> Vec<Field>;
}

#[inline]
pub fn combine_fields(parts: impl IntoIterator<Item = Vec<Field>>) -> Vec<Field> {
    parts.into_iter().flatten().collect()
}