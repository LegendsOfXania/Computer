use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use computer_model::{Field as ModelField, FieldKind, Node, Value, id::EntryId};

#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as an entry field",
    label = "this type does not implement `Field`",
    note = "use a supported type: String, bool, i8/i16/i32/i64, u8/u16/u32, f32/f64, EntryId, Vec, HashSet, HashMap, Option"
)]
pub trait Field: Sized {
    fn definition(name: &str, flags: &[(&str, &str)], out: &mut Vec<ModelField>);

    fn read(node: Option<Node<'_>>) -> Result<Self, String>;
}

#[doc(hidden)]
pub fn define<T: Field>(name: &str, flags: &[(&str, &str)], out: &mut Vec<ModelField>) {
    T::definition(name, flags, out);
}

#[doc(hidden)]
pub fn member<T: Field>(node: Node<'_>, key: &str) -> Result<T, String> {
    T::read(node.get(key)).map_err(|error| format!("field `{key}`: {error}"))
}

#[doc(hidden)]
pub fn define_struct(name: &str, flags: &[(&str, &str)], fields: u32, out: &mut Vec<ModelField>) {
    leaf(name, flags, FieldKind::Struct(fields), out);
}

#[doc(hidden)]
pub fn expect_struct(node: Option<Node<'_>>) -> Result<Node<'_>, String> {
    let node = present(node)?;

    match node.value() {
        Some(Value::Struct(_)) => Ok(node),
        _ => Err("expected struct".into()),
    }
}

fn leaf(name: &str, flags: &[(&str, &str)], kind: FieldKind, out: &mut Vec<ModelField>) {
    out.push(ModelField {
        name: name.into(),
        flags: flags
            .iter()
            .map(|&(key, value)| (key.into(), value.into()))
            .collect(),
        kind,
    });
}

fn present(node: Option<Node<'_>>) -> Result<Node<'_>, String> {
    node.ok_or_else(|| "missing value".into())
}

macro_rules! scalar {
    ($($ty:ty => $kind:ident, $read:expr;)+) => {
        $(
            impl Field for $ty {
                fn definition(
                    name: &str,
                    flags: &[(&str, &str)],
                    out: &mut Vec<ModelField>,
                ) {
                    leaf(name, flags, FieldKind::$kind, out);
                }

                fn read(node: Option<Node<'_>>) -> Result<Self, String> {
                    $read(present(node)?)
                }
            }
        )+
    };
}

fn read_int<T>(node: Node<'_>) -> Result<T, String>
where
    T: TryFrom<i64>,
    T::Error: std::fmt::Display,
{
    T::try_from(i64::try_from(node)?).map_err(|error| error.to_string())
}

scalar! {
    i8      => Int, read_int;
    i16     => Int, read_int;
    i32     => Int, read_int;
    i64     => Int, read_int;
    u8      => Int, read_int;
    u16     => Int, read_int;
    u32     => Int, read_int;

    f64     => Float, f64::try_from;
    f32     => Float, |node| {
        #[allow(clippy::cast_possible_truncation)]
        f64::try_from(node).map(|value| value as f32)
    };

    bool    => Bool, bool::try_from;
    String  => String, String::try_from;
    EntryId => Reference, EntryId::try_from;
}

impl<T: Field> Field for Option<T> {
    fn definition(name: &str, flags: &[(&str, &str)], out: &mut Vec<ModelField>) {
        let mut flags = flags.to_vec();
        flags.push(("optional", "true"));

        T::definition(name, &flags, out);
    }

    fn read(node: Option<Node<'_>>) -> Result<Self, String> {
        match node {
            Some(node) if !matches!(node.value(), None | Some(Value::Null)) => {
                T::read(Some(node)).map(Some)
            }
            _ => Ok(None),
        }
    }
}

fn list_definition<T: Field>(name: &str, flags: &[(&str, &str)], out: &mut Vec<ModelField>) {
    leaf(name, flags, FieldKind::List, out);
    T::definition("", &[], out);
}

fn list_read<T: Field, C: FromIterator<T>>(node: Option<Node<'_>>) -> Result<C, String> {
    let node = present(node)?;

    match node.value() {
        Some(Value::List(_)) => node.children().map(|item| T::read(Some(item))).collect(),
        _ => Err("expected list".into()),
    }
}

macro_rules! list_field {
    ($collection:ident $(, $bound:path)*) => {
        impl<T: Field $(+ $bound)*> Field for $collection<T> {
            fn definition(
                name: &str,
                flags: &[(&str, &str)],
                out: &mut Vec<ModelField>,
            ) {
                list_definition::<T>(name, flags, out);
            }

            fn read(node: Option<Node<'_>>) -> Result<Self, String> {
                list_read(node)
            }
        }
    };
}

list_field!(Vec);
list_field!(HashSet, Eq, Hash);

impl<K: Field + Eq + Hash, V: Field> Field for HashMap<K, V> {
    fn definition(name: &str, flags: &[(&str, &str)], out: &mut Vec<ModelField>) {
        leaf(name, flags, FieldKind::Map, out);
        K::definition("", &[], out);
        V::definition("", &[], out);
    }

    fn read(node: Option<Node<'_>>) -> Result<Self, String> {
        let node = present(node)?;

        match node.value() {
            Some(Value::Map(_)) => node
                .pairs()
                .map(|(key, value)| Ok((K::read(Some(key))?, V::read(Some(value))?)))
                .collect(),
            _ => Err("expected map".into()),
        }
    }
}
