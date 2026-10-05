use hypercs_db::PgPool;
use hypercs_domain::validation::normalize_text;
use hypercs_domain::{DomainError, DomainResult, Role};
use uuid::Uuid;
use validator::ValidationError;

use crate::dto::{
    ChangePasswordRequest, UpdateAccountRequest, UpdatePreferencesRequest, UpdateProfileRequest,
};

pub fn validate_language(value: &str) -> Result<(), ValidationError> {
    if value == "en-US" || value == "de" {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_language"))
    }
}

fn validate_website(website: &Option<String>) -> DomainResult<()> {
    if let Some(url) = website
        && !url.is_empty()
        && !(url.starts_with("http://") || url.starts_with("https://"))
    {
        return Err(DomainError::validation(
            "website",
            "invalid_url",
            "Website must start with http:// or https://.",
        ));
    }
    Ok(())
}

pub async fn get_me(db: &PgPool, user_id: Uuid) -> DomainResult<hypercs_db::users::MeRow> {
    hypercs_db::users::find_me(db, user_id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)
}

pub async fn update_profile(
    db: &PgPool,
    user_id: Uuid,
    req: &UpdateProfileRequest,
) -> DomainResult<()> {
    validate_website(&req.website)?;
    let display_name = normalize_text(&req.display_name, false);
    if hypercs_domain::validation::contains_zero_width(&display_name) {
        return Err(DomainError::validation(
            "displayName",
            "invalid_display_name",
            "Display name contains invalid characters.",
        ));
    }
    let location = req.location.as_deref().map(|s| normalize_text(s, false));
    let about_me = req.about_me.as_deref().map(|s| normalize_text(s, true));
    hypercs_db::users::update_profile(
        db,
        user_id,
        &display_name,
        location.as_deref(),
        req.website.as_deref(),
        about_me.as_deref(),
    )
    .await
    .map_err(|_| DomainError::Internal)
}

pub async fn update_account(
    db: &PgPool,
    user_id: Uuid,
    req: &UpdateAccountRequest,
) -> DomainResult<()> {
    let email = normalize_text(&req.email, false).to_lowercase();
    if hypercs_db::users::email_taken(db, &email)
        .await
        .map_err(|_| DomainError::Internal)?
    {
        // Allow keeping one's own email unchanged.
        let current = hypercs_db::users::find_me(db, user_id)
            .await
            .map_err(|_| DomainError::Internal)?
            .ok_or(DomainError::NotFound)?;
        if current.email.to_lowercase() != email {
            return Err(DomainError::validation(
                "email",
                "email_taken",
                "This email address is already registered.",
            ));
        }
    }
    let first_name = req.first_name.as_deref().map(|s| normalize_text(s, false));
    let last_name = req.last_name.as_deref().map(|s| normalize_text(s, false));
    hypercs_db::users::update_account(
        db,
        user_id,
        first_name.as_deref(),
        last_name.as_deref(),
        &email,
    )
    .await
    .map_err(|_| DomainError::Internal)
}

pub async fn change_password(
    db: &PgPool,
    user_id: Uuid,
    current_session_id: Uuid,
    req: &ChangePasswordRequest,
) -> DomainResult<()> {
    let current_hash = hypercs_db::users::password_hash_by_id(db, user_id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)?;
    if !crate::auth::crypto::verify_password(&req.current_password, &current_hash) {
        return Err(DomainError::validation(
            "currentPassword",
            "invalid_password",
            "Current password is incorrect.",
        ));
    }
    let new_hash =
        crate::auth::crypto::hash_password(&req.new_password).ok_or(DomainError::Internal)?;
    hypercs_db::users::update_password_hash(db, user_id, &new_hash)
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::sessions::delete_all_for_user_except(db, user_id, Some(current_session_id))
        .await
        .map_err(|_| DomainError::Internal)
}

pub async fn update_preferences(
    db: &PgPool,
    user_id: Uuid,
    req: &UpdatePreferencesRequest,
) -> DomainResult<()> {
    hypercs_db::users::update_preferences(
        db,
        user_id,
        &req.language,
        &req.timezone,
        req.notify_on_reply,
        req.notify_on_quote,
        req.show_online,
    )
    .await
    .map_err(|_| DomainError::Internal)
}

pub async fn list_sessions(
    db: &PgPool,
    user_id: Uuid,
) -> DomainResult<Vec<hypercs_db::sessions::SessionListRow>> {
    hypercs_db::sessions::list_for_user(db, user_id)
        .await
        .map_err(|_| DomainError::Internal)
}

pub async fn revoke_session(db: &PgPool, user_id: Uuid, session_id: Uuid) -> DomainResult<()> {
    // Scope the delete to the caller's own sessions by checking membership first.
    let sessions = list_sessions(db, user_id).await?;
    if !sessions.iter().any(|s| s.id == session_id) {
        return Err(DomainError::NotFound);
    }
    hypercs_db::sessions::delete_by_id(db, session_id)
        .await
        .map_err(|_| DomainError::Internal)
}

pub async fn public_profile(
    db: &PgPool,
    username: &str,
) -> DomainResult<hypercs_db::users::PublicProfileRow> {
    hypercs_db::users::find_public_profile(db, username)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)
}

pub async fn list_all_users(db: &PgPool) -> DomainResult<Vec<hypercs_db::users::AdminUserRow>> {
    hypercs_db::users::list_all(db)
        .await
        .map_err(|_| DomainError::Internal)
}

pub async fn update_role(db: &PgPool, user_id: Uuid, role_str: &str) -> DomainResult<()> {
    let role = Role::from_db_str(role_str).ok_or_else(|| {
        DomainError::validation(
            "role",
            "invalid_role",
            "Role must be admin, moderator or member.",
        )
    })?;
    hypercs_db::users::update_role(db, user_id, role)
        .await
        .map_err(|_| DomainError::Internal)
}
