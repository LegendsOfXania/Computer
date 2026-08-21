mod error;
mod library;

pub use error::StorageError;
pub use library::{
    FileStorage,
    Storage,
};