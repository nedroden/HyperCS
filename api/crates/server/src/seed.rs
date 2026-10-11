//! First-start seeding: the admin account from `ADMIN_*` and the sample
//! forum structure from the mocks. Both steps are idempotent and safe to run
//! on every boot.

use chrono::Utc;
use hypercs_api::services::{auth_service, topic_service};
use hypercs_db::PgPool;
use uuid::Uuid;

use crate::config::Config;

/// Creates the first admin from the environment if no admin exists yet.
/// Returns whether an account was created.
pub async fn bootstrap_admin(db: &PgPool, config: &Config) -> anyhow::Result<bool> {
    match (
        &config.admin_username,
        &config.admin_email,
        &config.admin_password,
    ) {
        (Some(username), Some(email), Some(password)) => {
            auth_service::bootstrap_admin(db, username, email, password)
                .await
                .map_err(|e| anyhow::anyhow!("admin bootstrap failed: {e}"))
        }
        _ => {
            tracing::warn!(
                "ADMIN_USERNAME/ADMIN_EMAIL/ADMIN_PASSWORD not fully set; skipping admin bootstrap"
            );
            Ok(false)
        }
    }
}

/// Creates the sample structure (an admin-only "Administration and
/// Moderation" category with "Forum administration"; "My first category"
/// with "General Discussion" and "Second board"; a welcome topic) when there
/// are no categories yet, in one transaction. Content is English user content
/// and is not translated. Returns whether anything was created.
pub async fn seed_sample_structure(db: &PgPool) -> anyhow::Result<bool> {
    let mut tx = db.begin().await?;
    // Serialize concurrent first starts.
    sqlx::query("select pg_advisory_xact_lock(hashtextextended('hypercs-seed', 0))")
        .execute(&mut *tx)
        .await?;
    if hypercs_db::structure::count_categories(&mut *tx).await? > 0 {
        return Ok(false);
    }
    let Some(admin_id) = hypercs_db::users::first_admin_id(&mut *tx).await? else {
        tracing::warn!("no admin account yet; skipping sample structure seed");
        return Ok(false);
    };

    let admin_category = Uuid::now_v7();
    hypercs_db::structure::create_category(
        &mut *tx,
        admin_category,
        "Administration and Moderation",
        "Visible to administrators only.",
        true,
        0,
    )
    .await?;
    hypercs_db::structure::create_board(
        &mut *tx,
        Uuid::now_v7(),
        admin_category,
        "Forum administration",
        "Discuss moderation and site administration.",
        true,
        0,
    )
    .await?;

    let category = Uuid::now_v7();
    hypercs_db::structure::create_category(&mut *tx, category, "My first category", "", false, 1)
        .await?;
    let general = Uuid::now_v7();
    hypercs_db::structure::create_board(
        &mut *tx,
        general,
        category,
        "General Discussion",
        "Talk about anything.",
        false,
        0,
    )
    .await?;
    hypercs_db::structure::create_board(
        &mut *tx,
        Uuid::now_v7(),
        category,
        "Second board",
        "A second place to talk about things.",
        false,
        1,
    )
    .await?;

    topic_service::insert_topic_with_first_post(
        &mut tx,
        general,
        admin_id,
        "Welcome to HyperCS!",
        "Welcome to your brand new forum! This is the first post in your first topic.\n\n\
         Feel free to edit or delete it, create new boards and categories, and invite your community.",
        Utc::now(),
    )
    .await?;

    tx.commit().await?;
    tracing::info!("seeded sample forum structure");
    Ok(true)
}
