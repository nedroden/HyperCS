use hypercs_db::PgPool;
use hypercs_domain::{
    AuthUser as Principal, DomainError, Role,
    constants::{
        ABOUT_ME_MAX, DISPLAY_NAME_MAX, DISPLAY_NAME_MIN, EMAIL_MAX, LOCATION_MAX, PERSON_NAME_MAX,
        USERS_PER_PAGE, WEBSITE_MAX,
    },
    pagination::PageParams,
    policy,
    validation::{is_http_url, is_valid_timezone},
};
use uuid::Uuid;

use crate::{
    auth::{AuthUser, crypto},
    dto::{
        AdminUserPage, ChangePasswordRequest, MeResponse, PublicProfile, SessionInfo,
        UpdateAccountRequest, UpdatePreferencesRequest, UpdateProfileRequest,
    },
    error::{ApiError, ApiResult},
    services::{text, view},
};

pub async fn get_me(db: &PgPool, user: &AuthUser) -> ApiResult<MeResponse> {
    let row = hypercs_db::users::find_me(db, user.id)
        .await?
        .ok_or_else(ApiError::unauthenticated)?;
    Ok(view::me(row, user.csrf_token.clone()))
}

pub async fn update_profile(
    db: &PgPool,
    user_id: Uuid,
    req: &UpdateProfileRequest,
) -> ApiResult<()> {
    let mut tx = db.begin().await?;
    let me = hypercs_db::users::find_me_for_update(&mut tx, user_id)
        .await?
        .ok_or_else(ApiError::unauthenticated)?;

    let display_name = match &req.display_name {
        Some(name) => text::line("displayName", name, DISPLAY_NAME_MIN, DISPLAY_NAME_MAX)?,
        None => me.display_name,
    };
    let location = text::merge_nullable(me.location, &req.location, |v| {
        text::line("location", v, 0, LOCATION_MAX)
    })?;
    let website = text::merge_nullable(me.website, &req.website, |v| {
        let url = text::line("website", v, 0, WEBSITE_MAX)?;
        if !url.is_empty() && !is_http_url(&url) {
            return Err(ApiError::validation(
                "website",
                "url",
                "Website must be an http:// or https:// URL.",
            ));
        }
        Ok(url)
    })?;
    let about_me = text::merge_nullable(me.about_me, &req.about_me, |v| {
        text::text_trimmed("aboutMe", v, ABOUT_ME_MAX)
    })?;

    hypercs_db::users::update_profile(
        &mut *tx,
        user_id,
        &display_name,
        location.as_deref(),
        website.as_deref(),
        about_me.as_deref(),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn update_account(
    db: &PgPool,
    user_id: Uuid,
    req: &UpdateAccountRequest,
) -> ApiResult<()> {
    let mut tx = db.begin().await?;
    let me = hypercs_db::users::find_me_for_update(&mut tx, user_id)
        .await?
        .ok_or_else(ApiError::unauthenticated)?;

    let email = match &req.email {
        Some(email) => {
            let email = text::line("email", email, 3, EMAIL_MAX)?.to_lowercase();
            if hypercs_db::users::email_taken_by_other(&mut *tx, &email, Some(user_id)).await? {
                return Err(ApiError::validation(
                    "email",
                    "email_taken",
                    "This email address is already registered.",
                ));
            }
            email
        }
        None => me.email,
    };
    let first_name = text::merge_nullable(me.first_name, &req.first_name, |v| {
        text::line("firstName", v, 0, PERSON_NAME_MAX)
    })?;
    let last_name = text::merge_nullable(me.last_name, &req.last_name, |v| {
        text::line("lastName", v, 0, PERSON_NAME_MAX)
    })?;

    hypercs_db::users::update_account(
        &mut *tx,
        user_id,
        first_name.as_deref(),
        last_name.as_deref(),
        &email,
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Verifies the current password, stores the new hash and revokes every
/// other session of the account (the current session stays signed in).
pub async fn change_password(
    db: &PgPool,
    user: &AuthUser,
    req: &ChangePasswordRequest,
) -> ApiResult<()> {
    let mut tx = db.begin().await?;
    let current_hash = hypercs_db::users::password_hash_for_update(&mut tx, user.id)
        .await?
        .ok_or_else(ApiError::unauthenticated)?;
    if !crypto::verify_password_async(req.current_password.clone(), current_hash).await {
        return Err(ApiError::validation(
            "currentPassword",
            "invalid_password",
            "The current password is incorrect.",
        ));
    }
    let new_hash = crypto::hash_password_async(req.new_password.clone())
        .await
        .ok_or_else(ApiError::internal)?;
    hypercs_db::users::update_password_hash(&mut *tx, user.id, &new_hash).await?;
    hypercs_db::sessions::delete_all_for_user_except(&mut *tx, user.id, Some(user.session_id))
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn update_preferences(
    db: &PgPool,
    user_id: Uuid,
    req: &UpdatePreferencesRequest,
) -> ApiResult<()> {
    if let Some(language) = &req.language
        && language != "en-US"
        && language != "de"
    {
        return Err(ApiError::validation(
            "language",
            "invalid_language",
            "Language must be 'en-US' or 'de'.",
        ));
    }
    if let Some(timezone) = &req.timezone
        && !is_valid_timezone(timezone)
    {
        return Err(ApiError::validation(
            "timezone",
            "invalid_timezone",
            "Unknown IANA time zone.",
        ));
    }

    let mut tx = db.begin().await?;
    let me = hypercs_db::users::find_me_for_update(&mut tx, user_id)
        .await?
        .ok_or_else(ApiError::unauthenticated)?;
    hypercs_db::users::update_preferences(
        &mut *tx,
        user_id,
        req.language.as_deref().unwrap_or(&me.language),
        req.timezone.as_deref().unwrap_or(&me.timezone),
        req.notify_on_reply.unwrap_or(me.notify_on_reply),
        req.notify_on_quote.unwrap_or(me.notify_on_quote),
        req.show_online.unwrap_or(me.show_online),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn list_sessions(db: &PgPool, user: &AuthUser) -> ApiResult<Vec<SessionInfo>> {
    let rows = hypercs_db::sessions::list_for_user(db, user.id).await?;
    Ok(rows
        .into_iter()
        .map(|row| SessionInfo {
            current: row.id == user.session_id,
            id: row.id,
            user_agent: row.user_agent,
            remember: row.remember,
            created_at: row.created_at,
            last_seen_at: row.last_seen_at,
            expires_at: row.expires_at,
        })
        .collect())
}

/// Revokes one of the caller's own sessions (another user's session id is
/// indistinguishable from an unknown one).
pub async fn revoke_session(db: &PgPool, user_id: Uuid, session_id: Uuid) -> ApiResult<()> {
    if hypercs_db::sessions::delete_owned(db, session_id, user_id).await? {
        Ok(())
    } else {
        Err(ApiError::not_found())
    }
}

pub async fn public_profile(db: &PgPool, username: &str) -> ApiResult<PublicProfile> {
    hypercs_db::users::find_public_profile(db, username)
        .await?
        .map(view::public_profile)
        .ok_or_else(ApiError::not_found)
}

pub async fn list_users(
    db: &PgPool,
    actor: &Principal,
    page: Option<u32>,
    per_page: Option<u32>,
) -> ApiResult<AdminUserPage> {
    if !policy::can_manage_users(actor) {
        return Err(ApiError::forbidden());
    }
    let params = PageParams::new(page, per_page, USERS_PER_PAGE);
    let rows = hypercs_db::users::list_page(db, params.limit(), params.offset()).await?;
    let total = hypercs_db::users::count_all(db).await?;
    Ok(AdminUserPage::new(
        rows.into_iter().map(view::admin_user).collect(),
        params,
        total,
    ))
}

/// Admin-only. Roles are read from the database on every request, so the
/// change applies immediately (including to the target's open sessions). The
/// last admin cannot be demoted.
pub async fn update_role(
    db: &PgPool,
    actor: &Principal,
    username: &str,
    new_role: Role,
) -> ApiResult<()> {
    if !policy::can_manage_users(actor) {
        return Err(ApiError::forbidden());
    }
    let mut tx = db.begin().await?;
    let admins = hypercs_db::users::lock_admins(&mut tx).await?;
    let (target_id, current) = hypercs_db::users::lock_by_username(&mut tx, username)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let current = Role::from_db_or_member(&current);
    if policy::would_remove_last_admin(current, new_role, admins.len() as i64) {
        return Err(DomainError::Conflict { code: "last_admin" }.into());
    }
    hypercs_db::users::update_role(&mut *tx, target_id, new_role).await?;
    tx.commit().await?;
    Ok(())
}
