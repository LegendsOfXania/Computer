use crate::Value;

/// Describes the shape of a value without storing the value itself.
///
/// `Value` contains runtime data. `Schema` describes which `Value` instances
/// are valid for a field. In particular, `Schema::Struct` maps directly to a
/// Rust struct and `Schema::List` maps to a `Vec<T>`-like value.
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

    Reference {
        tags: Vec<String>,
    },

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
    pub fn reference(
        tags: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self::Reference {
            tags: tags
                .into_iter()
                .map(Into::into)
                .collect(),
        }
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
            | (Self::Boolean, Value::Boolean(_)) => true,

            (
                Self::Enumeration { values },
                Value::Enum(value),
            ) => {
                values
                    .iter()
                    .any(|candidate| candidate == value)
            }

            (Self::Reference { .. }, Value::Reference(_)) => true,

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
        schema: Schema,
    ) -> Self {
        Self {
            name: name.into(),
            schema,
            required: true,
        }
    }

    #[inline]
    pub fn optional(
        name: impl Into<String>,
        schema: Schema,
    ) -> Self {
        Self {
            name: name.into(),
            schema,
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