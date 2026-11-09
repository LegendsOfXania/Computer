mod index;
pub mod query;

pub use index::{SearchIndex, SearchMatch, extension_for_kind};
pub use query::{complete_operator, is_operator, scope};

pub const PAGE: u8 = 1;
pub const ENTRY: u8 = 2;
pub const NEW_ENTRY: u8 = 4;
