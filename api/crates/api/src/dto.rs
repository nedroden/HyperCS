//! Request/response DTOs. Validation limits mirror `hypercs_domain::constants`
//! so OpenAPI schemas and the generated zod validators stay in sync with the
//! service layer's own checks.

use chrono::{DateTime, Utc};
use hypercs_domain::constants::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// ---- Auth ----

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    #[validate(length(min = "USERNAME_MIN", max = "USERNAME_MAX"))]
    pub username: String,
    #[validate(length(max = "EMAIL_MAX"), email)]
    pub email: String,
    #[validate(length(min = "PASSWORD_MIN", max = "PASSWORD_MAX"))]
    pub password: String,
    #[validate(custom(function = "crate::services::auth_service::validate_accept_terms"))]
    pub accept_terms: bool,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    #[validate(length(min = 1, max = "USERNAME_MAX"))]
    pub username: String,
    #[validate(length(min = 1, max = "PASSWORD_MAX"))]
    pub password: String,
    #[serde(default)]
    pub remember: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub role: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub about_me: Option<String>,
    pub language: String,
    pub timezone: String,
    pub notify_on_reply: bool,
    pub notify_on_quote: bool,
    pub show_online: bool,
    pub post_count: i32,
    pub created_at: DateTime<Utc>,
    pub csrf_token: String,
}

// ---- Settings ----

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    #[validate(length(min = "DISPLAY_NAME_MIN", max = "DISPLAY_NAME_MAX"))]
    pub display_name: String,
    #[validate(length(max = 100))]
    pub location: Option<String>,
    #[validate(length(max = "WEBSITE_MAX"), url)]
    pub website: Option<String>,
    #[validate(length(max = "ABOUT_ME_MAX"))]
    pub about_me: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountRequest {
    #[validate(length(max = 60))]
    pub first_name: Option<String>,
    #[validate(length(max = 60))]
    pub last_name: Option<String>,
    #[validate(length(max = "EMAIL_MAX"), email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(length(min = "PASSWORD_MIN", max = "PASSWORD_MAX"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePreferencesRequest {
    #[validate(custom(function = "crate::services::user_service::validate_language"))]
    pub language: String,
    #[validate(length(min = 1, max = 60))]
    pub timezone: String,
    pub notify_on_reply: bool,
    pub notify_on_quote: bool,
    pub show_online: bool,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleRequest {
    pub role: String,
}

// ---- Forum structure ----

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRequest {
    #[validate(length(min = "CATEGORY_BOARD_NAME_MIN", max = "CATEGORY_BOARD_NAME_MAX"))]
    pub name: String,
    #[validate(length(max = "DESCRIPTION_MAX"))]
    pub description: String,
    #[serde(default)]
    pub admin_only: bool,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardRequest {
    pub category_id: Uuid,
    #[validate(length(min = "CATEGORY_BOARD_NAME_MIN", max = "CATEGORY_BOARD_NAME_MAX"))]
    pub name: String,
    #[validate(length(max = "DESCRIPTION_MAX"))]
    pub description: String,
    #[serde(default)]
    pub admin_only: bool,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReorderDirection {
    pub direction: Direction,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardIndexResponse {
    pub categories: Vec<hypercs_db::structure::CategoryIndexItem>,
}

// ---- Topics & posts ----

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTopicRequest {
    #[validate(length(min = "TOPIC_TITLE_MIN", max = "TOPIC_TITLE_MAX"))]
    pub title: String,
    #[validate(length(min = "POST_BODY_MIN", max = "POST_BODY_MAX"))]
    pub body: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePostRequest {
    #[validate(length(min = "POST_BODY_MIN", max = "POST_BODY_MAX"))]
    pub body: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePostRequest {
    #[validate(length(min = "POST_BODY_MIN", max = "POST_BODY_MAX"))]
    pub body: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModerateTopicRequest {
    pub pinned: Option<bool>,
    pub locked: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MoveTopicRequest {
    pub board_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopicResponse {
    pub topic: hypercs_db::topics::TopicDetailRow,
    pub posts: hypercs_db::pagination::Page<hypercs_db::posts::PostRow>,
}

// ---- News / CMS ----

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewsRequest {
    #[validate(length(min = "NEWS_TITLE_MIN", max = "NEWS_TITLE_MAX"))]
    pub title: String,
    #[validate(length(max = "NEWS_BODY_MAX"))]
    pub body: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HomepageFeature {
    #[validate(length(min = 1, max = 60))]
    pub title: String,
    #[validate(length(min = 1, max = 200))]
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HomepageContent {
    #[validate(length(min = 1, max = 120))]
    pub hero_heading: String,
    #[validate(length(min = 1, max = 300))]
    pub hero_intro: String,
    #[validate(length(min = 1, max = 60))]
    pub hero_button_label: String,
    #[validate(length(min = 3, max = 3), nested)]
    pub features: Vec<HomepageFeature>,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TermsContent {
    #[validate(length(min = 1, max = 20_000))]
    pub body: String,
}

// ---- Pagination query ----

#[derive(Debug, Deserialize, ToSchema)]
pub struct PageQuery {
    pub page: Option<u32>,
    #[serde(rename = "perPage")]
    pub per_page: Option<u32>,
}
