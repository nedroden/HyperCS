//! Domain-level errors. The api crate maps these to RFC 9457 problem+json.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("validation failed")]
    Validation(Vec<FieldError>),
    #[error("bad request: {code}")]
    BadRequest { code: &'static str },
    #[error("not found")]
    NotFound,
    #[error("forbidden")]
    Forbidden,
    #[error("unauthorized: {code}")]
    Unauthorized { code: &'static str },
    #[error("conflict: {code}")]
    Conflict { code: &'static str },
    #[error("method not allowed")]
    MethodNotAllowed,
    #[error("payload too large")]
    PayloadTooLarge,
    #[error("unsupported media type")]
    UnsupportedMediaType,
    #[error("too many requests: {code}")]
    TooManyRequests { code: &'static str },
    #[error("internal error")]
    Internal,
}

#[derive(Debug, Clone)]
pub struct FieldError {
    pub field: String,
    pub code: String,
    pub message: String,
}

impl DomainError {
    pub fn validation(field: &str, code: &str, message: impl Into<String>) -> Self {
        DomainError::Validation(vec![FieldError {
            field: field.to_string(),
            code: code.to_string(),
            message: message.into(),
        }])
    }

    /// The default 401: no (valid) session.
    pub fn unauthenticated() -> Self {
        DomainError::Unauthorized {
            code: "unauthorized",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            DomainError::Validation(_) => "validation_failed",
            DomainError::BadRequest { code } => code,
            DomainError::NotFound => "not_found",
            DomainError::Forbidden => "forbidden",
            DomainError::Unauthorized { code } => code,
            DomainError::Conflict { code } => code,
            DomainError::MethodNotAllowed => "method_not_allowed",
            DomainError::PayloadTooLarge => "payload_too_large",
            DomainError::UnsupportedMediaType => "unsupported_media_type",
            DomainError::TooManyRequests { code } => code,
            DomainError::Internal => "internal_error",
        }
    }
}

pub type DomainResult<T> = Result<T, DomainError>;
