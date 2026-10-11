//! Flood control and the duplicate-post guard. Account-based (never
//! IP-based) and database-backed: the author's latest post is the reference,
//! so it works across restarts and multiple API instances. Callers hold the
//! per-user posting advisory lock, which serializes concurrent posts by the
//! same user and closes the check-then-insert race.

use chrono::{DateTime, Utc};
use hypercs_domain::DomainError;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{error::ApiResult, state::FloodConfig};

pub async fn check_posting_allowed(
    conn: &mut PgConnection,
    config: &FloodConfig,
    author_id: Uuid,
    body: &str,
    now: DateTime<Utc>,
) -> ApiResult<()> {
    let Some((last_at, last_body)) =
        hypercs_db::posts::latest_by_author(&mut *conn, author_id).await?
    else {
        return Ok(());
    };
    let elapsed = (now - last_at).num_milliseconds();

    // An identical consecutive post (typically a double submit) is reported
    // as a duplicate rather than as flooding.
    if config.duplicate_window_secs > 0
        && elapsed < config.duplicate_window_secs * 1000
        && last_body == body
    {
        return Err(DomainError::Conflict {
            code: "duplicate_post",
        }
        .into());
    }

    let interval = if config.new_account_age_mins > 0 && config.new_account_interval_secs > 0 {
        let created = hypercs_db::users::created_at_by_id(&mut *conn, author_id).await?;
        let is_new = created
            .is_some_and(|c| now - c < chrono::Duration::minutes(config.new_account_age_mins));
        if is_new {
            config.new_account_interval_secs
        } else {
            config.min_interval_secs
        }
    } else {
        config.min_interval_secs
    };
    if interval > 0 && elapsed < interval * 1000 {
        return Err(DomainError::TooManyRequests {
            code: "posting_too_fast",
        }
        .into());
    }
    Ok(())
}
