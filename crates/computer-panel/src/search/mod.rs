mod index;
mod items;
mod query;

pub use index::{Fields, SearchIndex, extension};
pub use items::{SearchItem, Target, build_index, build_sections};
pub use query::{highlight, scope};

pub const PAGE: u8 = 1;
pub const ENTRY: u8 = 2;
pub const NEW_ENTRY: u8 = 4;
