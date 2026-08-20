use crate::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Schema {
    Null,
    Text,
    Integer,
    Float,
    Boolean,

    Enumeration {
        values: Vec<String>,
    },

    Reference(ReferenceSchema),

    Struct(StructSchema),

    List(Box<Schema>),
}

impl Schema {
    #[inline]
    pub fn enumeration(
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self::Enumeration {
            values: values
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }

    #[inline]
    pub fn reference() -> ReferenceSchema {
        ReferenceSchema::default()
    }

    #[inline]
    pub fn structure(
        fields: impl IntoIterator<Item = Field>,
    ) -> Self {
        Self::Struct(StructSchema::new(fields))
    }

    #[inline]
    pub fn list(element: Self) -> Self {
        Self::List(Box::new(element))
    }

    pub fn accepts(&self, value: &Value) -> bool {
        match (self, value) {
            (Self::Null, Value::Null)
            | (Self::Text, Value::Text(_))
            | (
                Self::Integer,
                Value::Number(crate::Number::Integer(_)),
            )
            | (
                Self::Float,
                Value::Number(crate::Number::Float(_)),
            )
            | (Self::Boolean, Value::Boolean(_))
            | (Self::Reference(_), Value::Reference(_)) => true,

            (
                Self::Enumeration { values },
                Value::Enum(value),
            ) => values
                .iter()
                .any(|candidate| candidate == value),

            (
                Self::Struct(schema),
                Value::Struct(values),
            ) => schema.accepts(values),

            (
                Self::List(element),
                Value::List(values),
            ) => values
                .iter()
                .all(|value| element.accepts(value)),

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
    pub fn entry_type(
        mut self,
        entry_type: impl Into<String>,
    ) -> Self {
        self.entry_type = Some(entry_type.into());

        self
    }

    #[inline]
    pub fn has_tag(
        mut self,
        tag: impl Into<String>,
    ) -> Self {
        self.tags.push(tag.into());

        self
    }

    #[inline]
    pub fn has_tags(
        mut self,
        tags: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.tags.extend(
            tags
                .into_iter()
                .map(Into::into),
        );

        self
    }

    #[inline]
    pub fn entry_type_name(&self) -> Option<&str> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    name: String,
    schema: Schema,
    required: bool,
}

impl Field {
    #[inline]
    pub fn new(
        name: impl Into<String>,
        schema: impl Into<Schema>,
    ) -> Self {
        Self {
            name: name.into(),
            schema: schema.into(),
            required: true,
        }
    }

    #[inline]
    pub fn optional(
        name: impl Into<String>,
        schema: impl Into<Schema>,
    ) -> Self {
        Self {
            name: name.into(),
            schema: schema.into(),
            required: false,
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    #[inline]
    pub const fn required(&self) -> bool {
        self.required
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StructSchema {
    fields: Vec<Field>,
}

impl StructSchema {
    #[inline]
    pub fn new(
        fields: impl IntoIterator<Item = Field>,
    ) -> Self {
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
        self.fields
            .iter()
            .find(|field| field.name == name)
    }

    fn accepts(
        &self,
        values: &std::collections::BTreeMap<String, Value>,
    ) -> bool {
        values
            .keys()
            .all(|name| self.field(name).is_some())
            && self.fields.iter().all(|field| {
                match values.get(field.name()) {
                    Some(value) => field.schema().accepts(value),
                    None => !field.required(),
                }
            })
    }
}

pub trait Fields {
    fn fields() -> Vec<Field>;
}

#[inline]
pub fn combine_fields<I>(parts: I) -> Vec<Field>
where
    I: IntoIterator<Item = Vec<Field>>,
{
    parts
        .into_iter()
        .flatten()
        .collect()
}