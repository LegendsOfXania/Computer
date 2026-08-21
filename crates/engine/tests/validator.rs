use std::collections::BTreeMap;

use engine::{
    EntryDefinition,
    Registry,
    ValidationError,
    Validator,
};

use model::{
    EntryData,
    Field,
    Schema,
    Value,
};

use runtime::Library;

#[test]
fn validates_entry_and_collects_errors() {
    let registry = Registry::new();

    registry
        .register_entry(
            EntryDefinition::new("dialogue")
                .field(Field::new("text", Schema::Text))
        )
        .expect("Failed to register entry");

    let library = Library::new();

    let entry = EntryData::new(
        "dialogue",
        BTreeMap::from([
            ("name".to_string(), Value::integer(42)),
            ("unknown".to_string(), Value::text("hello")),
        ]),
    );

    let validator = Validator::new(&registry, &library);

    let errors = validator.validate_entry(&entry);

    assert_eq!(
        errors,
        vec![
            ValidationError::UnknownField {
                field: "unknown".into(),
            },
            ValidationError::InvalidValue {
                field: "name".into(),
            },
            ValidationError::MissingField {
                field: "text".into(),
            },
        ],
    );
}