//! Extractors whose rejections are problem+json instead of axum's plain-text
//! defaults.

use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Request},
};
use validator::Validate;

use crate::error::ApiError;

/// `Json<T>` + `validator`: parse errors and validation failures both become
/// problem+json (400/413/415/422).
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: Validate + for<'de> serde::Deserialize<'de>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

/// `Path<T>` with a problem+json rejection (a malformed id is a 404).
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct ApiPath<T>(pub T);

/// `Query<T>` with a problem+json rejection (400 `invalid_query`).
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(ApiError))]
pub struct ApiQuery<T>(pub T);
