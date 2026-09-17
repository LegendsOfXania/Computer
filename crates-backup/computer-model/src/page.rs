use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageType {
    Sequence,
    Static,
}

impl PageType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sequence => "sequence",
            Self::Static => "static",
        }
    }
}

impl fmt::Display for PageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PageType {
    type Err = ();
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "sequence" => Ok(Self::Sequence),
            "static" => Ok(Self::Static),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageData {
    name: String,
    page_type: PageType,
    priority: u32,
}

impl PageData {
    pub fn new(name: impl Into<String>, page_type: PageType, priority: u32) -> Self {
        Self {
            name: name.into(),
            page_type,
            priority,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub const fn page_type(&self) -> PageType {
        self.page_type
    }
    pub const fn priority(&self) -> u32 {
        self.priority
    }
}
