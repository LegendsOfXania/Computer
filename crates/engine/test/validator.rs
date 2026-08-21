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
    let mut registry = Registry::new();

    registry
        .entries_mut()
        .register(
            EntryDefinition::new(
                "dialogue",
                vec![
                    Field::new(
                        "name",
                        Schema::Text,
                    ),
                    Field::new(
                        "text",
                        Schema::Text,
                    ),
                    Field::optional(
                        "enabled",
                        Schema::Boolean,
                    ),
                ],
            ),
        )
        .unwrap();

    let library = Library::new();

    let entry = EntryData::new(
        "dialogue",
        [
            (
                "name",
                Value::integer(42),
            ),
            (
                "unknown",
                Value::text("hello"),
            ),
        ],
    );

    let validator = Validator::new(
        &registry,
        &library,
    );

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