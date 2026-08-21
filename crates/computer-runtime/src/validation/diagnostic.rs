use super::ValidationError;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    page_id: String,
    entry_id: String,
    error: ValidationError,
}
impl ValidationDiagnostic {
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
    pub fn page_id(&self) -> &str {
        &self.page_id
    }
    pub fn entry_id(&self) -> &str {
        &self.entry_id
    }
    pub fn error(&self) -> &ValidationError {
        &self.error
    }
    pub fn into_error(self) -> ValidationError {
        self.error
    }
}
