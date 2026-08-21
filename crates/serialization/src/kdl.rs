use crate::{Format, Number, SerializationError, Value};

use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

use std::collections::{BTreeMap, btree_map::Entry};

pub struct KdlFormat;

impl Format for KdlFormat {
    fn decode(input: &str) -> Result<Value, SerializationError> {
        let document = input
            .parse::<KdlDocument>()
            .map_err(|error| SerializationError::Parse(error.to_string()))?;

        decode_document(&document)
    }

    fn encode(value: &Value) -> Result<String, SerializationError> {
        let Value::Struct(values) = value else {
            return Err(SerializationError::invalid_structure(
                "the root value must be a struct",
            ));
        };

        Ok(encode_document(values).to_string())
    }
}

fn decode_document(document: &KdlDocument) -> Result<Value, SerializationError> {
    let mut values = BTreeMap::new();

    for node in document.nodes() {
        let name = node.name().value();

        let value = decode_node(node)?;

        match values.entry(name.to_owned()) {
            Entry::Vacant(entry) => {
                entry.insert(value);
            }

            Entry::Occupied(_) => {
                return Err(SerializationError::invalid_structure(format!(
                    "duplicate node `{name}`"
                )));
            }
        }
    }

    Ok(Value::Struct(values))
}

fn decode_node(node: &KdlNode) -> Result<Value, SerializationError> {
    match node.ty() {
        Some(ty) if ty.value() == "list" => decode_list(node),

        Some(ty) => Err(SerializationError::unsupported_type(ty.value())),

        None => decode_standard_node(node),
    }
}

fn decode_standard_node(node: &KdlNode) -> Result<Value, SerializationError> {
    let entries = node.entries();
    let children = node.children();

    match (entries.is_empty(), children) {
        (true, Some(children)) => decode_document(children),

        (false, None) => {
            if entries.len() != 1 {
                return Err(SerializationError::invalid_structure(format!(
                    "node `{}` must contain exactly one value",
                    node.name().value(),
                )));
            }

            let entry = &entries[0];

            if entry.name().is_some() {
                return Err(SerializationError::invalid_structure(format!(
                    "node `{}` cannot contain named properties",
                    node.name().value(),
                )));
            }

            decode_entry(entry)
        }

        (true, None) => Err(SerializationError::invalid_structure(format!(
            "node `{}` has neither a value nor children",
            node.name().value(),
        ))),

        (false, Some(_)) => Err(SerializationError::invalid_structure(format!(
            "node `{}` cannot contain both values and children",
            node.name().value(),
        ))),
    }
}

fn decode_list(node: &KdlNode) -> Result<Value, SerializationError> {
    if !node.entries().is_empty() {
        return Err(SerializationError::invalid_structure(format!(
            "list `{}` cannot contain direct values",
            node.name().value(),
        )));
    }

    let children = node.children().ok_or_else(|| {
        SerializationError::invalid_structure(format!(
            "list `{}` has no children",
            node.name().value(),
        ))
    })?;

    let mut values = Vec::with_capacity(children.nodes().len());

    for child in children.nodes() {
        if child.name().value() != "-" {
            return Err(SerializationError::invalid_structure(format!(
                "list `{}` can only contain `-` items",
                node.name().value(),
            )));
        }

        values.push(decode_node(child)?);
    }

    Ok(Value::List(values))
}

fn decode_entry(entry: &KdlEntry) -> Result<Value, SerializationError> {
    match entry.ty() {
        Some(ty) if ty.value() == "ref" => {
            let value = entry.value().as_string().ok_or_else(|| {
                SerializationError::invalid_structure("a reference must contain a string")
            })?;

            Ok(Value::Reference(value.to_owned()))
        }

        Some(ty) if ty.value() == "enum" => {
            let value = entry.value().as_string().ok_or_else(|| {
                SerializationError::invalid_structure("an enum must contain a string")
            })?;

            Ok(Value::Enum(value.to_owned()))
        }

        Some(ty) => Err(SerializationError::unsupported_type(ty.value())),

        None => decode_primitive(entry.value()),
    }
}

fn decode_primitive(value: &KdlValue) -> Result<Value, SerializationError> {
    match value {
        KdlValue::Null => Ok(Value::Null),

        KdlValue::String(value) => Ok(Value::Text(value.clone())),

        KdlValue::Bool(value) => Ok(Value::Boolean(*value)),

        KdlValue::Integer(value) => {
            let value =
                i64::try_from(*value).map_err(|_| SerializationError::IntegerOutOfRange(*value))?;

            Ok(Value::Number(Number::Integer(value)))
        }

        KdlValue::Float(value) => Ok(Value::Number(Number::Float(*value))),
    }
}

fn encode_document(values: &BTreeMap<String, Value>) -> KdlDocument {
    let mut document = KdlDocument::new();

    document
        .nodes_mut()
        .extend(values.iter().map(|(name, value)| encode_node(name, value)));

    document
}

fn encode_node(name: &str, value: &Value) -> KdlNode {
    let mut node = KdlNode::new(name);

    match value {
        Value::Struct(values) => {
            node.set_children(encode_document(values));
        }

        Value::List(values) => {
            node.set_ty("list");

            let mut children = KdlDocument::new();

            children
                .nodes_mut()
                .extend(values.iter().map(|value| encode_node("-", value)));

            node.set_children(children);
        }

        _ => {
            node.push(encode_entry(value));
        }
    }

    node
}

fn encode_entry(value: &Value) -> KdlEntry {
    match value {
        Value::Null => KdlEntry::new(KdlValue::Null),

        Value::Text(value) => KdlEntry::new(value.clone()),

        Value::Number(Number::Integer(value)) => KdlEntry::new(i128::from(*value)),

        Value::Number(Number::Float(value)) => KdlEntry::new(*value),

        Value::Boolean(value) => KdlEntry::new(*value),

        Value::Enum(value) => {
            let mut entry = KdlEntry::new(value.clone());

            entry.set_ty("enum");

            entry
        }

        Value::Reference(value) => {
            let mut entry = KdlEntry::new(value.clone());

            entry.set_ty("ref");

            entry
        }

        Value::Struct(_) | Value::List(_) => {
            unreachable!("structured values must be encoded as nodes")
        }
    }
}
