use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::get,
};
use hypercs_domain::constants::TERMS_BODY_LIMIT_BYTES;

use crate::{
    auth::AuthUser,
    dto::{HomepageContent, SiteSettings, TermsContent},
    error::{
        ApiResult, BadRequest, Forbidden, PayloadTooLarge, Unauthorized, Unprocessable,
        UnsupportedMediaType,
    },
    extract::ValidatedJson,
    services::cms_service,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/cms/homepage", get(get_homepage).put(put_homepage))
        .route("/cms/site", get(get_site).put(put_site))
        .route(
            "/cms/terms",
            // 20,000 characters can be up to ~80 KB of UTF-8: raise the
            // default 64 KB limit for this route only.
            get(get_terms)
                .put(put_terms)
                .layer(DefaultBodyLimit::max(TERMS_BODY_LIMIT_BYTES)),
        )
}

/// Homepage hero and feature blocks.
#[utoipa::path(get, path = "/api/v1/cms/homepage", operation_id = "get_homepage", tag = "cms",
    responses((status = 200, body = HomepageContent)))]
async fn get_homepage(State(state): State<AppState>) -> ApiResult<Json<HomepageContent>> {
    Ok(Json(cms_service::get_homepage(&state.db).await?))
}

/// Admin: replace the homepage blocks.
#[utoipa::path(put, path = "/api/v1/cms/homepage", operation_id = "put_homepage", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    request_body = HomepageContent,
    responses((status = 204, description = "Saved."),
        BadRequest, Unauthorized, Forbidden, PayloadTooLarge, UnsupportedMediaType, Unprocessable))]
async fn put_homepage(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<HomepageContent>,
) -> ApiResult<StatusCode> {
    cms_service::put_homepage(&state.db, &user.as_domain(), &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Site title, slogan and default language.
#[utoipa::path(get, path = "/api/v1/cms/site", operation_id = "get_site_settings", tag = "cms",
    responses((status = 200, body = SiteSettings)))]
async fn get_site(State(state): State<AppState>) -> ApiResult<Json<SiteSettings>> {
    Ok(Json(cms_service::get_site(&state.db).await?))
}

/// Admin: replace the site settings.
#[utoipa::path(put, path = "/api/v1/cms/site", operation_id = "put_site_settings", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    request_body = SiteSettings,
    responses((status = 204, description = "Saved."),
        BadRequest, Unauthorized, Forbidden, PayloadTooLarge, UnsupportedMediaType, Unprocessable))]
async fn put_site(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<SiteSettings>,
) -> ApiResult<StatusCode> {
    cms_service::put_site(&state.db, &user.as_domain(), &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Terms of service text.
#[utoipa::path(get, path = "/api/v1/cms/terms", operation_id = "get_terms", tag = "cms",
    responses((status = 200, body = TermsContent)))]
async fn get_terms(State(state): State<AppState>) -> ApiResult<Json<TermsContent>> {
    Ok(Json(cms_service::get_terms(&state.db).await?))
}

/// Admin: replace the terms of service text (body limit 128 KB).
#[utoipa::path(put, path = "/api/v1/cms/terms", operation_id = "put_terms", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    request_body = TermsContent,
    responses((status = 204, description = "Saved."),
        BadRequest, Unauthorized, Forbidden, PayloadTooLarge, UnsupportedMediaType, Unprocessable))]
async fn put_terms(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<TermsContent>,
) -> ApiResult<StatusCode> {
    cms_service::put_terms(&state.db, &user.as_domain(), &req).await?;
    Ok(StatusCode::NO_CONTENT)
}
