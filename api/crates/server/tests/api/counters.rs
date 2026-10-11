//! Denormalized counters, hard-delete cascades and the `recount` repair tool.

use axum::http::StatusCode;
use hypercs_db::maintenance::RecountReport;
use serde_json::json;
use uuid::Uuid;

use crate::common::*;

/// Counters are consistent exactly when the repair tool has nothing to fix.
async fn assert_consistent(t: &TestApp, context: &str) {
    // `recount_all` only writes rows that differ, and reports how many.
    let mut tx = t.pool.begin().await.unwrap();
    let report = hypercs_db::maintenance::recount_all_in(&mut tx)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    assert_eq!(
        report,
        RecountReport::default(),
        "denormalized counters drifted ({context})"
    );
}

#[tokio::test]
async fn counters_match_after_create_reply_edit_delete_move_sequences() {
    let t = TestApp::new().await;
    let w = t.world().await;

    let t1 = w.member.create_topic(w.board, "Topic one", "one").await;
    assert_consistent(&t, "create topic").await;
    let r1 = w.other.reply(t1, "reply 1").await;
    let r2 = w.member.reply(t1, "reply 2").await;
    assert_consistent(&t, "replies").await;
    assert_eq!(t.counts(w.board).await, (1, 3));
    assert_eq!(t.user_post_count(w.member_id).await, 2);
    assert_eq!(t.user_post_count(w.other_id).await, 1);

    w.other
        .patch(&format!("/api/v1/posts/{r1}"), json!({"body": "edited"}))
        .await
        .no_content();
    assert_consistent(&t, "edit").await;

    // deleting the most recent post moves the last-post pointers back
    w.member
        .delete(&format!("/api/v1/posts/{r2}"))
        .await
        .no_content();
    assert_consistent(&t, "delete last reply").await;
    let index = w.guest.get("/api/v1/categories").await;
    assert_eq!(
        index.body["categories"][0]["boards"][0]["lastPost"]["postId"],
        r1.to_string()
    );

    let t2 = w.other.create_topic(w.board, "Topic two", "two").await;
    w.member.reply(t2, "r").await;
    w.moderator
        .patch(
            &format!("/api/v1/topics/{t2}"),
            json!({"boardId": w.board2}),
        )
        .await
        .no_content();
    assert_consistent(&t, "move").await;
    assert_eq!(t.counts(w.board).await, (1, 2));
    assert_eq!(t.counts(w.board2).await, (1, 2));

    // delete the first post of topic one: whole topic goes
    let first: (Uuid,) = sqlx::query_as("select first_post_id from topics where id = $1")
        .bind(t1)
        .fetch_one(&t.pool)
        .await
        .unwrap();
    w.member
        .delete(&format!("/api/v1/posts/{}", first.0))
        .await
        .no_content();
    assert_consistent(&t, "delete first post").await;
    assert_eq!(t.counts(w.board).await, (0, 0));
    assert_eq!(t.scalar_i64("select count(*) from topics").await, 1);
    assert_eq!(t.scalar_i64("select count(*) from posts").await, 2);

    w.admin
        .delete(&format!("/api/v1/topics/{t2}"))
        .await
        .no_content();
    assert_consistent(&t, "delete topic").await;
    assert_eq!(t.counts(w.board2).await, (0, 0));
    for user in [w.member_id, w.other_id] {
        assert_eq!(t.user_post_count(user).await, 0);
    }
}

/// A small deterministic xorshift generator (no extra dependency).
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

