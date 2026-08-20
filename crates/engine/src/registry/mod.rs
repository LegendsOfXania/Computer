pub mod entry;
pub mod error;
pub mod tag;

pub use entry::{EntryDefinition, EntryRegistry};
pub use error::RegistryError;
pub use tag::{TagDefinition, TagRegistry};

#[derive(Debug, Default)]
pub struct Registry {
    tags: TagRegistry,
    entries: EntryRegistry,
}

impl Registry {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn tags(&self) -> &TagRegistry {
        &self.tags
    }

    #[inline]
    pub fn entries(&self) -> &EntryRegistry {
        &self.entries
    }

    #[inline]
    pub fn register_tag(&self, definition: TagDefinition) -> Result<(), RegistryError> {
        self.tags.register(definition)
    }

    #[inline]
    pub fn register_entry(&self, definition: EntryDefinition) -> Result<(), RegistryError> {
        self.entries.register(definition, &self.tags)
    }
}