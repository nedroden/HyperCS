//! A `Json` extractor that runs `validator` and returns the problem+json
//! shape on failure instead of axum's default JSON rejection.

use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
};
use hypercs_domain::{DomainError, FieldError};
use validator::Validate;

use crate::{error::ApiError, state::AppState};

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: Validate + for<'de> serde::Deserialize<'de>,
    S: Send + Sync,
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(json_rejection_to_error)?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

fn json_rejection_to_error(rejection: JsonRejection) -> ApiError {
    ApiError::Domain(DomainError::Validation(vec![FieldError {
        field: "body",
        code: "invalid_json",
        message: rejection.body_text(),
    }]))
}
