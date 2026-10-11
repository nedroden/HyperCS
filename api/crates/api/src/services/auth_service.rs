use chrono::Utc;
use hypercs_db::PgPool;
use hypercs_domain::{
    DomainError, FieldError, Role,
    constants::{EMAIL_MAX, USER_AGENT_MAX, USERNAME_MAX, USERNAME_MIN},
    validation::{is_reserved_username, is_valid_username_chars},
};
use uuid::Uuid;

use crate::{
    auth::{cookie::session_expiry, crypto},
    dto::{LoginRequest, RegisterRequest, UserSummary},
    error::{ApiError, ApiResult},
    services::{cms_service, text},
};

pub async fn register(db: &PgPool, req: &RegisterRequest) -> ApiResult<UserSummary> {
    let username = text::line("username", &req.username, USERNAME_MIN, USERNAME_MAX)?;
    if !is_valid_username_chars(&username) {
        return Err(ApiError::validation(
            "username",
            "invalid_username",
            "Usernames may only contain letters, digits, '-' and '_'.",
        ));
    }
    let email = text::line("email", &req.email, 3, EMAIL_MAX)?.to_lowercase();

    // Collect both uniqueness problems so the form can show them together.
    // Reserved names are reported as "taken" (they are unavailable).
    let mut errors = Vec::new();
    if is_reserved_username(&username) || hypercs_db::users::username_taken(db, &username).await? {
        errors.push(FieldError {
            field: "username".into(),
            code: "username_taken".into(),
            message: "This username is not available.".into(),
        });
    }
    if hypercs_db::users::email_taken_by_other(db, &email, None).await? {
        errors.push(FieldError {
            field: "email".into(),
            code: "email_taken".into(),
            message: "This email address is already registered.".into(),
        });
    }
    if !errors.is_empty() {
        return Err(DomainError::Validation(errors).into());
    }

    let password_hash = crypto::hash_password_async(req.password.clone())
        .await
        .ok_or_else(ApiError::internal)?;
    let id = Uuid::now_v7();
    // A concurrent registration of the same name loses on the unique index
    // and is mapped to the same field error (see `From<sqlx::Error>`).
    let language = cms_service::default_language(db).await?;
    hypercs_db::users::create_user(
        db,
        id,
        &username,
        &email,
        &password_hash,
        Role::Member,
        &language,
    )
    .await?;
    Ok(UserSummary {
        id,
        display_name: username.clone(),
        username,
        role: Role::Member,
    })
}

pub struct LoggedInSession {
    pub token: String,
    pub remember: bool,
}

fn invalid_credentials() -> ApiError {
    DomainError::Unauthorized {
        code: "invalid_credentials",
    }
    .into()
}

/// A short, log/DB-safe device hint from the `User-Agent` header.
fn device_hint(user_agent: Option<&str>) -> Option<String> {
    let cleaned: String = user_agent?
        .chars()
        .filter(|c| !c.is_control())
        .take(USER_AGENT_MAX)
        .collect();
    let cleaned = cleaned.trim().to_string();
    (!cleaned.is_empty()).then_some(cleaned)
}

/// Verifies credentials and creates a brand-new session (the token is never
/// reused: if the request carried an old session cookie, that session is
/// deleted, i.e. the session is rotated on login). Failures are identical
/// whether or not the username exists, in message and in timing.
pub async fn login(
    db: &PgPool,
    req: &LoginRequest,
    user_agent: Option<&str>,
    previous_token: Option<&str>,
) -> ApiResult<LoggedInSession> {
    let user = hypercs_db::users::find_auth_by_username(db, req.username.trim()).await?;
    let user = match user {
        Some(u) => u,
        None => {
            crypto::verify_dummy_async(req.password.clone()).await;
            return Err(invalid_credentials());
        }
    };
    if !crypto::verify_password_async(req.password.clone(), user.password_hash.clone()).await {
        return Err(invalid_credentials());
    }

    let token = crypto::generate_token();
    let csrf_token = crypto::generate_token();
    let now = Utc::now();

    let mut tx = db.begin().await?;
    if let Some(old) = previous_token {
        hypercs_db::sessions::delete_by_token_hash(&mut *tx, &crypto::hash_token(old)).await?;
    }
    hypercs_db::sessions::create_session(
        &mut *tx,
        Uuid::now_v7(),
        user.id,
        &crypto::hash_token(&token),
        &csrf_token,
        req.remember,
        device_hint(user_agent).as_deref(),
        session_expiry(req.remember, now),
    )
    .await?;
    tx.commit().await?;

    Ok(LoggedInSession {
        token,
        remember: req.remember,
    })
}

pub async fn logout(db: &PgPool, session_id: Uuid) -> ApiResult<()> {
    hypercs_db::sessions::delete_by_id(db, session_id).await?;
    Ok(())
}

/// Creates the first admin from env-provided credentials if no admin exists
/// yet. Idempotent; called at server startup. Returns whether an account was
/// created. Reserved usernames (like the default `admin`) are allowed here.
pub async fn bootstrap_admin(
    db: &PgPool,
    username: &str,
    email: &str,
    password: &str,
) -> ApiResult<bool> {
    if hypercs_db::users::any_admin_exists(db).await? {
        return Ok(false);
    }
    let username = text::line("username", username, USERNAME_MIN, USERNAME_MAX)?;
    if !is_valid_username_chars(&username) {
        return Err(ApiError::validation(
            "username",
            "invalid_username",
            "ADMIN_USERNAME may only contain letters, digits, '-' and '_'.",
        ));
    }
    let email = text::line("email", email, 3, EMAIL_MAX)?.to_lowercase();
    let password_chars = password.chars().count() as u64;
    if !(hypercs_domain::constants::PASSWORD_MIN..=hypercs_domain::constants::PASSWORD_MAX)
        .contains(&password_chars)
    {
        return Err(ApiError::validation(
            "password",
            "length",
            "ADMIN_PASSWORD must be between 10 and 128 characters.",
        ));
    }
    let password_hash = crypto::hash_password_async(password.to_string())
        .await
        .ok_or_else(ApiError::internal)?;
    hypercs_db::users::create_user(
        db,
        Uuid::now_v7(),
        &username,
        &email,
        &password_hash,
        Role::Admin,
        cms_service::DEFAULT_LANGUAGE,
    )
    .await?;
    tracing::info!(username, "bootstrapped first admin account");
    Ok(true)
}
