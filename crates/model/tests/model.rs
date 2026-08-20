use model::{
    combine_fields,
    EntryData,
    Field,
    Fields,
    PageData,
    PageType,
    Schema,
    StructSchema,
    Value,
};

struct DialogueFields;

impl Fields for DialogueFields {
    fn fields() -> Vec<Field> {
        vec![
            Field::new(
                "text",
                Schema::Text,
            ),
            Field::new(
                "speaker",
                Schema::reference(["character"]),
            ),
        ]
    }
}

struct SpokenDialogueFields;

impl Fields for SpokenDialogueFields {
    fn fields() -> Vec<Field> {
        combine_fields([
            DialogueFields::fields(),
            vec![
                Field::new(
                    "voice",
                    Schema::Text,
                ),
            ],
        ])
    }
}

#[test]
fn entry_data_keeps_type_and_fields() {
    let entry = EntryData::new(
        "dialogue",
        std::collections::BTreeMap::from([(
            "name".into(),
            Value::text("Greeting"),
        )]),
    );

    assert_eq!(entry.entry_type(), "dialogue");
    assert_eq!(entry.field("name"), Some(&Value::text("Greeting")));
}

#[test]
fn field_sets_can_be_composed() {
    let fields = SpokenDialogueFields::fields();

    assert_eq!(
        fields
            .iter()
            .map(|field| field.name())
            .collect::<Vec<_>>(),
        ["text", "speaker", "voice"],
    );
}

#[test]
fn struct_schema_matches_a_rust_struct_shape() {
    let position = StructSchema::new([
        Field::new(
            "world",
            Schema::Text,
        ),
        Field::new(
            "x",
            Schema::Float,
        ),
        Field::new(
            "y",
            Schema::Float,
        ),
        Field::new(
            "z",
            Schema::Float,
        ),
    ]);

    let schema = Schema::Struct(position.clone());

    let value = Value::structure(
        std::collections::BTreeMap::from([
            (
                "world".into(),
                Value::text("world"),
            ),
            (
                "x".into(),
                Value::float(10.0),
            ),
            (
                "y".into(),
                Value::float(64.0),
            ),
            (
                "z".into(),
                Value::float(-20.0),
            ),
        ]),
    );

    assert!(schema.accepts(&value));

    assert_eq!(
        position
            .field("x")
            .unwrap()
            .schema(),
        &Schema::Float,
    );
}

#[test]
fn optional_fields_are_not_required_for_struct_validation() {
    let schema = Schema::structure([
        Field::new(
            "name",
            Schema::Text,
        ),
        Field::optional(
            "description",
            Schema::Text,
        ),
    ]);

    let value = Value::structure(
        std::collections::BTreeMap::from([
            (
                "name".into(),
                Value::text("Computer"),
            ),
        ]),
    );

    assert!(schema.accepts(&value));
}

#[test]
fn nested_structs_and_lists_are_described_recursively() {
    let position = Schema::structure([
        Field::new(
            "x",
            Schema::Float,
        ),
        Field::new(
            "y",
            Schema::Float,
        ),
        Field::new(
            "z",
            Schema::Float,
        ),
    ]);

    let path = Schema::list(position);

    let value = Value::list(vec![
        Value::structure(
            std::collections::BTreeMap::from([
                (
                    "x".into(),
                    Value::float(1.0),
                ),
                (
                    "y".into(),
                    Value::float(2.0),
                ),
                (
                    "z".into(),
                    Value::float(3.0),
                ),
            ]),
        ),
    ]);

    assert!(path.accepts(&value));
}

#[test]
fn schema_rejects_wrong_value_shapes() {
    assert!(!Schema::Text.accepts(
        &Value::integer(42),
    ));

    assert!(!Schema::Integer.accepts(
        &Value::float(42.0),
    ));

    assert!(
        !Schema::list(Schema::Text)
            .accepts(&Value::list(vec![
                Value::integer(42),
            ]))
    );
}

#[test]
fn page_type_round_trip() {
    for page_type in [
        PageType::Sequence,
        PageType::Manifest,
        PageType::Static,
    ] {
        assert_eq!(
            PageType::parse(page_type.as_str()),
            Some(page_type),
        );
    }
}

#[test]
fn page_data_keeps_metadata() {
    let page = PageData::new(
        "page",
        "Page",
        PageType::Sequence,
        10,
    );

    assert_eq!(page.id(), "page");
    assert_eq!(page.name(), "Page");
    assert_eq!(
        page.page_type(),
        PageType::Sequence,
    );
    assert_eq!(page.priority(), 10);
}