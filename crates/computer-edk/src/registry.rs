use crate::entry::EntryDefinition;

#[derive(Debug, Default)]
pub struct Registry {
    entries: Vec<EntryDefinition>,
}

impl Registry {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn entries(&self) -> &[EntryDefinition] {
        &self.entries
    }
}