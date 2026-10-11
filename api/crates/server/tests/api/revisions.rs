//! Post edit history: revision chain, concurrency and restore.

use std::collections::BTreeSet;

use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;

use crate::common::*;

async fn revision_bodies_oldest_first(t: &TestApp, post: Uuid) -> Vec<String> {
    let rows: Vec<(String,)> =
        sqlx::query_as("select body from post_revisions where post_id = $1 order by id")
            .bind(post)
            .fetch_all(&t.pool)
            .await
            .unwrap();
    rows.into_iter().map(|r| r.0).collect()
}

async fn current_body(t: &TestApp, post: Uuid) -> String {
    let row: (String,) = sqlx::query_as("select body from posts where id = $1")
        .bind(post)
        .fetch_one(&t.pool)
        .await
        .unwrap();
    row.0
}

#[tokio::test]
async fn every_edit_stores_the_previous_body_as_a_revision() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w.member.create_topic(w.board, "Hello there", "v0").await;
    let post = w.member.reply(topic, "v1").await;
    let path = format!("/api/v1/posts/{post}");

    w.member
        .patch(&path, json!({"body": "v2"}))
        .await
        .no_content();
    w.moderator
        .patch(&path, json!({"body": "v3"}))
        .await
        .no_content();
    // an unchanged body is a no-op: no revision, no new "edited" state
    w.moderator
        .patch(&path, json!({"body": "v3"}))
        .await
        .no_content();

    assert_eq!(revision_bodies_oldest_first(&t, post).await, ["v1", "v2"]);
    assert_eq!(current_body(&t, post).await, "v3");

    let history = w.member.get(&format!("{path}/revisions")).await;
    history.ok();
    let items = history.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["body"], "v2", "newest first");
    assert_eq!(items[0]["editedBy"]["username"], "mod_mia");
    assert_eq!(items[1]["body"], "v1");
    assert_eq!(items[1]["editedBy"]["username"], "member_max");
    assert_eq!(history.body["total"], 2);
    assert_eq!(history.body["perPage"], 20);

    // pagination
    let page = w
        .member
        .get(&format!("{path}/revisions?perPage=1&page=2"))
        .await;
    assert_eq!(page.body["items"][0]["body"], "v1");
    assert_eq!(page.body["totalPages"], 2);
}

#[tokio::test]
async fn history_is_visible_to_author_and_staff_only() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w.member.create_topic(w.board, "Hello there", "v0").await;
    let post = w.member.reply(topic, "v1").await;
    let path = format!("/api/v1/posts/{post}/revisions");
    w.member
        .patch(&format!("/api/v1/posts/{post}"), json!({"body": "v2"}))
        .await
        .no_content();

    for client in [&w.member, &w.moderator, &w.admin] {
        client.get(&path).await.ok();
    }
    w.other
        .get(&path)
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    w.guest
        .get(&path)
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    w.member
        .get(&format!("/api/v1/posts/{}/revisions", Uuid::now_v7()))
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn concurrent_edits_serialize_and_leave_a_complete_revision_chain() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w.member.create_topic(w.board, "Hello there", "base").await;
    let post = w.member.reply(topic, "original").await;
    let path = format!("/api/v1/posts/{post}");

    // Fire many edits at the same time from different editors.
    let editors = [w.member.clone(), w.moderator.clone(), w.admin.clone()];
    let n = 12;
    let mut tasks = Vec::new();
    for i in 0..n {
        let client = editors[i % editors.len()].clone();
        let path = path.clone();
        tasks.push(tokio::spawn(async move {
            client
                .patch(&path, json!({"body": format!("edit {i}")}))
                .await
                .status
        }));
    }
    for task in tasks {
        assert_eq!(task.await.unwrap(), StatusCode::NO_CONTENT);
    }

    // Nothing was lost: the original plus all but the last-applied edit are
    // in the history, and the last-applied edit is the current body.
    let revisions = revision_bodies_oldest_first(&t, post).await;
    let current = current_body(&t, post).await;
    assert_eq!(revisions.len(), n, "one revision per edit");
    assert_eq!(
        revisions[0], "original",
        "the oldest revision is the original body"
    );

    let mut seen: BTreeSet<String> = revisions.iter().cloned().collect();
    assert_eq!(seen.len(), n, "no duplicates in the chain");
    assert!(
        seen.insert(current.clone()),
        "current body is not also a revision"
    );
    let mut expected: BTreeSet<String> = (0..n).map(|i| format!("edit {i}")).collect();
    expected.insert("original".to_string());
    assert_eq!(
        seen, expected,
        "every submitted version is in the chain exactly once"
    );

    // The chain is ordered consistently with the edits: each revision's
    // editor row exists and edited_by/edited_at on the post are the last edit.
    let (edited_by, has_edited_at): (Option<Uuid>, bool) =
        sqlx::query_as("select edited_by, edited_at is not null from posts where id = $1")
            .bind(post)
            .fetch_one(&t.pool)
            .await
            .unwrap();
    assert!(edited_by.is_some() && has_edited_at);
    let revision_editors: i64 = t
        .scalar_i64("select count(*) from post_revisions where edited_by is not null")
        .await;
    assert_eq!(revision_editors as usize, n);
}

