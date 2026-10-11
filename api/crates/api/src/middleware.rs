//! HTTP middleware: request ids + one tracing span per request, and security
//! headers.

use std::{
    sync::{Arc, OnceLock},
    time::Instant,
};

use axum::{
    extract::{MatchedPath, Request, State},
    http::{HeaderName, HeaderValue, header},
    middleware::Next,
    response::Response,
};
use tracing::Instrument;
use uuid::Uuid;

use crate::state::AppState;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Written by the auth extractor once the session is resolved so the request
/// span can record the user id after the handler ran.
#[derive(Clone, Default)]
pub struct RequestUser(pub Arc<OnceLock<Uuid>>);

/// A client-supplied request id is propagated only if it is short and made of
/// harmless characters (it ends up in logs and a response header).
fn valid_request_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

/// Generates or propagates `X-Request-Id`, opens one tracing span per request
/// and logs one completion event with method, path, route, status, latency
/// and user id. Never logs bodies, query strings, cookies or headers.
pub async fn request_context(mut req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .filter(|v| valid_request_id(v))
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::now_v7().to_string());
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        req.headers_mut()
            .insert(HeaderName::from_static(REQUEST_ID_HEADER), value);
    }

    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "unmatched".to_string());
    let user = RequestUser::default();
    req.extensions_mut().insert(user.clone());

    let span = tracing::info_span!("http_request", %request_id, %method, %path);
    let started = Instant::now();
    let mut response = next.run(req).instrument(span.clone()).await;
    let latency_ms = started.elapsed().as_millis() as u64;
    let status = response.status().as_u16();
    let user_id = user.0.get().map(Uuid::to_string);

    let _entered = span.enter();
    if status >= 500 {
        tracing::error!(%request_id, %method, %path, %route, status, latency_ms, user_id, "request completed");
    } else {
        tracing::info!(%request_id, %method, %path, %route, status, latency_ms, user_id, "request completed");
    }

    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static(REQUEST_ID_HEADER), value);
    }
    response
}

fn is_docs_path(path: &str) -> bool {
    path == "/api/v1/docs" || path.starts_with("/api/v1/docs/")
}

/// The API serves JSON only, so the default policy forbids everything.
const API_CSP: &str =
    "default-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'";
/// Swagger UI (opt-in via `API_DOCS_ENABLED`) needs its own scripts, styles
/// (it ships an inline `<style>`), images and same-origin requests.
const DOCS_CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
    img-src 'self' data:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; \
    base-uri 'self'; form-action 'none'";

/// Adds security headers to every response (without overriding any a handler
/// set explicitly).
pub async fn security_headers(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let docs = is_docs_path(req.uri().path());
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    let mut set = |name: HeaderName, value: &'static str| {
        headers
            .entry(name)
            .or_insert_with(|| HeaderValue::from_static(value));
    };
    set(header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    set(header::X_FRAME_OPTIONS, "DENY");
    set(header::REFERRER_POLICY, "no-referrer");
    set(
        header::CONTENT_SECURITY_POLICY,
        if docs { DOCS_CSP } else { API_CSP },
    );
    set(
        HeaderName::from_static("cross-origin-resource-policy"),
        "same-origin",
    );
    set(
        HeaderName::from_static("permissions-policy"),
        "camera=(), microphone=(), geolocation=()",
    );
    if !docs {
        // Responses may be per-user (cookie-authenticated): never cache them.
        set(header::CACHE_CONTROL, "no-store");
    }
    if state.config.cookie_secure {
        set(
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=31536000; includeSubDomains",
        );
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_id_validation() {
        assert!(valid_request_id("abc-123_DEF.4"));
        assert!(!valid_request_id(""));
        assert!(!valid_request_id(&"a".repeat(65)));
        assert!(!valid_request_id("bad id"));
        assert!(!valid_request_id("a\r\nb"));
    }

    #[test]
    fn docs_paths() {
        assert!(is_docs_path("/api/v1/docs"));
        assert!(is_docs_path("/api/v1/docs/swagger-ui.css"));
        assert!(!is_docs_path("/api/v1/docsx"));
        assert!(!is_docs_path("/api/v1/openapi.json"));
    }
}
