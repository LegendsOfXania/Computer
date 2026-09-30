use std::collections::{HashMap, HashSet};

use computer_model::{
    EntryDefinition,
    Registry as ModelRegistry,
    field::FieldDefinition,
};

use crate::EntryBlueprint;

#[derive(Debug)]
pub enum RegistryError {
    DuplicateEntry {
        kind: String,
    },
    MissingBase {
        kind: String,
        base: String,
    },
    BaseCycle {
        kind: String,
    },
    DuplicateField {
        kind: String,
        field: String,
    },
}

pub struct Registry {
    entries: HashMap<&'static str, EntryBlueprint>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        blueprint: EntryBlueprint,
    ) -> Result<(), RegistryError> {
        if self.entries.contains_key(blueprint.kind) {
            return Err(RegistryError::DuplicateEntry {
                kind: blueprint.kind.to_owned(),
            });
        }

        self.entries.insert(blueprint.kind, blueprint);

        Ok(())
    }

    pub fn get(&self, kind: &str) -> Option<&EntryBlueprint> {
        self.entries.get(kind)
    }

    pub fn contains(&self, kind: &str) -> bool {
        self.entries.contains_key(kind)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn build(&self) -> Result<ModelRegistry, RegistryError> {
        let mut entries = Vec::with_capacity(self.entries.len());

        for blueprint in self.entries.values() {
            let fields = self.resolve_fields(blueprint.kind)?;

            entries.push(EntryDefinition {
                kind: blueprint.kind.to_owned(),
                version: 0,
                tags: blueprint
                    .tags
                    .iter()
                    .map(|tag| (*tag).to_owned())
                    .collect(),
                fields,
            });
        }

        Ok(ModelRegistry::new(entries))
    }

    fn resolve_fields(
        &self,
        kind: &'static str,
    ) -> Result<Vec<FieldDefinition>, RegistryError> {
        let mut resolving = HashSet::new();

        self.resolve_fields_inner(kind, &mut resolving)
    }

    fn resolve_fields_inner(
        &self,
        kind: &'static str,
        resolving: &mut HashSet<&'static str>,
    ) -> Result<Vec<FieldDefinition>, RegistryError> {
        if !resolving.insert(kind) {
            return Err(RegistryError::BaseCycle {
                kind: kind.to_owned(),
            });
        }

        let result = self.resolve_fields_inner_impl(kind, resolving);

        resolving.remove(kind);

        result
    }

    fn resolve_fields_inner_impl(
        &self,
        kind: &'static str,
        resolving: &mut HashSet<&'static str>,
    ) -> Result<Vec<FieldDefinition>, RegistryError> {
        let blueprint = match self.entries.get(kind) {
            Some(blueprint) => blueprint,
            None => {
                return Err(RegistryError::MissingBase {
                    kind: kind.to_owned(),
                    base: kind.to_owned(),
                });
            }
        };

        let mut fields = match blueprint.base {
            Some(base) => {
                if !self.entries.contains_key(base) {
                    return Err(RegistryError::MissingBase {
                        kind: kind.to_owned(),
                        base: base.to_owned(),
                    });
                }

                self.resolve_fields_inner(base, resolving)?
            }

            None => Vec::new(),
        };

        let mut names = fields
            .iter()
            .map(|field| field.name.clone())
            .collect::<HashSet<_>>();

        for field in &blueprint.fields {
            if !names.insert(field.name.clone()) {
                return Err(RegistryError::DuplicateField {
                    kind: kind.to_owned(),
                    field: field.name.clone(),
                });
            }

            fields.push(field.clone());
        }

        Ok(fields)
    }
}