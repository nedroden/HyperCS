//! Single error type for the whole API, mapped once to RFC 9457
//! `application/problem+json`. Handlers and services return `ApiResult<T>`.
//!
//! The API returns stable machine-readable `code`s; translating them into
//! user-facing text is the front-end's job. `detail`/`message` are English
//! developer-facing text only.

use axum::{
    Json,
    extract::rejection::{JsonRejection, PathRejection, QueryRejection},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use hypercs_domain::{DomainError, FieldError};
use serde::Serialize;
use utoipa::{IntoResponses, ToSchema};
use validator::{ValidationErrors, ValidationErrorsKind};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    Sqlx(sqlx::Error),
}

pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    pub fn not_found() -> Self {
        DomainError::NotFound.into()
    }
    pub fn forbidden() -> Self {
        DomainError::Forbidden.into()
    }
    pub fn unauthenticated() -> Self {
        DomainError::unauthenticated().into()
    }
    pub fn internal() -> Self {
        DomainError::Internal.into()
    }
    pub fn validation(field: &str, code: &str, message: impl Into<String>) -> Self {
        DomainError::validation(field, code, message).into()
    }
    pub fn conflict(code: &'static str) -> Self {
        DomainError::Conflict { code }.into()
    }
}

/// Unique-violation races (two concurrent registrations) surface as the same
/// field errors as the up-front checks.
impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        if let Some(db) = error.as_database_error()
            && db.is_unique_violation()
        {
            match db.constraint() {
                Some("users_username_lower_idx") => {
                    return ApiError::validation(
                        "username",
                        "username_taken",
                        "This username is already taken.",
                    );
                }
                Some("users_email_lower_idx") => {
                    return ApiError::validation(
                        "email",
                        "email_taken",
                        "This email address is already registered.",
                    );
                }
                _ => {}
            }
        }
        ApiError::Sqlx(error)
    }
}

