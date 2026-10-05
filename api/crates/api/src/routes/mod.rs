use std::time::Duration;

use axum::{Router, http::HeaderValue};
use tower_http::{
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi as _;
use utoipa_swagger_ui::SwaggerUi;

pub mod auth;
pub mod cms;
pub mod health;
pub mod me;
pub mod news;
pub mod stats;
pub mod structure;
pub mod topics;
pub mod users;

use crate::{openapi::ApiDoc, state::AppState};

const REQUEST_ID_HEADER: &str = "x-request-id";

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

    let mut app = Router::new()
        .nest("/api/v1", api_routes)
        .layer(RequestBodyLimitLayer::new(
            hypercs_domain::constants::DEFAULT_BODY_LIMIT_BYTES,
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::REFERRER_POLICY,
            HeaderValue::from_static("same-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'"),
        ))
        .layer(PropagateRequestIdLayer::new(
            axum::http::HeaderName::from_static(REQUEST_ID_HEADER),
        ))
        .layer(TraceLayer::new_for_http().on_failure(
            |error: tower_http::classify::ServerErrorsFailureClass,
             latency: Duration,
             _span: &tracing::Span| {
                tracing::error!(%error, ?latency, "request failed");
            },
        ))
        .layer(SetRequestIdLayer::new(
            axum::http::HeaderName::from_static(REQUEST_ID_HEADER),
            MakeRequestUuid,
        ));

    if !state.config.allowed_origins.is_empty() {
        let origins: Vec<HeaderValue> = state
            .config
            .allowed_origins
            .iter()
            .filter_map(|o| HeaderValue::from_str(o).ok())
            .collect();
        app = app.layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_credentials(true)
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::HeaderName::from_static("x-csrf-token"),
                ])
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PATCH,
                    axum::http::Method::PUT,
                    axum::http::Method::DELETE,
                ]),
        );
    }

    if state.config.docs_enabled {
        app = app
            .merge(SwaggerUi::new("/api/v1/docs").url("/api/v1/openapi.json", ApiDoc::openapi()));
    } else {
        app = app.route(
            "/api/v1/openapi.json",
            axum::routing::get(|| async { axum::Json(ApiDoc::openapi()) }),
        );
    }

    app.with_state(state)
}
