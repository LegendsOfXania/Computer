mod matcher;
mod operator;
mod parser;

pub use operator::{complete_operator, is_operator};
pub use parser::{Query, SearchField, scope};
pub(crate) use matcher::{match_indices, patterns};
