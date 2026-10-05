//! Domain-level errors. The api crate maps these to RFC 9457 problem+json.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("validation failed")]
    Validation(Vec<FieldError>),
    #[error("not found")]
    NotFound,
    #[error("forbidden")]
    Forbidden,
    #[error("unauthorized")]
    Unauthorized,
    #[error("conflict: {code}")]
    Conflict { code: &'static str },
    #[error("too many requests: {code}")]
    TooManyRequests { code: &'static str },
    #[error("internal error")]
    Internal,
}

#[derive(Debug, Clone)]
pub struct FieldError {
    pub field: &'static str,
    pub code: &'static str,
    pub message: String,
}

impl DomainError {
    pub fn validation(field: &'static str, code: &'static str, message: impl Into<String>) -> Self {
        DomainError::Validation(vec![FieldError {
            field,
            code,
            message: message.into(),
        }])
    }

    pub fn code(&self) -> &'static str {
        match self {
            DomainError::Validation(_) => "validation_failed",
            DomainError::NotFound => "not_found",
            DomainError::Forbidden => "forbidden",
            DomainError::Unauthorized => "unauthorized",
            DomainError::Conflict { code } => code,
            DomainError::TooManyRequests { code } => code,
            DomainError::Internal => "internal_error",
        }
    }
}

pub type DomainResult<T> = Result<T, DomainError>;
