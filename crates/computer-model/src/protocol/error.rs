use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolError {
    AlreadyExists,
    InvalidValue,
    InvalidReference,
    InvalidRequest,
    NotFound,
    UnknownDefinition,
    PermissionDenied,
    Internal,
}