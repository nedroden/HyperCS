use chrono::Utc;
use hypercs_db::PgPool;
use hypercs_domain::constants::{
    DUPLICATE_POST_WINDOW_SECS, MIN_POST_INTERVAL_SECS, NEW_ACCOUNT_AGE_MINS,
    NEW_ACCOUNT_POST_INTERVAL_SECS,
};
use hypercs_domain::{DomainError, DomainResult};
use uuid::Uuid;

/// Flood control (minimum time between posts, stricter for brand-new
/// accounts) and a duplicate-post guard (reject an identical consecutive
/// post within a short window). Not rate limiting in the IP sense: purely
/// account-based, per spec.
pub async fn check_posting_allowed(
    db: &PgPool,
    author_id: Uuid,
    account_created_at: chrono::DateTime<Utc>,
    body: &str,
) -> DomainResult<()> {
    let now = Utc::now();
    if let Some(last) = hypercs_db::posts::last_post_time_for_author(db, author_id)
        .await
        .map_err(|_| DomainError::Internal)?
    {
        let account_age_mins = (now - account_created_at).num_minutes();
        let min_interval = if account_age_mins < NEW_ACCOUNT_AGE_MINS {
            NEW_ACCOUNT_POST_INTERVAL_SECS
        } else {
            MIN_POST_INTERVAL_SECS
        };
        if (now - last).num_seconds() < min_interval {
            return Err(DomainError::TooManyRequests {
                code: "posting_too_fast",
            });
        }
        if (now - last).num_seconds() < DUPLICATE_POST_WINDOW_SECS
            && let Some(last_body) = hypercs_db::posts::last_post_body_for_author(db, author_id)
                .await
                .map_err(|_| DomainError::Internal)?
            && last_body == body
        {
            return Err(DomainError::Conflict {
                code: "duplicate_post",
            });
        }
    }
    Ok(())
}
