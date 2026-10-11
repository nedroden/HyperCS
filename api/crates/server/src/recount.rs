use hypercs_db::PgPool;

/// Recomputes all denormalized counters (users' post counts, topics' reply
/// counts and last posts, boards' topic/post counts and last posts) from the
/// source tables. Safe to run at any time as a repair tool.
pub async fn run(db: &PgPool) -> anyhow::Result<hypercs_db::maintenance::RecountReport> {
    let report = hypercs_db::maintenance::recount_all(db).await?;
    tracing::info!(
        users = report.users,
        topics = report.topics,
        boards = report.boards,
        "recount complete (rows corrected)"
    );
    Ok(report)
}
