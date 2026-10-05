//! Creates the sample forum structure from the mocks on first start. Runs
//! only when no categories exist yet, so it is safe to call on every boot.

use hypercs_db::PgPool;
use uuid::Uuid;

pub async fn seed_sample_structure(db: &PgPool) -> anyhow::Result<()> {
    let existing = hypercs_db::structure::list_categories_ordered(db).await?;
    if !existing.is_empty() {
        return Ok(());
    }
    let Some(admin_id) = hypercs_db::users::first_admin_id(db).await? else {
        tracing::warn!("no admin account yet; skipping sample structure seed");
        return Ok(());
    };

    // Admin-only category, as in the mocks.
    let admin_category_id = Uuid::now_v7();
    hypercs_db::structure::create_category(
        db,
        admin_category_id,
        "Administration and Moderation",
        "Visible to administrators only.",
        true,
        0,
    )
    .await?;
    let admin_board_id = Uuid::now_v7();
    hypercs_db::structure::create_board(
        db,
        admin_board_id,
        admin_category_id,
        "Forum administration",
        "Discuss moderation and site administration.",
        true,
        0,
    )
    .await?;

    // Public category with a welcome topic.
    let category_id = Uuid::now_v7();
    hypercs_db::structure::create_category(db, category_id, "My first category", "", false, 1)
        .await?;
    let general_board_id = Uuid::now_v7();
    hypercs_db::structure::create_board(
        db,
        general_board_id,
        category_id,
        "General Discussion",
        "Talk about anything HyperCS related.",
        false,
        0,
    )
    .await?;
    let second_board_id = Uuid::now_v7();
    hypercs_db::structure::create_board(
        db,
        second_board_id,
        category_id,
        "Second board",
        "A second place to talk about things.",
        false,
        1,
    )
    .await?;

    let topic_id = Uuid::now_v7();
    let post_id = Uuid::now_v7();
    let title = "Welcome to HyperCS!";
    let body = "Welcome to your brand new HyperCS forum! This is the first post in your first topic.\n\n\
                Feel free to edit or delete it, create new boards and categories, and invite your community.";
    hypercs_db::topics::create_topic(
        db,
        topic_id,
        general_board_id,
        title,
        "welcome-to-hypercs",
        admin_id,
    )
    .await?;
    hypercs_db::posts::create_post(db, post_id, topic_id, admin_id, body).await?;
    let now = chrono::Utc::now();
    hypercs_db::topics::set_last_post(db, topic_id, Some(post_id), Some(now)).await?;
    hypercs_db::structure::bump_board_counts(db, general_board_id, 1, 1).await?;
    hypercs_db::structure::set_board_last_post(db, general_board_id, Some(post_id)).await?;
    hypercs_db::users::bump_post_count(db, admin_id, 1).await?;

    tracing::info!("seeded sample forum structure");
    Ok(())
}
