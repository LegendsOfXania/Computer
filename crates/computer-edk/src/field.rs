use std::collections::{HashMap, HashSet};

use computer_model::{
    field::{FieldDefinition, FieldKind},
    key::EntryKey,
};

pub trait Field {
    fn definition(name: &str) -> FieldDefinition;
}

fn definition(name: &str, kind: FieldKind) -> FieldDefinition {
    FieldDefinition {
        name: name.into(),
        kind,
        flags: Vec::new(),
    }
}

fn ranged_definition(
    name: &str,
    min: impl ToString,
    max: impl ToString,
) -> FieldDefinition {
    FieldDefinition {
        name: name.into(),
        kind: FieldKind::Int,
        flags: vec![
            ("min".into(), min.to_string()),
            ("max".into(), max.to_string()),
        ],
    }
}

impl Field for String {
    fn definition(name: &str) -> FieldDefinition {
        definition(name, FieldKind::String)
    }
}

impl Field for bool {
    fn definition(name: &str) -> FieldDefinition {
        definition(name, FieldKind::Bool)
    }
}

impl Field for i32 {
    fn definition(name: &str) -> FieldDefinition {
        ranged_definition(name, i32::MIN, i32::MAX)
    }
}

impl Field for f32 {
    fn definition(name: &str) -> FieldDefinition {
        definition(name, FieldKind::Float)
    }
}

macro_rules! signed_int_field {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Field for $ty {
                fn definition(name: &str) -> FieldDefinition {
                    ranged_definition(name, <$ty>::MIN, <$ty>::MAX)
                }
            }
        )*
    };
}

signed_int_field!(
    i8,
    i16,
);

macro_rules! unsigned_int_field {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Field for $ty {
                fn definition(name: &str) -> FieldDefinition {
                    ranged_definition(name, <$ty>::MIN, <$ty>::MAX)
                }
            }
        )*
    };
}

unsigned_int_field!(
    u8,
    u16,
);

impl Field for EntryKey {
    fn definition(name: &str) -> FieldDefinition {
        definition(name, FieldKind::Reference)
    }
}

impl<T> Field for Vec<T>
where
    T: Field,
{
    fn definition(name: &str) -> FieldDefinition {
        FieldDefinition {
            name: name.into(),
            kind: FieldKind::List(Box::new(T::definition("").kind)),
            flags: Vec::new(),
        }
    }
}


impl<K, V> Field for HashMap<K, V>
where
    K: Field,
    V: Field,
{
    fn definition(name: &str) -> FieldDefinition {
        FieldDefinition {
            name: name.into(),
            kind: FieldKind::Map {
                key: Box::new(K::definition("").kind),
                value: Box::new(V::definition("").kind),
            },
            flags: Vec::new(),
        }
    }
}

impl<K> Field for HashSet<K>
where
    K: Field,
{
    fn definition(name: &str) -> FieldDefinition {
        FieldDefinition {
            name: name.into(),
            kind: FieldKind::List(Box::new(K::definition("").kind)),
            flags: vec![("unique".into(), "true".into())],
        }
    }
}

impl<T> Field for Option<T>
where
    T: Field,
{
    fn definition(name: &str) -> FieldDefinition {
        let mut definition = T::definition(name);
        definition.flags.push(("optional".into(), "true".into()));
        definition
    }
}