fn camel_case(field: &str) -> String {
    let mut out = String::with_capacity(field.len());
    let mut upper = false;
    for c in field.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

fn param_u64(error: &validator::ValidationError, name: &str) -> Option<u64> {
    error.params.get(name).and_then(|v| v.as_u64())
}

fn default_message(error: &validator::ValidationError) -> String {
    if let Some(message) = &error.message {
        return message.to_string();
    }
    match error.code.as_ref() {
        "length" => match (param_u64(error, "min"), param_u64(error, "max")) {
            (Some(min), Some(max)) => format!("Must be between {min} and {max} characters."),
            (Some(min), None) => format!("Must be at least {min} characters."),
            (None, Some(max)) => format!("Must be at most {max} characters."),
            _ => "Invalid length.".to_string(),
        },
        "email" => "Must be a valid email address.".to_string(),
        "url" => "Must be a valid URL.".to_string(),
        code => format!("Invalid value ({code})."),
    }
}

fn collect_field_errors(prefix: &str, errors: &ValidationErrors, out: &mut Vec<FieldError>) {
    for (name, kind) in errors.errors() {
        let path = if prefix.is_empty() {
            camel_case(name)
        } else {
            format!("{prefix}.{}", camel_case(name))
        };
        match kind {
            ValidationErrorsKind::Field(list) => {
                for e in list {
                    out.push(FieldError {
                        field: path.clone(),
                        code: e.code.to_string(),
                        message: default_message(e),
                    });
                }
            }
            ValidationErrorsKind::Struct(nested) => collect_field_errors(&path, nested, out),
            ValidationErrorsKind::List(items) => {
                for (index, nested) in items {
                    collect_field_errors(&format!("{path}[{index}]"), nested, out);
                }
            }
        }
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> Self {
        let mut fields = Vec::new();
        collect_field_errors("", &errors, &mut fields);
        // HashMap iteration order is unspecified; keep responses stable.
        fields.sort_by(|a, b| (&a.field, &a.code).cmp(&(&b.field, &b.code)));
        DomainError::Validation(fields).into()
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::MissingJsonContentType(_) => DomainError::UnsupportedMediaType.into(),
            JsonRejection::JsonSyntaxError(_) => DomainError::BadRequest {
                code: "invalid_json",
            }
            .into(),
            // Well-formed JSON of the wrong shape: missing/unknown field or
            // wrong type.
            JsonRejection::JsonDataError(e) => {
                ApiError::validation("body", "invalid_body", e.body_text())
            }
            other if other.status() == StatusCode::PAYLOAD_TOO_LARGE => {
                DomainError::PayloadTooLarge.into()
            }
            _ => DomainError::BadRequest {
                code: "invalid_body",
            }
            .into(),
        }
    }
}

impl From<PathRejection> for ApiError {
    fn from(_: PathRejection) -> Self {
        // Malformed ids (not a UUID) cannot name an existing resource.
        DomainError::NotFound.into()
    }
}

impl From<QueryRejection> for ApiError {
    fn from(_: QueryRejection) -> Self {
        DomainError::BadRequest {
            code: "invalid_query",
        }
        .into()
    }
}

/// One field-level problem inside a `422 validation_failed` response.
#[derive(Debug, Serialize, ToSchema)]
pub struct FieldProblem {
    /// JSON path of the offending field, camelCase (e.g. `username`,
    /// `features[1].title`; `body` for a malformed request body).
    #[schema(example = "username")]
    pub field: String,
    /// Stable, machine-readable code (`length`, `email`, `url`,
    /// `username_taken`, `email_taken`, `invalid_characters`, ...).
    #[schema(example = "username_taken")]
    pub code: String,
    /// English developer-facing message; the front-end translates by `code`.
    pub message: String,
}

/// RFC 9457 problem details. Every error response of the API has this shape
/// and the `application/problem+json` content type.
#[derive(Debug, Serialize, ToSchema)]
pub struct ProblemDetails {
    /// Always `about:blank`; the stable identifier is `code`.
    #[serde(rename = "type")]
    #[schema(example = "about:blank")]
    pub type_: String,
    #[schema(example = "Validation failed")]
    pub title: String,
    #[schema(example = 422)]
    pub status: u16,
    pub detail: String,
    /// Stable, machine-readable error code (e.g. `validation_failed`,
    /// `not_found`, `forbidden`, `invalid_credentials`, `posting_too_fast`).
    #[schema(example = "validation_failed")]
    pub code: String,
    /// Field-level errors; present for validation failures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<FieldProblem>>,
}

impl ApiError {
    fn status_title_detail(&self) -> (StatusCode, &'static str, &'static str) {
        match self {
            ApiError::Domain(DomainError::Validation(_)) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation failed",
                "One or more fields are invalid.",
            ),
            ApiError::Domain(DomainError::BadRequest { .. }) => (
                StatusCode::BAD_REQUEST,
                "Bad request",
                "The request could not be understood.",
            ),
            ApiError::Domain(DomainError::NotFound) => (
                StatusCode::NOT_FOUND,
                "Not found",
                "The requested resource does not exist.",
            ),
            ApiError::Domain(DomainError::Forbidden) => (
                StatusCode::FORBIDDEN,
                "Forbidden",
                "You are not allowed to perform this action.",
            ),
            ApiError::Domain(DomainError::Unauthorized { .. }) => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
                "Authentication is required or the credentials are invalid.",
            ),
            ApiError::Domain(DomainError::Conflict { .. }) => (
                StatusCode::CONFLICT,
                "Conflict",
                "The request conflicts with the current state.",
            ),
            ApiError::Domain(DomainError::MethodNotAllowed) => (
                StatusCode::METHOD_NOT_ALLOWED,
                "Method not allowed",
                "This HTTP method is not supported for this resource.",
            ),
            ApiError::Domain(DomainError::PayloadTooLarge) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "Payload too large",
                "The request body exceeds the size limit for this endpoint.",
            ),
            ApiError::Domain(DomainError::UnsupportedMediaType) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Unsupported media type",
                "The request body must be application/json.",
            ),
            ApiError::Domain(DomainError::TooManyRequests { .. }) => (
                StatusCode::TOO_MANY_REQUESTS,
                "Too many requests",
                "Please slow down and try again shortly.",
            ),
            ApiError::Domain(DomainError::Internal) | ApiError::Sqlx(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error",
                "Something went wrong.",
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

pub fn problem_response(error: &ApiError) -> Response {
    let (status, title, detail) = error.status_title_detail();
    let errors = match error {
        ApiError::Domain(DomainError::Validation(fields)) => Some(
            fields
                .iter()
                .map(|f| FieldProblem {
                    field: f.field.clone(),
                    code: f.code.clone(),
                    message: f.message.clone(),
                })
                .collect(),
        ),
        _ => None,
    };
    let body = ProblemDetails {
        type_: "about:blank".to_string(),
        title: title.to_string(),
        status: status.as_u16(),
        detail: detail.to_string(),
        code: error.code().to_string(),
        errors,
    };
    let mut response = (status, Json(body)).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/problem+json"),
    );
    response
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match &self {
            ApiError::Sqlx(e) => tracing::error!(error = %e, "database error"),
            ApiError::Domain(DomainError::Internal) => tracing::error!("internal error"),
            _ => {}
        }
        problem_response(&self)
    }
}

