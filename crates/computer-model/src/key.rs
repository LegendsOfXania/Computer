use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
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

impl fmt::Display for EntryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}