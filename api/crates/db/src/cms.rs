use serde_json::Value;
use sqlx::PgExecutor;

pub async fn get<'e>(ex: impl PgExecutor<'e>, key: &str) -> sqlx::Result<Option<Value>> {
    let row: Option<(Value,)> = sqlx::query_as("select data from cms_content where key = $1")
        .bind(key)
        .fetch_optional(ex)
        .await?;
    Ok(row.map(|r| r.0))
}

pub async fn put<'e>(ex: impl PgExecutor<'e>, key: &str, data: &Value) -> sqlx::Result<()> {
    sqlx::query(
        "insert into cms_content (key, data, updated_at) values ($1, $2, now()) \
         on conflict (key) do update set data = excluded.data, updated_at = now()",
    )
    .bind(key)
    .bind(data)
    .execute(ex)
    .await?;
    Ok(())
}
