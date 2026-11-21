use computer_model::{Field, FieldKind, FieldNode, Value};

pub fn defaults(def: &[Field]) -> Vec<Value> {
    let mut out = Vec::new();

    push(FieldNode::new(def, 0), &mut out);

    out
}

fn push(node: FieldNode, out: &mut Vec<Value>) {
    let Some(field) = node.value() else {
        out.push(Value::Null);
        return;
    };

    match &field.kind {
        FieldKind::String => out.push(Value::String(String::new())),
        FieldKind::Int => out.push(Value::Int(0)),
        FieldKind::Float => out.push(Value::Float(0.0)),
        FieldKind::Bool => out.push(Value::Bool(false)),
        FieldKind::Reference => out.push(Value::Null),
        FieldKind::Enum(options) => {
            out.push(Value::Enum(options.first().cloned().unwrap_or_default()));
        }
        FieldKind::List => out.push(Value::List(0)),
        FieldKind::Map => out.push(Value::Map(0)),
        FieldKind::Struct(_) => group(node.children().collect(), Value::Struct, out),
        FieldKind::Variant(_) => match node.cases().next() {
            Some((case, fields)) => group(
                fields,
                |names| Value::Variant((case.to_owned(), names)),
                out,
            ),
            None => out.push(Value::Null),
        },
    }
}

fn group(fields: Vec<FieldNode>, head: impl FnOnce(Vec<String>) -> Value, out: &mut Vec<Value>) {
    out.push(head(
        fields
            .iter()
            .filter_map(|node| node.value())
            .map(|field| field.name.clone())
            .collect(),
    ));

    for field in fields {
        push(field, out);
    }
}
