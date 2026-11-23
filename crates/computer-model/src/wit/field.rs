use std::{collections::BTreeMap, str::FromStr};

use super::{EntryDefinition, Field, FieldKind, FieldNode};

impl Field {
    #[must_use]
    pub fn new(name: impl Into<String>, kind: FieldKind) -> Self {
        Self {
            name: name.into(),
            flags: BTreeMap::new(),
            kind,
        }
    }

    #[must_use]
    pub fn flag(&self, key: &str) -> Option<&str> {
        self.flags.get(key).map(String::as_str)
    }

    #[must_use]
    pub fn has_flag(&self, key: &str) -> bool {
        self.flag(key).is_some_and(|value| value != "false")
    }

    #[must_use]
    pub fn bound<T: FromStr>(&self, key: &str) -> Option<T> {
        self.flag(key)?.parse().ok()
    }
}

impl<'a> FieldNode<'a> {
    pub fn cases(&self) -> impl Iterator<Item = (&'a str, Vec<Self>)> + 'a {
        let mut children = self.children();
        let cases: &[(String, u32)] = match self.value().map(|field| &field.kind) {
            Some(FieldKind::Variant(cases)) => cases,
            _ => &[],
        };

        cases.iter().map(move |(name, count)| {
            (
                name.as_str(),
                children.by_ref().take(*count as usize).collect(),
            )
        })
    }
}

impl EntryDefinition {
    pub fn members(&self) -> impl Iterator<Item = FieldNode<'_>> {
        FieldNode::forest(&self.fields)
    }

    #[must_use]
    pub fn field(&self, name: &str) -> Option<FieldNode<'_>> {
        self.members()
            .find(|node| node.value().is_some_and(|field| field.name == name))
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        FieldNode::trees(&self.fields).is_some()
    }
}
