mod entry;
mod error;
mod tag;

pub use entry::{EntryDefinition, EntryRegistry};
pub use error::RegistryError;
pub use tag::{TagDefinition, TagRegistry};

use std::sync::LazyLock;

#[derive(Debug, Default)]
pub struct Registry {
    tags: TagRegistry,
    entries: EntryRegistry,
}
static REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);
impl Registry {
    pub fn global() -> &'static Self {
        &REGISTRY
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn tags(&self) -> &TagRegistry {
        &self.tags
    }
    pub fn entries(&self) -> &EntryRegistry {
        &self.entries
    }
    pub fn register_tag(&self, definition: TagDefinition) -> Result<(), RegistryError> {
        self.tags.register(definition)
    }
    pub fn register_entry(&self, definition: EntryDefinition) -> Result<(), RegistryError> {
        self.entries.register(definition, &self.tags)
    }
}
