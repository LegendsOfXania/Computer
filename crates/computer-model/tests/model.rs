use model::{EntryData, Field, PageData, PageType, Schema, Value};
use std::collections::BTreeMap;

#[test]
fn entry_data_has_no_runtime_identity() {
    let data = EntryData::new("dialogue", BTreeMap::new());
    assert_eq!(data.entry_type(), "dialogue");
}

#[test]
fn reference_schema_builds_constraints() {
    let schema = Schema::reference()
        .with_entry_type("dialogue")
        .with_tags(["triggerable", "action"]);
    let field = Field::new("target", schema);
    assert!(field.schema().accepts(&Value::reference("page:entry")));
}

#[test]
fn page_type_round_trips_through_text() {
    let page = PageData::new("main", PageType::Sequence, 0);
    assert_eq!(page.page_type().to_string(), "sequence");
    assert_eq!("static".parse::<PageType>(), Ok(PageType::Static));
}
