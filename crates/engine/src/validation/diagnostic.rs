use crate::ValidationError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    page_id: String,
    entry_id: String,
    error: ValidationError,
}

impl ValidationDiagnostic {
    #[inline]
    pub fn new(
        page_id: impl Into<String>,
        entry_id: impl Into<String>,
        error: ValidationError,
    ) -> Self {
        Self {
            page_id: page_id.into(),
            entry_id: entry_id.into(),
            error,
        }
    }

    #[inline]
    pub fn page_id(&self) -> &str {
        &self.page_id
    }

    #[inline]
    pub fn entry_id(&self) -> &str {
        &self.entry_id
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