use super::{RegistryError, TagRegistry};
use model::{Field, Schema};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub struct EntryDefinition {
    entry_type: String,
    tags: Vec<String>,
    fields: Vec<Field>,
}
impl EntryDefinition {
    pub fn new(entry_type: impl Into<String>) -> Self {
        Self {
            entry_type: entry_type.into(),
            tags: Vec::new(),
            fields: Vec::new(),
        }
    }
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    pub fn field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }
    pub fn entry_type(&self) -> &str {
        &self.entry_type
    }
    pub fn tags(&self) -> &[String] {
        &self.tags
    }
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }
}
fn resolve(
    definition: &EntryDefinition,
    tags: &TagRegistry,
) -> Result<EntryDefinition, RegistryError> {
    let mut fields = vec![Field::new("name", Schema::Text)];
    let mut names = HashSet::from([String::from("name")]);
    for tag_name in definition.tags() {
        let tag = tags
            .get(tag_name)
            .ok_or_else(|| RegistryError::UnknownTag {
                entry_type: definition.entry_type().to_owned(),
                tag: tag_name.clone(),
            })?;
        for field in tag.fields() {
            if !names.insert(field.name().to_owned()) {
                return Err(RegistryError::FieldCollision {
                    entry_type: definition.entry_type().to_owned(),
                    field: field.name().to_owned(),
                });
            }
            fields.push(field.clone());
        }
    }
    for field in definition.fields() {
        if !names.insert(field.name().to_owned()) {
            return Err(RegistryError::FieldCollision {
                entry_type: definition.entry_type().to_owned(),
                field: field.name().to_owned(),
            });
        }
        fields.push(field.clone());
    }
    Ok(EntryDefinition {
        entry_type: definition.entry_type().to_owned(),
        tags: definition.tags().to_vec(),
        fields,
    })
}
#[derive(Debug, Default)]
pub struct EntryRegistry {
    entries: RwLock<HashMap<String, EntryDefinition>>,
}
impl EntryRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn register(
        &self,
        definition: EntryDefinition,
        tags: &TagRegistry,
    ) -> Result<(), RegistryError> {
        let definition = resolve(&definition, tags)?;
        let mut entries = self.entries.write().expect("EntryRegistry lock poisoned");
        if entries.contains_key(definition.entry_type()) {
            return Err(RegistryError::DuplicateEntryType(
                definition.entry_type().to_owned(),
            ));
        }
        entries.insert(definition.entry_type().to_owned(), definition);
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
}
