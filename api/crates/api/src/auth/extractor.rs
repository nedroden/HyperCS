//! Authentication extractors. These authenticate only; authorization
//! decisions are made by the policy module and services, never here.

use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, Method, header, request::Parts},
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use hypercs_domain::{DomainError, Role, constants::LAST_SEEN_THROTTLE_MINUTES};
use uuid::Uuid;

use crate::{
    auth::{
        cookie::idle_expiry,
        crypto::{hash_token, tokens_equal},
        origin::{OriginHeaders, origin_is_allowed},
    },
    error::{ApiError, ApiResult},
    middleware::RequestUser,
    state::AppState,
};

pub const SESSION_COOKIE: &str = "hypercs_session";
pub const CSRF_HEADER: &str = "x-csrf-token";

/// The authenticated principal plus the session it came from.
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub id: Uuid,
    /// Read from the database on every request, so role changes apply
    /// immediately.
    pub role: Role,
    pub session_id: Uuid,
    pub csrf_token: String,
}

impl AuthUser {
    pub fn as_domain(&self) -> hypercs_domain::AuthUser {
        hypercs_domain::AuthUser {
            id: self.id,
            role: self.role,
        }
    }
}

/// Resolves the session cookie to a user. Expired sessions are deleted; a
/// valid session's `last_seen_at` is touched at most once a minute. Database
/// failures are errors (500), not silently treated as "logged out".
async fn resolve_session(parts: &Parts, state: &AppState) -> ApiResult<Option<AuthUser>> {
    let jar = CookieJar::from_headers(&parts.headers);
    let Some(cookie) = jar.get(SESSION_COOKIE) else {
        return Ok(None);
    };
    let token_hash = hash_token(cookie.value());
    let Some(row) = hypercs_db::sessions::find_by_token_hash(&state.db, &token_hash).await? else {
        return Ok(None);
    };
    let now = Utc::now();
    if row.expires_at <= now {
        hypercs_db::sessions::delete_by_id(&state.db, row.id).await?;
        return Ok(None);
    }
    if now - row.last_seen_at >= chrono::Duration::minutes(LAST_SEEN_THROTTLE_MINUTES) {
        hypercs_db::sessions::touch(
            &state.db,
            row.id,
            now,
            chrono::Duration::minutes(LAST_SEEN_THROTTLE_MINUTES),
            idle_expiry(now),
        )
        .await?;
    }
    if let Some(slot) = parts.extensions.get::<RequestUser>() {
        let _ = slot.0.set(row.user_id);
    }
    Ok(Some(AuthUser {
        id: row.user_id,
        role: Role::from_db_or_member(&row.role),
        session_id: row.id,
        csrf_token: row.csrf_token,
    }))
}

fn is_unsafe_method(method: &Method) -> bool {
    matches!(
        *method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    )
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

fn check_same_origin(parts: &Parts, state: &AppState) -> ApiResult<()> {
    let headers = OriginHeaders {
        origin: header_str(&parts.headers, header::ORIGIN.as_str()),
        host: header_str(&parts.headers, header::HOST.as_str()),
        forwarded_host: header_str(&parts.headers, "x-forwarded-host"),
        sec_fetch_site: header_str(&parts.headers, "sec-fetch-site"),
    };
    if origin_is_allowed(headers, &state.config.allowed_origins) {
        Ok(())
    } else {
        Err(DomainError::Forbidden.into())
    }
}

/// Required authentication. For state-changing methods this also enforces a
/// trusted `Origin` and the `X-CSRF-Token` header against the session's token
/// (checked in that order, after the session lookup: guests get 401).
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let unsafe_method = is_unsafe_method(&parts.method);
        if unsafe_method {
            check_same_origin(parts, state)?;
        }
        let user = resolve_session(parts, state)
            .await?
            .ok_or_else(ApiError::unauthenticated)?;
        if unsafe_method {
            let header = header_str(&parts.headers, CSRF_HEADER).unwrap_or("");
            if header.is_empty() || !tokens_equal(header, &user.csrf_token) {
                return Err(DomainError::Forbidden.into());
            }
        }
        Ok(user)
    }
}

/// Optional authentication for guest-accessible reads. Never fails because of
/// a missing/invalid session (that is just a guest).
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(OptionalAuthUser(resolve_session(parts, state).await?))
    }
}

/// Origin check only, for state-changing requests that have no session yet
/// (register, login).
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
