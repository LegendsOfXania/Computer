use computer_model::value::Value;
use indexmap::IndexMap;

pub trait ValueExt {
    fn into_text(self) -> String;
    fn into_int(self) -> i64;
    fn into_float(self) -> f64;
    fn into_list(self) -> Vec<Value>;
    fn into_map(self) -> Vec<(Value, Value)>;
    fn into_fields(self) -> IndexMap<String, Value>;
}

impl ValueExt for Value {
    fn into_text(self) -> String {
        match self {
            Self::String(text) => text,
            _ => String::new(),
        }
    }

    fn into_int(self) -> i64 {
        match self {
            Self::Int(number) => number,
            _ => 0,
        }
    }

    fn into_float(self) -> f64 {
        match self {
            Self::Float(number) => number,
            _ => 0.0,
        }
    }

    fn into_list(self) -> Vec<Value> {
        match self {
            Self::List(items) => items,
            _ => Vec::new(),
        }
    }

    fn into_map(self) -> Vec<(Value, Value)> {
        match self {
            Self::Map(pairs) => pairs,
            _ => Vec::new(),
        }
    }

    fn into_fields(self) -> IndexMap<String, Value> {
        match self {
            Self::Struct(fields) | Self::Variant { fields, .. } => fields,
            _ => IndexMap::new(),
        }
    }
}

pub fn update<T: Clone>(items: &[T], index: usize, change: impl FnOnce(&mut T)) -> Vec<T> {
    let mut next = items.to_vec();
    change(&mut next[index]);
    next
}

pub fn remove<T: Clone>(items: &[T], index: usize) -> Vec<T> {
    let mut next = items.to_vec();
    next.remove(index);
    next
}

pub fn duplicate<T: Clone>(items: &[T], index: usize) -> Vec<T> {
    let mut next = items.to_vec();
    next.insert(index + 1, items[index].clone());
    next
}

pub fn push<T: Clone>(items: &[T], item: T) -> Vec<T> {
    let mut next = items.to_vec();
    next.push(item);
    next
}

pub fn insert(
    values: &IndexMap<String, Value>,
    name: String,
    value: Value,
) -> IndexMap<String, Value> {
    let mut next = values.clone();
    next.insert(name, value);
    next
}
