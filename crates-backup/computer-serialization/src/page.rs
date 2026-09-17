use crate::{SerializationError, kdl::{decode_node, encode_node}};

use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

use model::{EntryData, PageData, PageType, Value};

use std::{
    collections::{BTreeMap, btree_map::Entry},
    str::FromStr,
};

#[derive(Debug, Clone, PartialEq)]
pub struct RawEntry {
    id: String,
    data: EntryData,
}

impl RawEntry {
    #[inline]
    pub fn new(
        id: impl Into<String>,
        data: EntryData,
    ) -> Self {
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

pub fn decode_page(
    input: &str,
) -> Result<RawPage, SerializationError> {
    let document = input
        .parse::<KdlDocument>()
        .map_err(|error| SerializationError::Parse(error.to_string()))?;

    decode_page_document(&document)
}

pub fn encode_page(
    page: &RawPage,
) -> Result<String, SerializationError> {
    Ok(encode_page_document(page).to_string())
}

fn decode_page_document(
    document: &KdlDocument,
) -> Result<RawPage, SerializationError> {
    let nodes = document.nodes();

    if nodes.len() != 1 {
        return Err(SerializationError::invalid_structure(
            "a page document must contain exactly one `page` node",
        ));
    }

    decode_page_node(&nodes[0])
}

fn decode_page_node(
    node: &KdlNode,
) -> Result<RawPage, SerializationError> {
    if node.name().value() != "page" {
        return Err(SerializationError::invalid_structure(
            "the root node must be `page`",
        ));
    }

    if node.ty().is_some() {
        return Err(SerializationError::invalid_structure(
            "page cannot have a type annotation",
        ));
    }

    let mut properties = collect_properties(node, "page")?;

    let id = required_text(&mut properties, "id", "page")?;
    let name = required_text(&mut properties, "name", "page")?;

    let page_type = required_text(&mut properties, "type", "page")
        .and_then(|value| {
            PageType::from_str(&value).map_err(|_| {
                SerializationError::invalid_structure(
                    format!("unknown page type `{value}`"),
                )
            })
        })?;

    let priority = required_u32(
        &mut properties,
        "priority",
        "page",
    )?;

    ensure_empty(&properties, "page")?;

    let children = node.children().ok_or_else(|| {
        SerializationError::invalid_structure(
            "page must contain an entry document",
        )
    })?;

    let entries = children
        .nodes()
        .iter()
        .map(decode_entry_node)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(RawPage::new(
        id,
        PageData::new(name, page_type, priority),
        entries,
    ))
}

fn decode_entry_node(
    node: &KdlNode,
) -> Result<RawEntry, SerializationError> {
    if node.name().value() != "entry" {
        return Err(SerializationError::invalid_structure(format!(
            "page can only contain `entry` nodes, found `{}`",
            node.name().value(),
        )));
    }

    if node.ty().is_some() {
        return Err(SerializationError::invalid_structure(
            "entry cannot have a type annotation",
        ));
    }

    let mut properties = collect_properties(node, "entry")?;

    let id = required_text(
        &mut properties,
        "id",
        "entry",
    )?;

    let entry_type = required_text(
        &mut properties,
        "type",
        "entry",
    )?;

    ensure_empty(&properties, "entry")?;

    let fields = match node.children() {
        Some(children) => decode_fields(children)?,
        None => BTreeMap::new(),
    };

    Ok(RawEntry::new(
        id,
        EntryData::new(entry_type, fields),
    ))
}

fn decode_fields(
    document: &KdlDocument,
) -> Result<BTreeMap<String, Value>, SerializationError> {
    let mut fields = BTreeMap::new();

    for node in document.nodes() {
        let name = node.name().value();

        let value = decode_node(node)?;

        match fields.entry(name.to_owned()) {
            Entry::Vacant(entry) => {
                entry.insert(value);
            }

            Entry::Occupied(_) => {
                return Err(SerializationError::invalid_structure(
                    format!("duplicate field `{name}`"),
                ));
            }
        }
    }

    Ok(fields)
}

fn collect_properties(
    node: &KdlNode,
    context: &str,
) -> Result<BTreeMap<String, KdlValue>, SerializationError> {
    let mut properties = BTreeMap::new();

    for entry in node.entries() {
        let Some(name) = entry.name() else {
            return Err(SerializationError::invalid_structure(
                format!(
                    "{context} cannot contain positional values"
                ),
            ));
        };

        match properties.entry(name.value().to_owned()) {
            Entry::Vacant(property) => {
                property.insert(entry.value().clone());
            }

            Entry::Occupied(_) => {
                return Err(SerializationError::invalid_structure(
                    format!(
                        "{context} contains duplicate property `{}`",
                        name.value(),
                    ),
                ));
            }
        }
    }

    Ok(properties)
}

fn required_text(
    properties: &mut BTreeMap<String, KdlValue>,
    name: &str,
    context: &str,
) -> Result<String, SerializationError> {
    match properties.remove(name) {
        Some(KdlValue::String(value))
            if !value.trim().is_empty() =>
        {
            Ok(value)
        }

        Some(_) => Err(
            SerializationError::invalid_structure(
                format!(
                    "{context} property `{name}` must be a non-empty string"
                ),
            ),
        ),

        None => Err(
            SerializationError::invalid_structure(
                format!(
                    "{context} is missing required property `{name}`"
                ),
            ),
        ),
    }
}

fn required_u32(
    properties: &mut BTreeMap<String, KdlValue>,
    name: &str,
    context: &str,
) -> Result<u32, SerializationError> {
    match properties.remove(name) {
        Some(KdlValue::Integer(value)) => {
            u32::try_from(value).map_err(|_| {
                SerializationError::invalid_structure(
                    format!(
                        "{context} property `{name}` must fit in u32"
                    ),
                )
            })
        }

        Some(_) => Err(
            SerializationError::invalid_structure(
                format!(
                    "{context} property `{name}` must be an integer"
                ),
            ),
        ),

        None => Err(
            SerializationError::invalid_structure(
                format!(
                    "{context} is missing required property `{name}`"
                ),
            ),
        ),
    }
}

fn ensure_empty(
    properties: &BTreeMap<String, KdlValue>,
    context: &str,
) -> Result<(), SerializationError> {
    if let Some(name) = properties.keys().next() {
        return Err(
            SerializationError::invalid_structure(
                format!(
                    "{context} has an unknown property `{name}`"
                ),
            ),
        );
    }

    Ok(())
}

fn encode_page_document(
    page: &RawPage,
) -> KdlDocument {
    let mut document = KdlDocument::new();

    document
        .nodes_mut()
        .push(encode_page_node(page));

    document
}

fn encode_page_node(
    page: &RawPage,
) -> KdlNode {
    let mut node = KdlNode::new("page");

    let data = page.data();

    node.entries_mut().extend([
        KdlEntry::new_prop("id", page.id()),
        KdlEntry::new_prop("name", data.name()),
        KdlEntry::new_prop(
            "type",
            data.page_type().as_str(),
        ),
        KdlEntry::new_prop(
            "priority",
            i128::from(data.priority()),
        ),
    ]);

    let mut children = KdlDocument::new();

    children
        .nodes_mut()
        .extend(
            page.entries()
                .iter()
                .map(encode_entry_node),
        );

    node.set_children(children);

    node
}

fn encode_entry_node(
    entry: &RawEntry,
) -> KdlNode {
    let mut node = KdlNode::new("entry");

    node.entries_mut().extend([
        KdlEntry::new_prop("id", entry.id()),
        KdlEntry::new_prop(
            "type",
            entry.data().entry_type(),
        ),
    ]);

    let mut children = KdlDocument::new();

    children
        .nodes_mut()
        .extend(
            entry
                .data()
                .fields()
                .iter()
                .map(|(name, value)| {
                    encode_node(name, value)
                }),
        );

    node.set_children(children);

    node
}