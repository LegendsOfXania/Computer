use model::{EntryData, Field, PageData, PageType, Schema, Value};
use runtime::{
    Entry, EntryDefinition, EntryKey, Library, Page, Registry, TagDefinition, ValidationError,
    Validator,
};
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(Debug)]
struct TestEntry {
    id: String,
    data: EntryData,
}
impl Entry for TestEntry {
    fn id(&self) -> &str {
        &self.id
    }
    fn data(&self) -> &EntryData {
        &self.data
    }
}
fn entry(id: &str, entry_type: &str, fields: BTreeMap<String, Value>) -> TestEntry {
    TestEntry {
        id: id.into(),
        data: EntryData::new(entry_type, fields),
    }
}

#[test]
fn library_indexes_page_entries() {
    let mut page = Page::new("0", PageData::new("main", PageType::Sequence, 0));
    page.insert(entry("one", "dialogue", BTreeMap::new()));
    let library = Library::new();
    library.insert(page);
    assert!(
        library
            .entries()
            .contains(&EntryKey::from_str("0:one").unwrap())
    );
}

#[test]
fn registry_resolves_tag_fields() {
    let registry = Registry::new();
    registry
        .register_tag(TagDefinition::new("triggerable").field(Field::new("trigger", Schema::Text)))
        .unwrap();
    registry
        .register_entry(EntryDefinition::new("action").tag("triggerable"))
        .unwrap();
    let definition = registry.entries().get("action").unwrap();
    assert!(
        definition
            .fields()
            .iter()
            .any(|field| field.name() == "name")
    );
    assert!(
        definition
            .fields()
            .iter()
            .any(|field| field.name() == "trigger")
    );
}

#[test]
fn validator_collects_local_errors() {
    let registry = Registry::new();
    registry
        .register_entry(EntryDefinition::new("dialogue").field(Field::new("text", Schema::Text)))
        .unwrap();
    let library = Library::new();
    let validator = Validator::new(&registry, &library);
    let data = EntryData::new(
        "dialogue",
        BTreeMap::from([
            ("name".into(), Value::integer(1)),
            ("extra".into(), Value::text("x")),
        ]),
    );
    assert_eq!(
        validator.validate_entry(&data),
        vec![
            ValidationError::UnknownField {
                field: "extra".into()
            },
            ValidationError::InvalidValue {
                field: "name".into()
            },
            ValidationError::MissingField {
                field: "text".into()
            }
        ]
    );
}

#[test]
fn validator_checks_reference_constraints() {
    let registry = Registry::new();
    registry
        .register_tag(TagDefinition::new("triggerable"))
        .unwrap();
    registry
        .register_entry(EntryDefinition::new("action").tag("triggerable"))
        .unwrap();
    registry
        .register_entry(EntryDefinition::new("dialogue").field(Field::new(
            "target",
            Schema::reference().with_tag("triggerable"),
        )))
        .unwrap();
    let mut page = Page::new("0", PageData::new("main", PageType::Sequence, 0));
    page.insert(entry(
        "action",
        "action",
        BTreeMap::from([("name".into(), Value::text("A"))]),
    ));
    let library = Library::new();
    library.insert(page);
    let validator = Validator::new(&registry, &library);
    let data = EntryData::new(
        "dialogue",
        BTreeMap::from([
            ("name".into(), Value::text("D")),
            ("target".into(), Value::reference("0:action")),
        ]),
    );
    assert!(validator.validate_entry(&data).is_empty());
}
