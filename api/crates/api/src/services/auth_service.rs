use chrono::Utc;
use hypercs_db::PgPool;
use hypercs_domain::validation::{
    contains_zero_width, is_reserved_username, is_valid_username_chars, normalize_text,
};
use hypercs_domain::{DomainError, DomainResult, Role};
use uuid::Uuid;
use validator::ValidationError;

use crate::dto::{LoginRequest, RegisterRequest};

pub fn validate_accept_terms(value: &bool) -> Result<(), ValidationError> {
    if *value {
        Ok(())
    } else {
        Err(ValidationError::new("terms_not_accepted")
            .with_message("You must accept the terms of service.".into()))
    }
}

pub struct RegisteredUser {
    pub id: Uuid,
}

pub async fn register(db: &PgPool, req: &RegisterRequest) -> DomainResult<RegisteredUser> {
    let username = normalize_text(&req.username, false);
    if !is_valid_username_chars(&username) {
        return Err(DomainError::validation(
            "username",
            "invalid_username",
            "Usernames may only contain letters, numbers, - and _.",
        ));
    }
    if is_reserved_username(&username) {
        return Err(DomainError::validation(
            "username",
            "username_taken",
            "This username is not available.",
        ));
    }
    let email = normalize_text(&req.email, false).to_lowercase();

    if hypercs_db::users::username_taken(db, &username)
        .await
        .map_err(|_| DomainError::Internal)?
    {
        return Err(DomainError::validation(
            "username",
            "username_taken",
            "This username is already taken.",
        ));
    }
    if hypercs_db::users::email_taken(db, &email)
        .await
        .map_err(|_| DomainError::Internal)?
    {
        return Err(DomainError::validation(
            "email",
            "email_taken",
            "This email address is already registered.",
        ));
    }

    let password_hash =
        crate::auth::crypto::hash_password(&req.password).ok_or(DomainError::Internal)?;
    let id = Uuid::now_v7();
    hypercs_db::users::create_user(
        db,
        id,
        &username,
        &username,
        &email,
        &password_hash,
        Role::Member,
    )
    .await
    .map_err(|_| DomainError::Internal)?;
    Ok(RegisteredUser { id })
}

pub struct LoggedInSession {
    pub user_id: Uuid,
    pub role: Role,
    pub token: String,
    pub csrf_token: String,
    pub remember: bool,
}

/// Credential errors are intentionally generic (never reveal whether the
/// username exists). Always creates a brand-new session (rotation).
pub async fn login(
    db: &PgPool,
    req: &LoginRequest,
    user_agent: Option<&str>,
    ip: Option<&str>,
) -> DomainResult<LoggedInSession> {
    if contains_zero_width(&req.username) {
        return Err(DomainError::validation(
            "username",
            "invalid_credentials",
            "Invalid username or password.",
        ));
    }
    let user = hypercs_db::users::find_auth_by_username(db, &req.username)
        .await
        .map_err(|_| DomainError::Internal)?;
    let user = match user {
        Some(u) => u,
        None => {
            // Still run a hash verification to keep timing similar whether or
            // not the username exists.
            let _ = crate::auth::crypto::verify_password(
                &req.password,
                "$argon2id$v=19$m=19456,t=2,p=1$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            );
            return Err(DomainError::validation(
                "password",
                "invalid_credentials",
                "Invalid username or password.",
            ));
        }
    };
    if !crate::auth::crypto::verify_password(&req.password, &user.password_hash) {
        return Err(DomainError::validation(
            "password",
            "invalid_credentials",
            "Invalid username or password.",
        ));
    }
    let role = Role::from_db_str(&user.role).ok_or(DomainError::Internal)?;

    let token = crate::auth::crypto::generate_token();
    let token_hash = crate::auth::crypto::hash_token(&token);
    let csrf_token = crate::auth::crypto::generate_token();
    let session_id = Uuid::now_v7();
    let expires_at = crate::auth::cookie::session_expiry(req.remember, Utc::now());
    hypercs_db::sessions::create_session(
        db,
        session_id,
        user.id,
        &token_hash,
        &csrf_token,
        req.remember,
        user_agent,
        ip,
        expires_at,
    )
    .await
    .map_err(|_| DomainError::Internal)?;

    Ok(LoggedInSession {
        user_id: user.id,
        role,
        token,
        csrf_token,
        remember: req.remember,
    })
}

pub async fn logout(db: &PgPool, session_id: Uuid) -> DomainResult<()> {
    hypercs_db::sessions::delete_by_id(db, session_id)
        .await
        .map_err(|_| DomainError::Internal)
}

/// Creates the first admin from env-provided credentials if no admin exists
/// yet. Called once at server startup.
pub async fn bootstrap_admin(
    db: &PgPool,
    username: &str,
    email: &str,
    password: &str,
) -> DomainResult<()> {
    if hypercs_db::users::any_admin_exists(db)
        .await
        .map_err(|_| DomainError::Internal)?
    {
        return Ok(());
    }
    let password_hash =
        crate::auth::crypto::hash_password(password).ok_or(DomainError::Internal)?;
    let id = Uuid::now_v7();
    hypercs_db::users::create_user(
        db,
        id,
        username,
        username,
        email,
        &password_hash,
        Role::Admin,
    )
    .await
    .map_err(|_| DomainError::Internal)?;
    tracing::info!(username, "bootstrapped first admin account");
    Ok(())
}