#[tokio::test]
async fn restoring_a_revision_is_a_new_edit_and_never_rewrites_history() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w.member.create_topic(w.board, "Hello there", "base").await;
    let post = w.member.reply(topic, "v0").await;
    let path = format!("/api/v1/posts/{post}");
    for body in ["v1", "v2", "v3"] {
        w.member
            .patch(&path, json!({"body": body}))
            .await
            .no_content();
    }
    assert_eq!(
        revision_bodies_oldest_first(&t, post).await,
        ["v0", "v1", "v2"]
    );

    let history = w.member.get(&format!("{path}/revisions")).await;
    let v1_revision = history.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["body"] == "v1")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // a moderator restores v1
    w.moderator
        .post_empty(&format!("{path}/revisions/{v1_revision}/restore"))
        .await
        .no_content();
    assert_eq!(current_body(&t, post).await, "v1");
    // a NEW revision holds the body that was replaced; old ones are untouched
    assert_eq!(
        revision_bodies_oldest_first(&t, post).await,
        ["v0", "v1", "v2", "v3"]
    );
    let (editor,): (String,) = sqlx::query_as(
        "select u.username from posts p join users u on u.id = p.edited_by where p.id = $1",
    )
    .bind(post)
    .fetch_one(&t.pool)
    .await
    .unwrap();
    assert_eq!(editor, "mod_mia");
}

#[tokio::test]
async fn restore_goes_through_authorization_and_validates_the_revision() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w.member.create_topic(w.board, "Hello there", "base").await;
    let post = w.member.reply(topic, "v0").await;
    let other_post = w.other.reply(topic, "o0").await;
    w.member
        .patch(&format!("/api/v1/posts/{post}"), json!({"body": "v1"}))
        .await
        .no_content();
    w.other
        .patch(
            &format!("/api/v1/posts/{other_post}"),
            json!({"body": "o1"}),
        )
        .await
        .no_content();
    let revision: (Uuid,) = sqlx::query_as("select id from post_revisions where post_id = $1")
        .bind(post)
        .fetch_one(&t.pool)
        .await
        .unwrap();
    let other_revision: (Uuid,) =
        sqlx::query_as("select id from post_revisions where post_id = $1")
            .bind(other_post)
            .fetch_one(&t.pool)
            .await
            .unwrap();
    let url = |p: Uuid, r: Uuid| format!("/api/v1/posts/{p}/revisions/{r}/restore");

    w.other
        .post_empty(&url(post, revision.0))
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    w.guest
        .post_empty(&url(post, revision.0))
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    w.member
        .without_csrf()
        .post_empty(&url(post, revision.0))
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    // a revision of a different post cannot be used
    w.member
        .post_empty(&url(post, other_revision.0))
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
    w.member
        .post_empty(&url(post, Uuid::now_v7()))
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
    assert_eq!(current_body(&t, post).await, "v1");

    w.member
        .post_empty(&url(post, revision.0))
        .await
        .no_content();
    assert_eq!(current_body(&t, post).await, "v0");
}

#[tokio::test]
async fn get_post_returns_the_post_with_topic_context() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w.member.create_topic(w.board, "Hello there", "first").await;
    let mut last = topic;
    for i in 0..16 {
        last = w.other.reply(topic, &format!("reply {i}")).await;
    }
    let first: Uuid = sqlx::query_scalar("select first_post_id from topics where id = $1")
        .bind(topic)
        .fetch_one(&t.pool)
        .await
        .unwrap();
    w.member
        .patch(&format!("/api/v1/posts/{last}"), json!({"body": "x"}))
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    w.other
        .patch(&format!("/api/v1/posts/{last}"), json!({"body": "edited"}))
        .await
        .no_content();

    // guests may read
    let res = w.guest.get(&format!("/api/v1/posts/{first}")).await;
    res.ok();
    assert_eq!(res.body["post"]["id"], first.to_string());
    assert_eq!(res.body["post"]["number"], 1);
    assert_eq!(res.body["post"]["body"], "first");
    assert_eq!(res.body["post"]["author"]["username"], "member_max");
    let c = &res.body["context"];
    assert_eq!(c["topicId"], topic.to_string());
    assert_eq!(c["topicTitle"], "Hello there");
    assert_eq!(c["topicSlug"], "hello-there");
    assert_eq!(c["topicLocked"], false);
    assert_eq!(c["boardId"], w.board.to_string());
    assert_eq!(c["boardName"], "Public board");
    assert_eq!(c["categoryId"], w.category.to_string());
    assert_eq!(c["categoryName"], "Public category");
    assert_eq!(c["isFirstPost"], true);
    assert_eq!(c["page"], 1);

    let res = w.member.get(&format!("/api/v1/posts/{last}")).await;
    res.ok();
    assert_eq!(res.body["post"]["number"], 17);
    assert_eq!(res.body["post"]["body"], "edited");
    assert_eq!(res.body["post"]["editedBy"]["username"], "other_olga");
    assert!(res.body["post"]["editedAt"].is_string());
    assert_eq!(res.body["context"]["isFirstPost"], false);
    assert_eq!(res.body["context"]["page"], 2);
}

#[tokio::test]
async fn get_post_unknown_id_is_not_found() {
    let t = TestApp::new().await;
    let w = t.world().await;
    w.guest
        .get(&format!("/api/v1/posts/{}", Uuid::now_v7()))
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
    w.member
        .get("/api/v1/posts/not-a-uuid")
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn get_post_in_admin_only_board_is_hidden_from_non_admins() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w.admin.create_topic(w.admin_board, "Secret", "hush").await;
    let post: Uuid = sqlx::query_scalar("select first_post_id from topics where id = $1")
        .bind(topic)
        .fetch_one(&t.pool)
        .await
        .unwrap();
    let path = format!("/api/v1/posts/{post}");
    for client in [&w.guest, &w.member, &w.moderator] {
        client
            .get(&path)
            .await
            .assert_problem(StatusCode::NOT_FOUND, "not_found");
    }
    w.admin.get(&path).await.ok();
}
