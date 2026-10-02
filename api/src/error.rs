use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Application error, mapped to the API's single JSON error shape.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)] // variants are used as endpoints are added
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("internal error")]
    Internal(#[from] sqlx::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    /// Machine-readable error code.
    code: &'static str,
    /// Human-readable message (not localized; the front-end translates by `code`).
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
            AppError::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
        };
        (
            status,
            Json(ErrorBody {
                code,
                message: self.to_string(),
            }),
        )
            .into_response()
    }
}