#[tokio::test]
async fn counters_stay_consistent_under_a_long_random_operation_sequence() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let writers = [&w.member, &w.other, &w.moderator, &w.admin];
    let boards = [w.board, w.board2];
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);

    for step in 0..120 {
        let topics: Vec<(Uuid,)> = sqlx::query_as("select id from topics order by id")
            .fetch_all(&t.pool)
            .await
            .unwrap();
        let replies: Vec<(Uuid,)> = sqlx::query_as(
            "select p.id from posts p join topics t on t.id = p.topic_id \
             where p.id <> t.first_post_id order by p.id",
        )
        .fetch_all(&t.pool)
        .await
        .unwrap();
        let writer = writers[rng.below(writers.len())];
        let op = rng.below(10);
        let label = format!("step {step}, op {op}");
        match op {
            0..=2 => {
                let board = boards[rng.below(2)];
                writer
                    .create_topic(board, &format!("Topic {step}"), &format!("body {step}"))
                    .await;
            }
            3..=5 if !topics.is_empty() => {
                let topic = topics[rng.below(topics.len())].0;
                writer.reply(topic, &format!("reply {step}")).await;
            }
            6 if !replies.is_empty() => {
                let post = replies[rng.below(replies.len())].0;
                w.moderator
                    .delete(&format!("/api/v1/posts/{post}"))
                    .await
                    .no_content();
            }
            7 if !topics.is_empty() => {
                let topic = topics[rng.below(topics.len())].0;
                w.moderator
                    .patch(
                        &format!("/api/v1/topics/{topic}"),
                        json!({"boardId": boards[rng.below(2)]}),
                    )
                    .await
                    .no_content();
            }
            8 if topics.len() > 2 => {
                let topic = topics[rng.below(topics.len())].0;
                w.admin
                    .delete(&format!("/api/v1/topics/{topic}"))
                    .await
                    .no_content();
            }
            9 if !topics.is_empty() => {
                // delete a topic's first post (removes the whole topic)
                let topic = topics[rng.below(topics.len())].0;
                let first: (Uuid,) =
                    sqlx::query_as("select first_post_id from topics where id = $1")
                        .bind(topic)
                        .fetch_one(&t.pool)
                        .await
                        .unwrap();
                w.admin
                    .delete(&format!("/api/v1/posts/{}", first.0))
                    .await
                    .no_content();
            }
            _ => {}
        }
        assert_consistent(&t, &label).await;
    }
}

#[tokio::test]
async fn recount_repairs_deliberately_corrupted_counters() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let t1 = w.member.create_topic(w.board, "Topic one", "one").await;
    w.other.reply(t1, "reply").await;
    let t2 = w.other.create_topic(w.board2, "Topic two", "two").await;
    w.member.reply(t2, "reply").await;
    w.member.reply(t2, "reply 2").await;

    let before_topics = t
        .scalar_i64("select sum(reply_count)::bigint from topics")
        .await;
    assert_eq!(before_topics, 3);

    // corrupt everything the tool is responsible for
    sqlx::query("update users set post_count = 999")
        .execute(&t.pool)
        .await
        .unwrap();
    sqlx::query(
        "update topics set reply_count = 42, last_post_id = null, last_post_at = '2000-01-01'",
    )
    .execute(&t.pool)
    .await
    .unwrap();
    sqlx::query("update boards set topic_count = 7, post_count = 13, last_post_id = null, last_post_at = null")
        .execute(&t.pool)
        .await
        .unwrap();

    let report = hypercs_server::recount::run(&t.pool).await.unwrap();
    assert!(
        report.users >= 2 && report.topics == 2 && report.boards >= 2,
        "{report:?}"
    );

    assert_eq!(t.counts(w.board).await, (1, 2));
    assert_eq!(t.counts(w.board2).await, (1, 3));
    assert_eq!(t.counts(w.admin_board).await, (0, 0));
    assert_eq!(t.user_post_count(w.member_id).await, 3);
    assert_eq!(t.user_post_count(w.other_id).await, 2);
    assert_eq!(t.user_post_count(w.admin_id).await, 0);
    let (reply_count, last_matches): (i32, bool) = sqlx::query_as(
        "select reply_count, last_post_id = (select id from posts where topic_id = $1 order by id desc limit 1) \
         from topics where id = $1",
    )
    .bind(t2)
    .fetch_one(&t.pool)
    .await
    .unwrap();
    assert_eq!(reply_count, 2);
    assert!(last_matches);
    let index = w.guest.get("/api/v1/categories").await;
    assert_eq!(
        index.body["categories"][0]["boards"][1]["lastPost"]["topicTitle"],
        "Topic two"
    );

    // idempotent: a second run has nothing left to fix
    let again = hypercs_server::recount::run(&t.pool).await.unwrap();
    assert_eq!(again, RecountReport::default());
}

async fn seed_content(t: &TestApp, w: &World) -> (Uuid, Uuid) {
    let topic = w
        .member
        .create_topic(w.board, "Doomed topic", "first")
        .await;
    let reply = w.other.reply(topic, "a reply").await;
    // edits create revisions that must go away with the post
    w.other
        .patch(&format!("/api/v1/posts/{reply}"), json!({"body": "edited"}))
        .await
        .no_content();
    let first: (Uuid,) = sqlx::query_as("select first_post_id from topics where id = $1")
        .bind(topic)
        .fetch_one(&t.pool)
        .await
        .unwrap();
    w.member
        .patch(
            &format!("/api/v1/posts/{}", first.0),
            json!({"body": "first edited"}),
        )
        .await
        .no_content();
    (topic, reply)
}

