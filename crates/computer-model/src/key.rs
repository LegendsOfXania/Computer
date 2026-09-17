use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntryKey(pub u64, pub u64);

impl EntryKey {
    pub const fn new(page_id: u64, entry_id: u64) -> Self {
        Self(page_id, entry_id)
    }

    pub const fn page_id(self) -> u64 {
        self.0
    }

    pub const fn entry_id(self) -> u64 {
        self.1
    }
}