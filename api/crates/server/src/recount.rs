use hypercs_db::PgPool;

/// Recomputes all denormalized counters from source tables. Safe to run at
/// any time as a repair tool; also exercised by tests that assert counter
/// consistency.
pub async fn run(db: &PgPool) -> anyhow::Result<()> {
    let mut conn = db.acquire().await?;

    for user_id in hypercs_db::users::all_ids(&mut *conn).await? {
        hypercs_db::users::recompute_post_count(&mut *conn, user_id).await?;
    }
    for topic_id in hypercs_db::topics::all_ids(&mut *conn).await? {
        hypercs_db::topics::recompute_topic_aggregates(&mut conn, topic_id).await?;
    }
    for board_id in hypercs_db::structure::all_board_ids(&mut *conn).await? {
        hypercs_db::structure::recompute_board_aggregates(&mut conn, board_id).await?;
    }

    tracing::info!("recount complete");
    Ok(())
}
