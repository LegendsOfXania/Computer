use model::{EntryData, PageData, PageType, Value};
use serialization::{Format, KdlFormat, RawEntry, RawPage, decode_page, encode_page};
use std::collections::BTreeMap;
#[test]
fn primitives_round_trip() {
    let value = Value::structure(BTreeMap::from([
        ("null".into(), Value::Null),
        ("text".into(), Value::text("hello")),
        ("integer".into(), Value::integer(42)),
        ("reference".into(), Value::reference("main:one")),
    ]));
    let encoded = KdlFormat::encode(&value).unwrap();
    assert_eq!(KdlFormat::decode(&encoded).unwrap(), value);
}
#[test]
fn page_round_trip() {
    let page = RawPage::new(
        "0",
        PageData::new("main", PageType::Sequence, 0),
        [RawEntry::new(
            "one",
            EntryData::new(
                "dialogue",
                BTreeMap::from([("name".into(), Value::text("Hello"))]),
            ),
        )],
    );
    let encoded = encode_page(&page).unwrap();
    assert_eq!(decode_page(&encoded).unwrap(), page);
}
