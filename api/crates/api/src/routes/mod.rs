use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    http::{StatusCode, header},
    middleware::{from_fn, from_fn_with_state},
    response::{IntoResponse, Response},
};
use hypercs_domain::constants::DEFAULT_BODY_LIMIT_BYTES;
use tower_http::{catch_panic::CatchPanicLayer, cors::CorsLayer};
use utoipa::OpenApi as _;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

pub mod auth;
pub mod cms;
pub mod health;
pub mod me;
pub mod news;
pub mod stats;
pub mod structure;
pub mod topics;
pub mod users;

use crate::{
    dto::CreatedResponse,
    error::{ApiError, problem_response},
    middleware::{request_context, security_headers},
    openapi::ApiDoc,
    state::AppState,
};

/// `201 Created` with the `Location` header and `{ "id": ... }` body.
pub(crate) fn created(location: String, id: Uuid) -> Response {
    (
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreatedResponse { id }),
    )
        .into_response()
}

async fn not_found_fallback() -> ApiError {
    ApiError::not_found()
}

async fn method_not_allowed_fallback() -> ApiError {
    hypercs_domain::DomainError::MethodNotAllowed.into()
}

fn panic_response(_: Box<dyn std::any::Any + Send + 'static>) -> Response {
    tracing::error!("handler panicked");
    problem_response(&ApiError::internal())
}

fn cors_layer(origins: &[String]) -> CorsLayer {
    let origins: Vec<_> = origins
        .iter()
        .filter_map(|o| axum::http::HeaderValue::from_str(o).ok())
        .collect();
    CorsLayer::new()
        .allow_origin(origins)
        .allow_credentials(true)
        .allow_headers([
            header::CONTENT_TYPE,
            axum::http::HeaderName::from_static("x-csrf-token"),
        ])
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .expose_headers([header::LOCATION])
        .max_age(std::time::Duration::from_secs(600))
}

pub fn router(state: AppState) -> Router {
    let api_routes = Router::new()
        .merge(health::routes())
        .merge(auth::routes())
        .merge(me::routes())
        .merge(users::routes())
        .merge(structure::routes())
        .merge(topics::routes())
        .merge(news::routes())
        .merge(cms::routes())
        .merge(stats::routes());

    let mut app = Router::new().nest("/api/v1", api_routes);

    // `/api/v1/openapi.json` is always available. Swagger UI (which also
    // serves that JSON) is added only when explicitly enabled; it serves its
    // own assets and gets a relaxed CSP from `security_headers`.
    app = if state.config.docs_enabled {
        app.merge(SwaggerUi::new("/api/v1/docs").url("/api/v1/openapi.json", ApiDoc::openapi()))
    } else {
        app.route(
            "/api/v1/openapi.json",
            axum::routing::get(|| async { Json(ApiDoc::openapi()) }),
        )
    };

    // Layers added later wrap the earlier ones, so the last is outermost:
    // CORS -> panic guard -> request id/span -> security headers -> routes.
    let app = app
        .fallback(not_found_fallback)
        .method_not_allowed_fallback(method_not_allowed_fallback)
        // Global default; individual routes override it with their own
        // `DefaultBodyLimit` layer. Violations surface as 413 problem+json
        // through the `ValidatedJson` rejection.
        .layer(DefaultBodyLimit::max(DEFAULT_BODY_LIMIT_BYTES))
        .layer(from_fn_with_state(state.clone(), security_headers))
        .layer(from_fn(request_context))
        .layer(CatchPanicLayer::custom(panic_response));

    // Explicit allow-list, empty by default: without entries no CORS headers
    // are emitted and browsers keep cross-origin requests blocked.
    let app = if state.config.allowed_origins.is_empty() {
        app
    } else {
        app.layer(cors_layer(&state.config.allowed_origins))
    };

    app.with_state(state)
}
