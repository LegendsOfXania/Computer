use std::fmt;

#[derive(Debug)]
pub enum SerializationError {
    Parse(String),
    InvalidStructure(String),
    UnsupportedType(String),
    IntegerOutOfRange(i128),
}

impl SerializationError {
    pub fn invalid_structure(message: impl Into<String>) -> Self {
        Self::InvalidStructure(message.into())
    }

    pub fn unsupported_type(ty: impl Into<String>) -> Self {
        Self::UnsupportedType(ty.into())
    }
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(message) => {
                write!(f, "failed to parse serialized data: {message}")
            }

            Self::InvalidStructure(message) => {
                write!(f, "invalid serialized structure: {message}")
            }

            Self::UnsupportedType(ty) => {
                write!(f, "unsupported serialized type `{ty}`")
            }

            Self::IntegerOutOfRange(value) => {
                write!(
                    f,
                    "integer `{value}` is outside the supported i64 range"
                )
            }
        }
    }
}

impl std::error::Error for SerializationError {}