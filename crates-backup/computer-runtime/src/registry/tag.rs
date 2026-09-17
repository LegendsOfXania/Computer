use super::RegistryError;
use model::Field;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq)]
pub struct TagDefinition {
    name: String,
    fields: Vec<Field>,
}
impl TagDefinition {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
        }
    }
    pub fn field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }
}
#[derive(Debug, Default)]
pub struct TagRegistry {
    tags: RwLock<HashMap<String, Arc<TagDefinition>>>,
}
impl TagRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, definition: TagDefinition) -> Result<(), RegistryError> {
        let mut tags = self.tags.write().expect("TagRegistry lock poisoned");
        if tags.contains_key(definition.name()) {
            return Err(RegistryError::DuplicateTag(definition.name().to_owned()));
        }
        tags.insert(definition.name().to_owned(), Arc::new(definition));
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Arc<TagDefinition>> {
        self.tags
            .read()
            .expect("TagRegistry lock poisoned")
            .get(name)
            .cloned()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tags
            .read()
            .expect("TagRegistry lock poisoned")
            .contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.tags.read().expect("TagRegistry lock poisoned").len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.tags
            .read()
            .expect("TagRegistry lock poisoned")
            .is_empty()
    }
}
