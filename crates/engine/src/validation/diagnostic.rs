use runtime::EntryKey;

use crate::ValidationError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    entry_key: EntryKey,
    error: ValidationError,
}

impl ValidationDiagnostic {
    #[inline]
    pub fn new(
        entry_key: EntryKey,
        error: ValidationError,
    ) -> Self {
        Self {
            entry_key: entry_key,
            error,
        }
    }

    #[inline]
    pub fn error(&self) -> &ValidationError {
        &self.error
    }

    #[inline]
    pub fn into_error(self) -> ValidationError {
        self.error
    }
}