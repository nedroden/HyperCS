//! Single error type for the whole API, mapped once to RFC 9457
//! `application/problem+json`. Handlers return `ApiResult<T>`.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use hypercs_domain::{DomainError, FieldError};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let mut field_errors = Vec::new();
        for (field, errs) in errors.field_errors() {
            for e in errs {
                let code: &'static str = Box::leak(e.code.to_string().into_boxed_str());
                field_errors.push(FieldError {
                    field: Box::leak(field.to_string().into_boxed_str()),
                    code,
                    message: e
                        .message
                        .clone()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("invalid {field}")),
                });
            }
        }
        ApiError::Domain(DomainError::Validation(field_errors))
    }
}

#[derive(Serialize)]
struct ProblemFieldError {
    field: &'static str,
    code: &'static str,
    message: String,
}

#[derive(Serialize)]
struct Problem {
    r#type: &'static str,
    title: &'static str,
    status: u16,
    detail: String,
    code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<ProblemFieldError>>,
}

impl ApiError {
    fn status_title_detail(&self) -> (StatusCode, &'static str, String) {
        match self {
            ApiError::Domain(DomainError::Validation(_)) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation failed",
                "One or more fields are invalid.".to_string(),
            ),
            ApiError::Domain(DomainError::NotFound) => (
                StatusCode::NOT_FOUND,
                "Not found",
                "The requested resource does not exist.".to_string(),
            ),
            ApiError::Domain(DomainError::Forbidden) => (
                StatusCode::FORBIDDEN,
                "Forbidden",
                "You are not allowed to perform this action.".to_string(),
            ),
            ApiError::Domain(DomainError::Unauthorized) => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
                "You must be logged in to perform this action.".to_string(),
            ),
            ApiError::Domain(DomainError::Conflict { .. }) => (
                StatusCode::CONFLICT,
                "Conflict",
                "The request conflicts with the current state.".to_string(),
            ),
            ApiError::Domain(DomainError::TooManyRequests { .. }) => (
                StatusCode::TOO_MANY_REQUESTS,
                "Too many requests",
                "Please slow down.".to_string(),
            ),
            ApiError::Domain(DomainError::Internal) | ApiError::Sqlx(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error",
                "Something went wrong.".to_string(),
            ),
        }
    }

    fn code(&self) -> &'static str {
        match self {
            ApiError::Domain(e) => e.code(),
            ApiError::Sqlx(_) => "internal_error",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if let ApiError::Sqlx(e) = &self {
            tracing::error!(error = %e, "database error");
        }
        let (status, title, detail) = self.status_title_detail();
        let errors = if let ApiError::Domain(DomainError::Validation(fields)) = &self {
            Some(
                fields
                    .iter()
                    .map(|f| ProblemFieldError {
                        field: f.field,
                        code: f.code,
                        message: f.message.clone(),
                    })
                    .collect(),
            )
        } else {
            None
        };
        let body = Problem {
            r#type: "about:blank",
            title,
            status: status.as_u16(),
            detail,
            code: self.code(),
            errors,
        };
        let mut response = (status, Json(body)).into_response();
        response
            .headers_mut()
            .insert("Content-Type", "application/problem+json".parse().unwrap());
        response
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
