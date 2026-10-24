use std::collections::{HashMap, HashSet};

use computer_model::{
    field::{FieldDefinition, FieldKind},
    key::EntryKey,
};

#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as an entry field",
    label = "this type does not implement `Field`",
    note = "add `#[derive(Field)]` to your struct/enum, or use a supported type: String, bool, i8/i16/i32, u8/u16, f32, EntryKey, Vec, HashMap, HashSet, Option"
)]
pub trait Field {
    fn definition(name: &str) -> FieldDefinition;
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

impl Field for String {
    fn definition(name: &str) -> FieldDefinition {
        plain(name, FieldKind::String)
    }
}

impl Field for bool {
    fn definition(name: &str) -> FieldDefinition {
        plain(name, FieldKind::Bool)
    }
}

impl Field for f32 {
    fn definition(name: &str) -> FieldDefinition {
        plain(name, FieldKind::Float)
    }
}

impl Field for EntryKey {
    fn definition(name: &str) -> FieldDefinition {
        plain(name, FieldKind::Reference)
    }
}

macro_rules! int_field {
    ($($ty:ty),* $(,)?) => {$(
        impl Field for $ty {
            fn definition(name: &str) -> FieldDefinition {
                flagged(
                    name,
                    FieldKind::Int,
                    &[("min", <$ty>::MIN.to_string()), ("max", <$ty>::MAX.to_string())],
                )
            }
        }
    )*};
}

int_field!(i8, i16, i32, u8, u16);

impl<T: Field> Field for Vec<T> {
    fn definition(name: &str) -> FieldDefinition {
        plain(name, FieldKind::List(Box::new(T::definition("").kind)))
    }
}

impl<T: Field> Field for HashSet<T> {
    fn definition(name: &str) -> FieldDefinition {
        flagged(
            name,
            FieldKind::List(Box::new(T::definition("").kind)),
            &[("unique", "true".into())],
        )
    }
}

impl<K: Field, V: Field> Field for HashMap<K, V> {
    fn definition(name: &str) -> FieldDefinition {
        plain(
            name,
            FieldKind::Map {
                key: Box::new(K::definition("").kind),
                value: Box::new(V::definition("").kind),
            },
        )
    }
}

impl<T: Field> Field for Option<T> {
    fn definition(name: &str) -> FieldDefinition {
        let mut definition = T::definition(name);
        definition.flags.push(("optional".into(), "true".into()));
        definition
    }
}
