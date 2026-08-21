#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    UnknownEntryType {
        entry_type: String,
    },
    MissingField {
        field: String,
    },
    UnknownField {
        field: String,
    },
    InvalidValue {
        field: String,
    },
    InvalidReference {
        field: String,
        reference: String,
    },
    UnknownReference {
        field: String,
        reference: String,
    },
    InvalidReferenceEntryType {
        field: String,
        expected: String,
        actual: String,
    },
    MissingReferenceTag {
        field: String,
        tag: String,
    },
}
