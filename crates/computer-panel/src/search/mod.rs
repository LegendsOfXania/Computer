
mod index;
mod query;

pub use index::{Fields, SearchIndex, extension};
pub use query::{highlight, scope};

pub const PAGE: u8 = 1;
pub const ENTRY: u8 = 2;
pub const NEW_ENTRY: u8 = 4;
pub const PAGE_PICK: u8 = 8;
