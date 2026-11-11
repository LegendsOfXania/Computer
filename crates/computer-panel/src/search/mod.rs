mod index;
mod query;

pub use index::{Fields, SearchIndex, extension};
pub use query::{complete, highlight, scope};

pub const PAGE: u8 = 1;
pub const ENTRY: u8 = 2;
pub const NEW_ENTRY: u8 = 4;
