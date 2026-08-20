use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    DuplicateTag(String),
    DuplicateEntry(String),
    UnknownTag { entry_type: String, tag: String },
    FieldCollision { entry_type: String, field: String },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateTag(name) => {
                write!(f, "tag '{name}' is already registered")
            }
            Self::DuplicateEntry(entry_type) => {
                write!(f, "entry type '{entry_type}' is already registered")
            }
            Self::UnknownTag { entry_type, tag } => {
                write!(
                    f,
                    "entry type '{entry_type}' references unknown tag '{tag}'"
                )
            }
            Self::FieldCollision { entry_type, field } => {
                write!(
                    f,
                    "entry type '{entry_type}' has a field name collision on '{field}' \
                     (own field, a tag's field, or the universal 'name' field)"
                )
            }
        }
    }
}

impl std::error::Error for RegistryError {}