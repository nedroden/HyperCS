use axum::{Json, Router, extract::State, http::StatusCode, routing::get};

use crate::{
    auth::AuthUser,
    dto::{HomepageContent, TermsContent},
    error::ApiResult,
    extract::ValidatedJson,
    services::cms_service,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/cms/homepage", get(get_homepage).put(put_homepage))
        .route("/cms/terms", get(get_terms).put(put_terms))
}

#[utoipa::path(get, path = "/api/v1/cms/homepage", responses((status = 200, body = HomepageContent)), tag = "cms")]
async fn get_homepage(State(state): State<AppState>) -> ApiResult<Json<HomepageContent>> {
    Ok(Json(cms_service::get_homepage(&state.db).await?))
}

#[utoipa::path(put, path = "/api/v1/cms/homepage", request_body = HomepageContent, responses((status = 204)), tag = "admin")]
async fn put_homepage(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<HomepageContent>,
) -> ApiResult<StatusCode> {
    cms_service::put_homepage(&state.db, &user.as_domain(), &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/api/v1/cms/terms", responses((status = 200, body = TermsContent)), tag = "cms")]
async fn get_terms(State(state): State<AppState>) -> ApiResult<Json<TermsContent>> {
    Ok(Json(cms_service::get_terms(&state.db).await?))
}

#[utoipa::path(put, path = "/api/v1/cms/terms", request_body = TermsContent, responses((status = 204)), tag = "admin")]
async fn put_terms(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<TermsContent>,
) -> ApiResult<StatusCode> {
    cms_service::put_terms(&state.db, &user.as_domain(), &req).await?;
    Ok(StatusCode::NO_CONTENT)
}
