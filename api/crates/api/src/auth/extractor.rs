//! Authentication extractors. These authenticate only; authorization
//! decisions are made by the policy module and services, never here.

use axum::{
    extract::FromRequestParts,
    http::{Method, request::Parts},
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use hypercs_domain::{DomainError, Role};
use uuid::Uuid;

use crate::{
    auth::crypto::hash_token, auth::origin::origin_is_allowed, error::ApiError, state::AppState,
};

pub const SESSION_COOKIE: &str = "hypercs_session";
pub const CSRF_HEADER: &str = "x-csrf-token";

#[derive(Clone, Copy, Debug)]
pub struct AuthUser {
    pub id: Uuid,
    pub role: Role,
    pub session_id: Uuid,
}

impl AuthUser {
    pub fn as_domain(&self) -> hypercs_domain::AuthUser {
        hypercs_domain::AuthUser {
            id: self.id,
            role: self.role,
        }
    }
}

/// Resolves the session cookie to a user, throttling `last_seen_at` updates
/// to at most once per minute and dropping sessions found to be expired.
async fn resolve_session(parts: &Parts, state: &AppState) -> Option<AuthUser> {
    let jar = CookieJar::from_headers(&parts.headers);
    let token = jar.get(SESSION_COOKIE)?.value().to_string();
    let token_hash = hash_token(&token);
    let row = hypercs_db::sessions::find_by_token_hash(&state.db, &token_hash)
        .await
        .ok()??;
    let now = Utc::now();
    if row.expires_at < now {
        let _ = hypercs_db::sessions::delete_by_id(&state.db, row.id).await;
        return None;
    }
    if (now - row.last_seen_at).num_minutes()
        >= hypercs_domain::constants::LAST_SEEN_THROTTLE_MINUTES
    {
        let _ = hypercs_db::sessions::touch_last_seen(&state.db, row.id, now).await;
    }
    let role = Role::from_db_str(&row.role)?;
    Some(AuthUser {
        id: row.user_id,
        role,
        session_id: row.id,
    })
}

/// Returns the session's CSRF token alongside the user, for the CSRF check
/// in the `AuthUser` extractor.
async fn resolve_session_with_csrf(parts: &Parts, state: &AppState) -> Option<(AuthUser, String)> {
    let jar = CookieJar::from_headers(&parts.headers);
    let token = jar.get(SESSION_COOKIE)?.value().to_string();
    let token_hash = hash_token(&token);
    let row = hypercs_db::sessions::find_by_token_hash(&state.db, &token_hash)
        .await
        .ok()??;
    let now = Utc::now();
    if row.expires_at < now {
        let _ = hypercs_db::sessions::delete_by_id(&state.db, row.id).await;
        return None;
    }
    if (now - row.last_seen_at).num_minutes()
        >= hypercs_domain::constants::LAST_SEEN_THROTTLE_MINUTES
    {
        let _ = hypercs_db::sessions::touch_last_seen(&state.db, row.id, now).await;
    }
    let role = Role::from_db_str(&row.role)?;
    Some((
        AuthUser {
            id: row.user_id,
            role,
            session_id: row.id,
        },
        row.csrf_token,
    ))
}

fn is_unsafe_method(method: &Method) -> bool {
    matches!(
        *method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    )
}

fn check_same_origin(parts: &Parts, state: &AppState) -> Result<(), ApiError> {
    let origin = parts
        .headers
        .get(axum::http::header::ORIGIN)
        .and_then(|v| v.to_str().ok());
    let host = parts
        .headers
        .get(axum::http::header::HOST)
        .and_then(|v| v.to_str().ok());
    if origin_is_allowed(origin, host, &state.config.allowed_origins) {
        Ok(())
    } else {
        Err(ApiError::Domain(DomainError::Forbidden))
    }
}

/// Required authentication. For state-changing methods this also enforces
/// same-origin + the `X-CSRF-Token` header against the session's token.
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let unsafe_method = is_unsafe_method(&parts.method);
        if unsafe_method {
            check_same_origin(parts, state)?;
            let (user, csrf_token) = resolve_session_with_csrf(parts, state)
                .await
                .ok_or(ApiError::Domain(DomainError::Unauthorized))?;
            let header = parts
                .headers
                .get(CSRF_HEADER)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            if header.is_empty() || header != csrf_token {
                return Err(ApiError::Domain(DomainError::Forbidden));
            }
            Ok(user)
        } else {
            resolve_session(parts, state)
                .await
                .ok_or(ApiError::Domain(DomainError::Unauthorized))
        }
    }
}

/// Optional authentication for guest-accessible endpoints. Never fails;
/// an invalid/missing session simply resolves to `None` (a guest).
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(OptionalAuthUser(resolve_session(parts, state).await))
    }
}

/// Validates same-origin for state-changing requests that don't require an
/// existing session (register, login).
pub struct SameOrigin;

impl FromRequestParts<AppState> for SameOrigin {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if is_unsafe_method(&parts.method) {
            check_same_origin(parts, state)?;
        }
        Ok(SameOrigin)
    }
}
