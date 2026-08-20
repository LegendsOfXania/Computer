use serialization::{
    Format,
    KdlFormat,  
    Value,
};

use std::collections::BTreeMap;

#[test]
fn primitives_round_trip() {
    let value = Value::structure(BTreeMap::from([
        ("null".into(), Value::Null),
        ("text".into(), Value::text("Hello, world!")),
        ("integer".into(), Value::integer(42)),
        ("float".into(), Value::float(42.5)),
        ("boolean".into(), Value::boolean(true)),
        ("enum".into(), Value::enumeration("sequence")),
        (
            "reference".into(),
            Value::reference("page:entry"),
        ),
    ]));

    assert_round_trip(value);
}

#[test]
fn nested_struct_round_trip() {
    let value = Value::structure(BTreeMap::from([
        (
            "position".into(),
            Value::structure(BTreeMap::from([
                ("x".into(), Value::integer(10)),
                ("y".into(), Value::integer(20)),
                ("z".into(), Value::integer(30)),
            ])),
        ),
        (
            "settings".into(),
            Value::structure(BTreeMap::from([
                ("enabled".into(), Value::boolean(true)),
                (
                    "name".into(),
                    Value::text("Computer"),
                ),
            ])),
        ),
    ]));

    assert_round_trip(value);
}

#[test]
fn list_round_trip() {
    let value = Value::structure(BTreeMap::from([
        (
            "items".into(),
            Value::list(vec![
                Value::text("apple"),
                Value::text("sword"),
                Value::text("shield"),
            ]),
        ),
    ]));

    assert_round_trip(value);
}

#[test]
fn nested_lists_round_trip() {
    let value = Value::structure(BTreeMap::from([
        (
            "values".into(),
            Value::list(vec![
                Value::list(vec![
                    Value::integer(1),
                    Value::integer(2),
                ]),
                Value::list(vec![
                    Value::integer(3),
                    Value::integer(4),
                ]),
            ]),
        ),
    ]));

    assert_round_trip(value);
}

#[test]
fn list_of_structs_round_trip() {
    let value = Value::structure(BTreeMap::from([
        (
            "players".into(),
            Value::list(vec![
                Value::structure(BTreeMap::from([
                    ("name".into(), Value::text("Xaya")),
                    ("level".into(), Value::integer(42)),
                ])),
                Value::structure(BTreeMap::from([
                    ("name".into(), Value::text("Steve")),
                    ("level".into(), Value::integer(10)),
                ])),
            ]),
        ),
    ]));

    assert_round_trip(value);
}

#[test]
fn empty_struct_round_trip() {
    let value = Value::structure(BTreeMap::from([
        (
            "empty".into(),
            Value::structure(BTreeMap::new()),
        ),
    ]));

    assert_round_trip(value);
}

#[test]
fn empty_list_round_trip() {
    let value = Value::structure(BTreeMap::from([
        (
            "items".into(),
            Value::list(Vec::new()),
        ),
    ]));

    assert_round_trip(value);
}

#[test]
fn complex_round_trip() {
    let value = Value::structure(BTreeMap::from([
        ("id".into(), Value::text("page-001")),
        (
            "metadata".into(),
            Value::structure(BTreeMap::from([
                (
                    "name".into(),
                    Value::text("My first page"),
                ),
                (
                    "type".into(),
                    Value::enumeration("sequence"),
                ),
                (
                    "author".into(),
                    Value::reference("users:xaya"),
                ),
            ])),
        ),
        (
            "entries".into(),
            Value::list(vec![
                Value::structure(BTreeMap::from([
                    ("id".into(), Value::text("entry-001")),
                    (
                        "type".into(),
                        Value::enumeration("dialogue"),
                    ),
                    (
                        "text".into(),
                        Value::text("Hello!"),
                    ),
                    (
                        "speaker".into(),
                        Value::reference("characters:npc"),
                    ),
                ])),
                Value::structure(BTreeMap::from([
                    ("id".into(), Value::text("entry-002")),
                    (
                        "type".into(),
                        Value::enumeration("action"),
                    ),
                    (
                        "enabled".into(),
                        Value::boolean(true),
                    ),
                ])),
            ]),
        ),
    ]));

    assert_round_trip(value);
}

#[test]
fn decode_primitives() {
    let input = r#"
"null" #null
text "Hello"
integer 42
float 42.5
boolean #true
mode (enum)"sequence"
reference (ref)"page:entry"
"#;

    let value = KdlFormat::decode(input).unwrap();

    let expected = Value::structure(BTreeMap::from([
        ("null".into(), Value::Null),
        ("text".into(), Value::text("Hello")),
        ("integer".into(), Value::integer(42)),
        ("float".into(), Value::float(42.5)),
        ("boolean".into(), Value::boolean(true)),
        (
            "mode".into(),
            Value::enumeration("sequence"),
        ),
        (
            "reference".into(),
            Value::reference("page:entry"),
        ),
    ]));

    assert_eq!(value, expected);
}

#[test]
fn decode_duplicate_nodes_fails() {
    let input = r#"
tag "first"
tag "second"
"#;

    assert!(KdlFormat::decode(input).is_err());
}

#[test]
fn decode_node_with_multiple_values_fails() {
    let input = r#"
position 10 20
"#;

    assert!(KdlFormat::decode(input).is_err());
}

#[test]
fn decode_node_with_value_and_children_fails() {
    let input = r#"
position 10 {
    x 20
}
"#;

    assert!(KdlFormat::decode(input).is_err());
}

#[test]
fn decode_unsupported_type_fails() {
    let input = r#"
value (unknown)"hello"
"#;

    assert!(KdlFormat::decode(input).is_err());
}

#[test]
fn root_must_be_a_struct() {
    let value = Value::text("Hello");

    assert!(KdlFormat::encode(&value).is_err());
}

fn assert_round_trip(value: Value) {
    let encoded = KdlFormat::encode(&value).unwrap();

    let decoded = KdlFormat::decode(&encoded).unwrap();

    assert_eq!(
        decoded,
        value,
        "\n--- Encoded KDL ---\n{encoded}"
    );
}