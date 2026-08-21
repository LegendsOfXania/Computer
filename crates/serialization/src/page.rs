use crate::{Format, KdlFormat, Number, SerializationError, Value};

use model::{EntryData, PageData, PageType};

use std::{collections::BTreeMap, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub struct RawEntry {
    id: String,
    data: EntryData,
}

impl RawEntry {
    #[inline]
    pub fn new(id: impl Into<String>, data: EntryData) -> Self {
        Self {
            id: id.into(),
            data,
        }
    }

    #[inline]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[inline]
    pub fn data(&self) -> &EntryData {
        &self.data
    }

    #[inline]
    pub fn into_parts(self) -> (String, EntryData) {
        (self.id, self.data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawPage {
    id: String,
    data: PageData,
    entries: Vec<RawEntry>,
}

impl RawPage {
    pub fn new(
        id: impl Into<String>,
        data: PageData,
        entries: impl IntoIterator<Item = RawEntry>,
    ) -> Self {
        Self {
            id: id.into(),
            data,
            entries: entries.into_iter().collect(),
        }
    }

    #[inline]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[inline]
    pub fn data(&self) -> &PageData {
        &self.data
    }

    #[inline]
    pub fn entries(&self) -> &[RawEntry] {
        &self.entries
    }

    #[inline]
    pub fn into_parts(self) -> (String, PageData, Vec<RawEntry>) {
        (self.id, self.data, self.entries)
    }
}

pub fn decode_page(input: &str) -> Result<RawPage, SerializationError> {
    let value = KdlFormat::decode(input)?;

    page_from_value(value)
}

pub fn encode_page(page: &RawPage) -> Result<String, SerializationError> {
    KdlFormat::encode(&page_to_value(page))
}

pub fn page_from_value(value: Value) -> Result<RawPage, SerializationError> {
    let mut root = into_struct(value, "page document")?;

    let id = required_text(&mut root, "id", "page document")?;

    let data = root
        .remove("page")
        .ok_or_else(|| SerializationError::invalid_structure("page document is missing `page`"))
        .and_then(decode_page_data)?;

    let entries = match root.remove("entries") {
        Some(Value::List(entries)) => entries,

        _ => {
            return Err(SerializationError::invalid_structure(
                "page document `entries` must be a list",
            ));
        }
    };

    ensure_empty(&root, "page document")?;

    let entries = entries
        .into_iter()
        .map(decode_raw_entry)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(RawPage::new(id, data, entries))
}

fn decode_raw_entry(value: Value) -> Result<RawEntry, SerializationError> {
    let mut values = into_struct(value, "entry")?;

    let id = required_text(&mut values, "id", "entry")?;

    let entry_type = required_enum(&mut values, "type", "entry")?;

    let fields = values
        .remove("fields")
        .ok_or_else(|| SerializationError::invalid_structure("entry is missing `fields`"))
        .and_then(|value| into_struct(value, "entry `fields`"))?;

    ensure_empty(&values, "entry")?;

    Ok(RawEntry::new(id, EntryData::new(entry_type, fields)))
}

fn decode_page_data(value: Value) -> Result<PageData, SerializationError> {
    let mut values = into_struct(value, "`page`")?;

    let name = required_text(&mut values, "name", "page")?;

    let page_type = required_enum(&mut values, "type", "page").and_then(|value| {
        PageType::from_str(&value).map_err(|_| {
            SerializationError::invalid_structure(format!("unknown page type `{value}`"))
        })
    })?;

    let priority = match values.remove("priority") {
        Some(Value::Number(Number::Integer(value))) => u32::try_from(value).map_err(|_| {
            SerializationError::invalid_structure("page `priority` must fit in u32")
        })?,

        _ => {
            return Err(SerializationError::invalid_structure(
                "page `priority` must be an integer",
            ));
        }
    };

    ensure_empty(&values, "page")?;

    Ok(PageData::new(name, page_type, priority))
}

pub fn page_to_value(raw_page: &RawPage) -> Value {
    let data = raw_page.data();

    let page_value = Value::structure(BTreeMap::from([
        ("name".into(), Value::text(data.name())),
        ("type".into(), Value::enumeration(data.page_type().as_str())),
        (
            "priority".into(),
            Value::integer(i64::from(data.priority())),
        ),
    ]));

    let entries = raw_page.entries().iter().map(raw_entry_to_value).collect();

    Value::structure(BTreeMap::from([
        ("id".into(), Value::text(raw_page.id())),
        ("page".into(), page_value),
        ("entries".into(), Value::list(entries)),
    ]))
}

fn raw_entry_to_value(entry: &RawEntry) -> Value {
    Value::structure(BTreeMap::from([
        ("id".into(), Value::text(entry.id())),
        ("type".into(), Value::enumeration(entry.data().entry_type())),
        (
            "fields".into(),
            Value::structure(entry.data().fields().clone()),
        ),
    ]))
}

fn into_struct(value: Value, context: &str) -> Result<BTreeMap<String, Value>, SerializationError> {
    match value {
        Value::Struct(values) => Ok(values),

        _ => Err(SerializationError::invalid_structure(format!(
            "{context} must be a struct"
        ))),
    }
}

fn required_text(
    values: &mut BTreeMap<String, Value>,
    field: &str,
    context: &str,
) -> Result<String, SerializationError> {
    match values.remove(field) {
        Some(Value::Text(value)) if !value.trim().is_empty() => Ok(value),

        _ => Err(SerializationError::invalid_structure(format!(
            "{context} `{field}` must be a non-empty string"
        ))),
    }
}

fn required_enum(
    values: &mut BTreeMap<String, Value>,
    field: &str,
    context: &str,
) -> Result<String, SerializationError> {
    match values.remove(field) {
        Some(Value::Enum(value)) if !value.trim().is_empty() => Ok(value),

        _ => Err(SerializationError::invalid_structure(format!(
            "{context} `{field}` must be a non-empty enum"
        ))),
    }
}

fn ensure_empty(values: &BTreeMap<String, Value>, context: &str) -> Result<(), SerializationError> {
    if let Some(field) = values.keys().next() {
        return Err(SerializationError::invalid_structure(format!(
            "{context} has an unknown field `{field}`"
        )));
    }

    Ok(())
}
