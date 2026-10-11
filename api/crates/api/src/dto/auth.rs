use chrono::{DateTime, Utc};
use hypercs_domain::{Role, constants::*};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::common::{max_chars, patch_nullable};

fn validate_true(value: &bool) -> Result<(), validator::ValidationError> {
    if *value {
        Ok(())
    } else {
        Err(validator::ValidationError::new("terms_not_accepted"))
    }
}

fn v_location(value: &str) -> Result<(), validator::ValidationError> {
    max_chars(value, LOCATION_MAX)
}
fn v_website(value: &str) -> Result<(), validator::ValidationError> {
    max_chars(value, WEBSITE_MAX)
}
fn v_about_me(value: &str) -> Result<(), validator::ValidationError> {
    max_chars(value, ABOUT_ME_MAX)
}
fn v_person_name(value: &str) -> Result<(), validator::ValidationError> {
    max_chars(value, PERSON_NAME_MAX)
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    /// Login name and profile URL slug; letters, digits, `_` and `-`;
    /// case-insensitively unique; cannot be changed later.
    #[validate(length(min = "USERNAME_MIN", max = "USERNAME_MAX"))]
    #[schema(
        min_length = 3,
        max_length = 32,
        pattern = "^[A-Za-z0-9_-]+$",
        example = "alice"
    )]
    pub username: String,
    #[validate(length(max = "EMAIL_MAX"), email)]
    #[schema(max_length = 254, format = "email", example = "alice@example.com")]
    pub email: String,
    #[validate(length(min = "PASSWORD_MIN", max = "PASSWORD_MAX"))]
    #[schema(min_length = 10, max_length = 128, format = "password")]
    pub password: String,
    /// Must be `true` (terms of service accepted).
    #[validate(custom(function = "validate_true"))]
    pub accept_terms: bool,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    #[validate(length(min = 1, max = "USERNAME_MAX"))]
    #[schema(min_length = 1, max_length = 32)]
    pub username: String,
    #[validate(length(min = 1, max = "PASSWORD_MAX"))]
    #[schema(min_length = 1, max_length = 128, format = "password")]
    pub password: String,
    /// "Remember me": 30-day persistent session instead of a browser-session
    /// cookie with 24 h idle expiry.
    #[serde(default)]
    pub remember: bool,
}

/// The logged-in account, including the per-session CSRF token to send as
/// `X-CSRF-Token` on every unsafe request.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub role: Role,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub about_me: Option<String>,
    /// `en-US` or `de`.
    pub language: String,
    /// IANA time zone name.
    pub timezone: String,
    pub notify_on_reply: bool,
    pub notify_on_quote: bool,
    pub show_online: bool,
    pub post_count: i32,
    pub created_at: DateTime<Utc>,
    pub csrf_token: String,
}

/// One signed-in device/browser.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub id: Uuid,
    /// Raw `User-Agent` (truncated) as a device hint; no IP address is stored.
    pub user_agent: Option<String>,
    pub remember: bool,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    /// True for the session making this request ("this device").
    pub current: bool,
}

/// Partial update of the public profile. Absent fields are unchanged;
/// `null` (or an empty string) clears an optional field.
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    #[validate(length(min = "DISPLAY_NAME_MIN", max = "DISPLAY_NAME_MAX"))]
    #[schema(min_length = 1, max_length = 50)]
    pub display_name: Option<String>,
    #[serde(default, deserialize_with = "patch_nullable")]
    #[validate(custom(function = "v_location"))]
    #[schema(value_type = Option<String>, max_length = 100)]
    pub location: Option<Option<String>>,
    /// `http://` or `https://` URL.
    #[serde(default, deserialize_with = "patch_nullable")]
    #[validate(custom(function = "v_website"))]
    #[schema(value_type = Option<String>, max_length = 200, format = "uri")]
    pub website: Option<Option<String>>,
    /// Markdown-free plain text.
    #[serde(default, deserialize_with = "patch_nullable")]
    #[validate(custom(function = "v_about_me"))]
    #[schema(value_type = Option<String>, max_length = 500)]
    pub about_me: Option<Option<String>>,
}

/// Partial update of the private account data.
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountRequest {
    #[serde(default, deserialize_with = "patch_nullable")]
    #[validate(custom(function = "v_person_name"))]
    #[schema(value_type = Option<String>, max_length = 60)]
    pub first_name: Option<Option<String>>,
    #[serde(default, deserialize_with = "patch_nullable")]
    #[validate(custom(function = "v_person_name"))]
    #[schema(value_type = Option<String>, max_length = 60)]
    pub last_name: Option<Option<String>>,
    #[validate(length(max = "EMAIL_MAX"), email)]
    #[schema(max_length = 254, format = "email")]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, max = "PASSWORD_MAX"))]
    #[schema(min_length = 1, max_length = 128, format = "password")]
    pub current_password: String,
    #[validate(length(min = "PASSWORD_MIN", max = "PASSWORD_MAX"))]
    #[schema(min_length = 10, max_length = 128, format = "password")]
    pub new_password: String,
}

/// Partial update of the account preferences.
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePreferencesRequest {
    #[schema(pattern = "^(en-US|de)$", example = "en-US")]
    pub language: Option<String>,
    /// IANA time zone name, e.g. `Europe/Berlin`.
    #[validate(length(min = 1, max = "TIMEZONE_MAX"))]
    #[schema(min_length = 1, max_length = 64, example = "UTC")]
    pub timezone: Option<String>,
    pub notify_on_reply: Option<bool>,
    pub notify_on_quote: Option<bool>,
    /// Show me in the list of users online.
    pub show_online: Option<bool>,
}
