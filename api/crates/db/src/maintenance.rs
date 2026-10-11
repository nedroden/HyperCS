//! Repair tooling: recomputes every denormalized counter from the source
//! tables. Set-based (one UPDATE per counter) and run in one transaction, so
//! it is safe to run on a live system. Exercised by tests that corrupt the
//! counters on purpose.

use sqlx::{PgConnection, PgPool};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecountReport {
    pub users: u64,
    pub topics: u64,
    pub boards: u64,
}

pub async fn recount_all(db: &PgPool) -> sqlx::Result<RecountReport> {
    let mut tx = db.begin().await?;
    let report = recount_all_in(&mut tx).await?;
    tx.commit().await?;
    Ok(report)
}

/// The same inside a caller-owned transaction/connection (lets tests check
/// "nothing to fix" and roll back).
pub async fn recount_all_in(tx: &mut PgConnection) -> sqlx::Result<RecountReport> {
    // users.post_count
    let users = sqlx::query(
        "update users u set post_count = coalesce(c.n, 0) \
         from users u2 left join (select author_id, count(*)::int as n from posts group by author_id) c \
              on c.author_id = u2.id \
         where u.id = u2.id and u.post_count is distinct from coalesce(c.n, 0)",
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    // topics.reply_count, last_post_id, last_post_at. A topic always keeps
    // at least its first post; if a topic somehow has none, it is left alone.
    let topics = sqlx::query(
        "update topics t set \
            reply_count = greatest(s.n - 1, 0), \
            last_post_id = s.last_id, \
            last_post_at = s.last_at \
         from ( \
            select p.topic_id, count(*)::int as n, \
                   (array_agg(p.id order by p.id desc))[1] as last_id, \
                   (array_agg(p.created_at order by p.id desc))[1] as last_at \
            from posts p group by p.topic_id) s \
         where t.id = s.topic_id \
           and (t.reply_count is distinct from greatest(s.n - 1, 0) \
                or t.last_post_id is distinct from s.last_id \
                or t.last_post_at is distinct from s.last_at)",
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    // boards.topic_count, post_count, last_post_id, last_post_at
    let boards = sqlx::query(
        "update boards b set \
            topic_count = s.topics, post_count = s.posts, \
            last_post_id = s.last_id, last_post_at = s.last_at \
         from ( \
            select b2.id, \
                   (select count(*)::int from topics t where t.board_id = b2.id) as topics, \
                   (select count(*)::int from posts p join topics t on t.id = p.topic_id \
                    where t.board_id = b2.id) as posts, \
                   (select t.last_post_id from topics t where t.board_id = b2.id \
                    order by t.last_post_at desc, t.id desc limit 1) as last_id, \
                   (select t.last_post_at from topics t where t.board_id = b2.id \
                    order by t.last_post_at desc, t.id desc limit 1) as last_at \
            from boards b2) s \
         where b.id = s.id \
           and (b.topic_count is distinct from s.topics \
                or b.post_count is distinct from s.posts \
                or b.last_post_id is distinct from s.last_id \
                or b.last_post_at is distinct from s.last_at)",
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    Ok(RecountReport {
        users,
        topics,
        boards,
    })
}
