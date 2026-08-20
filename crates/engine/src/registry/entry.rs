
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use model::Field;

use super::{RegistryError, TagRegistry};

#[derive(Debug, Clone, PartialEq)]
pub struct EntryDefinition {
    entry_type: String,
    tags: Vec<String>,
    fields: Vec<Field>,
}

impl EntryDefinition {
    #[inline]
    pub fn new(entry_type: impl Into<String>) -> Self {
        Self {
            entry_type: entry_type.into(),
            tags: Vec::new(),
            fields: Vec::new(),
        }
    }

    #[inline]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    #[inline]
    pub fn field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    #[inline]
    pub fn entry_type(&self) -> &str {
        &self.entry_type
    }

    #[inline]
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    #[inline]
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }
}

fn universal_name_field() -> Field {
    Field::new("name", model::Schema::Text)
}

fn resolve(
    definition: &EntryDefinition,
    tags: &TagRegistry,
) -> Result<EntryDefinition, RegistryError> {
    let mut resolved_fields = vec![universal_name_field()];
    let mut seen: HashSet<String> = HashSet::from(["name".to_owned()]);

    for tag_name in definition.tags() {
        let tag = tags.get(tag_name).ok_or_else(|| RegistryError::UnknownTag {
            entry_type: definition.entry_type().to_owned(),
            tag: tag_name.clone(),
        })?;

        for field in tag.fields() {
            if !seen.insert(field.name().to_owned()) {
                return Err(RegistryError::FieldCollision {
                    entry_type: definition.entry_type().to_owned(),
                    field: field.name().to_owned(),
                });
            }
            resolved_fields.push(field.clone());
        }
    }

    for field in definition.fields() {
        if !seen.insert(field.name().to_owned()) {
            return Err(RegistryError::FieldCollision {
                entry_type: definition.entry_type().to_owned(),
                field: field.name().to_owned(),
            });
        }
        resolved_fields.push(field.clone());
    }

    Ok(EntryDefinition {
        entry_type: definition.entry_type().to_owned(),
        tags: definition.tags().to_vec(),
        fields: resolved_fields,
    })
}

#[derive(Debug, Default)]
pub struct EntryRegistry {
    entries: RwLock<HashMap<String, EntryDefinition>>,
}

impl EntryRegistry {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers `definition`, resolving its tags against `tags` first.
    /// Prefer [`super::Registry::register_entry`], which supplies its own
    /// `TagRegistry` automatically -- this exists mainly so `EntryRegistry`
    /// stays independently testable, the same way `runtime::EntryStore` does.
    pub fn register(
        &self,
        definition: EntryDefinition,
        tags: &TagRegistry,
    ) -> Result<(), RegistryError> {
        let resolved = resolve(&definition, tags)?;

        let mut entries = self.entries.write().expect("EntryRegistry lock poisoned");
        if entries.contains_key(resolved.entry_type()) {
            return Err(RegistryError::DuplicateEntry(
                resolved.entry_type().to_owned(),
            ));
        }
        entries.insert(resolved.entry_type().to_owned(), resolved);
        Ok(())
    }

    pub fn get(&self, entry_type: &str) -> Option<EntryDefinition> {
        self.entries
            .read()
            .expect("EntryRegistry lock poisoned")
            .get(entry_type)
            .cloned()
    }

    pub fn contains(&self, entry_type: &str) -> bool {
        self.entries
            .read()
            .expect("EntryRegistry lock poisoned")
            .contains_key(entry_type)
    }

    pub fn len(&self) -> usize {
        self.entries
            .read()
            .expect("EntryRegistry lock poisoned")
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries
            .read()
            .expect("EntryRegistry lock poisoned")
            .is_empty()
    }

    pub fn clear(&self) {
        self.entries
            .write()
            .expect("EntryRegistry lock poisoned")
            .clear();
    }
}