#[tokio::test]
async fn deleting_a_post_removes_its_revisions() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let (_, reply) = seed_content(&t, &w).await;
    assert_eq!(t.scalar_i64("select count(*) from post_revisions").await, 2);
    w.other
        .delete(&format!("/api/v1/posts/{reply}"))
        .await
        .no_content();
    assert_eq!(t.scalar_i64("select count(*) from post_revisions").await, 1);
    assert_consistent(&t, "delete post").await;
}

#[tokio::test]
async fn deleting_the_first_post_cascades_to_the_topic_posts_and_revisions() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let (topic, _) = seed_content(&t, &w).await;
    let first: (Uuid,) = sqlx::query_as("select first_post_id from topics where id = $1")
        .bind(topic)
        .fetch_one(&t.pool)
        .await
        .unwrap();
    w.member
        .delete(&format!("/api/v1/posts/{}", first.0))
        .await
        .no_content();
    for table in ["topics", "posts", "post_revisions"] {
        assert_eq!(t.count(table).await, 0, "{table} should be empty");
    }
    assert_consistent(&t, "first post delete").await;
    assert_eq!(t.user_post_count(w.other_id).await, 0);
}

#[tokio::test]
async fn the_database_itself_deletes_the_topic_with_its_first_post() {
    // Backstop for deletes that bypass the service layer.
    let t = TestApp::new().await;
    let w = t.world().await;
    let (topic, _) = seed_content(&t, &w).await;
    let first: (Uuid,) = sqlx::query_as("select first_post_id from topics where id = $1")
        .bind(topic)
        .fetch_one(&t.pool)
        .await
        .unwrap();
    sqlx::query("delete from posts where id = $1")
        .bind(first.0)
        .execute(&t.pool)
        .await
        .unwrap();
    assert_eq!(t.scalar_i64("select count(*) from topics").await, 0);
    assert_eq!(t.scalar_i64("select count(*) from posts").await, 0);
}

#[tokio::test]
async fn deleting_a_board_cascades_and_fixes_authors_post_counts() {
    let t = TestApp::new().await;
    let w = t.world().await;
    seed_content(&t, &w).await;
    // content in the other board survives
    w.member.create_topic(w.board2, "Survivor", "stays").await;
    assert_eq!(t.user_post_count(w.member_id).await, 2);

    w.admin
        .delete(&format!("/api/v1/boards/{}", w.board))
        .await
        .no_content();
    assert_eq!(t.scalar_i64("select count(*) from topics").await, 1);
    assert_eq!(t.scalar_i64("select count(*) from posts").await, 1);
    assert_eq!(t.scalar_i64("select count(*) from post_revisions").await, 0);
    assert_eq!(t.user_post_count(w.member_id).await, 1);
    assert_eq!(t.user_post_count(w.other_id).await, 0);
    assert_consistent(&t, "board delete").await;
    w.admin
        .delete(&format!("/api/v1/boards/{}", w.board))
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn deleting_a_category_cascades_through_boards_topics_and_posts() {
    let t = TestApp::new().await;
    let w = t.world().await;
    seed_content(&t, &w).await;
    w.member.create_topic(w.board2, "Also doomed", "x").await;
    let other_cat = t.create_category("Other", false).await;
    let other_board = t.create_board(other_cat, "Other board", false).await;
    w.other.create_topic(other_board, "Survivor", "stays").await;

    w.admin
        .delete(&format!("/api/v1/categories/{}", w.category))
        .await
        .no_content();
    let remaining: (i64,) = sqlx::query_as("select count(*) from boards where category_id = $1")
        .bind(w.category)
        .fetch_one(&t.pool)
        .await
        .unwrap();
    assert_eq!(remaining.0, 0);
    assert_eq!(t.scalar_i64("select count(*) from topics").await, 1);
    assert_eq!(t.scalar_i64("select count(*) from posts").await, 1);
    assert_eq!(t.scalar_i64("select count(*) from post_revisions").await, 0);
    assert_eq!(t.user_post_count(w.member_id).await, 0);
    assert_eq!(t.user_post_count(w.other_id).await, 1);
    assert_consistent(&t, "category delete").await;
}