// ---- OpenAPI response sets (all `application/problem+json`) ----

fn problem_doc(description: &str) -> utoipa::openapi::RefOr<utoipa::openapi::response::Response> {
    use utoipa::openapi::{Ref, content::ContentBuilder, response::ResponseBuilder};
    ResponseBuilder::new()
        .description(description)
        .content(
            "application/problem+json",
            ContentBuilder::new()
                .schema(Some(Ref::from_schema_name("ProblemDetails")))
                .build(),
        )
        .build()
        .into()
}

/// Declares a unit struct usable in `#[utoipa::path(responses(...))]` that
/// documents one problem+json status referencing the `ProblemDetails` schema.
macro_rules! problem_responses {
    ($($(#[$meta:meta])* $name:ident = $status:literal, $description:literal;)*) => {
        $(
            $(#[$meta])*
            pub struct $name;

            impl IntoResponses for $name {
                fn responses() -> std::collections::BTreeMap<
                    String,
                    utoipa::openapi::RefOr<utoipa::openapi::response::Response>,
                > {
                    std::collections::BTreeMap::from([($status.to_string(), problem_doc($description))])
                }
            }
        )*
    };
}

problem_responses! {
    /// 400: malformed JSON, invalid query string.
    BadRequest = 400, "Malformed request (`invalid_json`, `invalid_query`).";
    /// 401: no valid session, or bad login credentials.
    Unauthorized = 401, "No valid session (`unauthorized`) or wrong credentials (`invalid_credentials`).";
    /// 403: authorization, CSRF or origin failure.
    Forbidden = 403, "Not allowed (`forbidden`): missing/invalid `X-CSRF-Token`, untrusted `Origin`, or insufficient role.";
    /// 404: unknown id, or a resource the caller may not see.
    NotFound = 404, "Not found (`not_found`). Also returned for resources hidden from the caller (admin-only boards).";
    /// 409: state conflicts.
    Conflict = 409, "Conflict with current state (`duplicate_post`, `last_admin`).";
    /// 413: body larger than the route's limit.
    PayloadTooLarge = 413, "Request body too large (`payload_too_large`).";
    /// 415: body is not JSON.
    UnsupportedMediaType = 415, "Content-Type must be application/json (`unsupported_media_type`).";
    /// 422: validation failures with field errors.
    Unprocessable = 422, "Validation failed (`validation_failed`) with per-field `errors`.";
    /// 429: flood control.
    TooManyRequests = 429, "Flood control (`posting_too_fast`): wait before posting again.";
}
