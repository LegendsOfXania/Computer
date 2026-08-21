use serialization::SerializationError;

use std::{
    error::Error,
    fmt,
    io,
};

#[derive(Debug)]
pub enum StorageError {
    Io(io::Error),
    Serialization(SerializationError),
}

impl fmt::Display for StorageError {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),

            Self::Serialization(error) => error.fmt(formatter),
        }
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),

            Self::Serialization(error) => Some(error),
        }
    }
}

impl From<io::Error> for StorageError {
    #[inline]
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<SerializationError> for StorageError {
    #[inline]
    fn from(error: SerializationError) -> Self {
        Self::Serialization(error)
    }
}