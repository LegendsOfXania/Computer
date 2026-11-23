use crate::id::EntryId;

use super::{Node, Value};

impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}

macro_rules! convert {
    ($($ty:ty => $variant:ident),+ $(,)?) => {$(
        impl From<$ty> for Value {
            fn from(value: $ty) -> Self {
                Self::$variant(value)
            }
        }
    )+};
}

convert!(i64 => Int, f64 => Float, bool => Bool, String => String);

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<EntryId> for Value {
    fn from(id: EntryId) -> Self {
        Self::Reference(id.to_string())
    }
}

impl<'a> Node<'a> {
    fn child(&self, names: &[String], name: &str) -> Option<Self> {
        self.children()
            .nth(names.iter().position(|known| known == name)?)
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<Self> {
        match self.value()? {
            Value::Struct(names) | Value::Variant((_, names)) => self.child(names, name),
            _ => None,
        }
    }

    pub fn pairs(&self) -> impl Iterator<Item = (Self, Self)> + 'a {
        let mut children = self.children();

        std::iter::from_fn(move || Some((children.next()?, children.next()?)))
    }

    fn has_same_fields(&self, names: &[String], other: &Self, other_names: &[String]) -> bool {
        names.len() == other_names.len()
            && names.iter().zip(self.children()).all(|(name, node)| {
                other
                    .child(other_names, name)
                    .is_some_and(|counterpart| node == counterpart)
            })
    }
}

impl PartialEq for Node<'_> {
    fn eq(&self, other: &Self) -> bool {
        let (a, b) = (self.value(), other.value());

        match (a, b) {
            (Some(Value::Struct(x)), Some(Value::Struct(y))) => self.has_same_fields(x, other, y),
            (Some(Value::Variant((case, x))), Some(Value::Variant((other_case, y)))) => {
                case == other_case && self.has_same_fields(x, other, y)
            }
            (Some(Value::List(x)), Some(Value::List(y)))
            | (Some(Value::Map(x)), Some(Value::Map(y))) => {
                x == y && self.children().zip(other.children()).all(|(a, b)| a == b)
            }
            _ => a == b,
        }
    }
}

fn mismatch<T>(expected: &str, found: Option<&Value>) -> Result<T, String> {
    Err(format!("expected {expected}, found {found:?}"))
}

macro_rules! extract {
    ($($ty:ty => $expected:literal, $pattern:pat => $result:expr),+ $(,)?) => {$(
        impl TryFrom<Node<'_>> for $ty {
            type Error = String;

            fn try_from(node: Node<'_>) -> Result<Self, String> {
                match node.value() {
                    Some($pattern) => Ok($result),
                    other => mismatch($expected, other),
                }
            }
        }
    )+};
}

extract!(
    String => "string", Value::String(text) | Value::Enum(text) => text.clone(),
    i64 => "int", Value::Int(number) => *number,
    f64 => "float", Value::Float(number) => *number,
    bool => "bool", Value::Bool(flag) => *flag,
    EntryId => "reference", Value::Reference(id) => EntryId::from(id.as_str()),
);

impl<'a, T: TryFrom<Node<'a>, Error = String>> TryFrom<Node<'a>> for Vec<T> {
    type Error = String;

    fn try_from(node: Node<'a>) -> Result<Self, String> {
        match node.value() {
            Some(Value::List(_)) => node.children().map(T::try_from).collect(),
            other => mismatch("list", other),
        }
    }
}
