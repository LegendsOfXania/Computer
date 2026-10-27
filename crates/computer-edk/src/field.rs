use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use computer_model::{
    field::{FieldDefinition, FieldKind},
    id::EntryId,
    value::Value,
};

#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as an entry field",
    label = "this type does not implement `Field`",
    note = "add `#[derive(Field)]` to your struct/enum, or use a supported type: String, bool, i8/i16/i32/i64, u8/u16/u32, f32/f64, EntryId, Vec, HashSet, HashMap, Option"
)]
pub trait Field: Sized {
    fn definition(name: &str) -> FieldDefinition;

    fn to_value(&self) -> Result<Value, String>;

    fn from_value(value: Value) -> Result<Self, String>;
}

fn plain(name: &str, kind: FieldKind) -> FieldDefinition {
    FieldDefinition {
        name: name.into(),
        kind,
        flags: Vec::new(),
    }
}

fn flagged(name: &str, kind: FieldKind, flags: &[(&str, String)]) -> FieldDefinition {
    FieldDefinition {
        name: name.into(),
        kind,
        flags: flags
            .iter()
            .map(|(key, value)| ((*key).into(), value.clone()))
            .collect(),
    }
}

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::String(_) => "string",
        Value::Int(_) => "int",
        Value::Float(_) => "float",
        Value::Bool(_) => "bool",
        Value::Enum(_) => "enum",
        Value::Reference(_) => "reference",
        Value::Struct(_) => "struct",
        Value::List(_) => "list",
        Value::Map(_) => "map",
        Value::Variant { .. } => "variant",
        Value::Null => "null",
    }
}

fn mismatch<T>(expected: &str, found: &Value) -> Result<T, String> {
    Err(format!("expected {expected}, found {}", type_name(found)))
}

impl Field for String {
    fn definition(name: &str) -> FieldDefinition {
        plain(name, FieldKind::String)
    }

    fn to_value(&self) -> Result<Value, String> {
        Ok(Value::String(self.clone()))
    }

    fn from_value(value: Value) -> Result<Self, String> {
        match value {
            Value::String(value) => Ok(value),
            other => mismatch("string", &other),
        }
    }
}

impl Field for bool {
    fn definition(name: &str) -> FieldDefinition {
        plain(name, FieldKind::Bool)
    }

    fn to_value(&self) -> Result<Value, String> {
        Ok(Value::Bool(*self))
    }

    fn from_value(value: Value) -> Result<Self, String> {
        match value {
            Value::Bool(value) => Ok(value),
            other => mismatch("bool", &other),
        }
    }
}

impl Field for EntryId {
    fn definition(name: &str) -> FieldDefinition {
        plain(name, FieldKind::Reference)
    }

    fn to_value(&self) -> Result<Value, String> {
        Ok(Value::Reference(self.clone()))
    }

    fn from_value(value: Value) -> Result<Self, String> {
        match value {
            Value::Reference(id) => Ok(id),
            other => mismatch("reference", &other),
        }
    }
}

macro_rules! int_field {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Field for $ty {
                fn definition(name: &str) -> FieldDefinition {
                    flagged(
                        name,
                        FieldKind::Int,
                        &[
                            ("min", <$ty>::MIN.to_string()),
                            ("max", <$ty>::MAX.to_string()),
                        ],
                    )
                }

                fn to_value(&self) -> Result<Value, String> {
                    Ok(Value::Int(i64::from(*self)))
                }

                fn from_value(value: Value) -> Result<Self, String> {
                    match value {
                        Value::Int(value) => <$ty>::try_from(value).map_err(|_| {
                            format!("{value} is out of range for {}", stringify!($ty))
                        }),
                        other => mismatch("int", &other),
                    }
                }
            }
        )*
    };
}

int_field!(i8, i16, i32, i64, u8, u16, u32);

macro_rules! float_field {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Field for $ty {
                fn definition(name: &str) -> FieldDefinition {
                    plain(name, FieldKind::Float)
                }

                fn to_value(&self) -> Result<Value, String> {
                    Ok(Value::Float(f64::from(*self)))
                }

                fn from_value(value: Value) -> Result<Self, String> {
                    match value {
                        Value::Float(value) => Ok(value as $ty),
                        other => mismatch("float", &other),
                    }
                }
            }
        )*
    };
}

float_field!(f32, f64);

impl<T: Field> Field for Vec<T> {
    fn definition(name: &str) -> FieldDefinition {
        plain(name, FieldKind::List(Box::new(T::definition("").kind)))
    }

    fn to_value(&self) -> Result<Value, String> {
        self.iter()
            .map(T::to_value)
            .collect::<Result<_, _>>()
            .map(Value::List)
    }

    fn from_value(value: Value) -> Result<Self, String> {
        match value {
            Value::List(values) => values.into_iter().map(T::from_value).collect(),
            other => mismatch("list", &other),
        }
    }
}

impl<T: Field + Eq + Hash> Field for HashSet<T> {
    fn definition(name: &str) -> FieldDefinition {
        flagged(
            name,
            FieldKind::List(Box::new(T::definition("").kind)),
            &[("unique", "true".into())],
        )
    }

    fn to_value(&self) -> Result<Value, String> {
        self.iter()
            .map(T::to_value)
            .collect::<Result<_, _>>()
            .map(Value::List)
    }

    fn from_value(value: Value) -> Result<Self, String> {
        match value {
            Value::List(values) => values.into_iter().map(T::from_value).collect(),
            other => mismatch("list", &other),
        }
    }
}

impl<K: Field + Eq + Hash, V: Field> Field for HashMap<K, V> {
    fn definition(name: &str) -> FieldDefinition {
        plain(
            name,
            FieldKind::Map {
                key: Box::new(K::definition("").kind),
                value: Box::new(V::definition("").kind),
            },
        )
    }

    fn to_value(&self) -> Result<Value, String> {
        self.iter()
            .map(|(key, value)| Ok((key.to_value()?, value.to_value()?)))
            .collect::<Result<_, String>>()
            .map(Value::Map)
    }

    fn from_value(value: Value) -> Result<Self, String> {
        match value {
            Value::Map(pairs) => pairs
                .into_iter()
                .map(|(key, value)| Ok((K::from_value(key)?, V::from_value(value)?)))
                .collect(),
            other => mismatch("map", &other),
        }
    }
}

impl<T: Field> Field for Option<T> {
    fn definition(name: &str) -> FieldDefinition {
        let mut definition = T::definition(name);
        definition.flags.push(("optional".into(), "true".into()));
        definition
    }

    fn to_value(&self) -> Result<Value, String> {
        match self {
            Some(value) => value.to_value(),
            None => Ok(Value::Null),
        }
    }

    fn from_value(value: Value) -> Result<Self, String> {
        match value {
            Value::Null => Ok(None),
            other => T::from_value(other).map(Some),
        }
    }
}
