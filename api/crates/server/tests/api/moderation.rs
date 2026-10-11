//! Moderation: pin/lock/move/delete, post edit/delete authorization and the
//! structure (category/board) management endpoints.

use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;

use crate::common::*;

#[tokio::test]
async fn pin_and_lock_toggle_independently_and_require_staff() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w.member.create_topic(w.board, "Hello there", "first").await;
    let path = format!("/api/v1/topics/{topic}");

    w.moderator
        .patch(&path, json!({"pinned": true}))
        .await
        .no_content();
    let page = w.guest.get(&path).await;
    assert_eq!(page.body["topic"]["pinned"], true);
    assert_eq!(page.body["topic"]["locked"], false);
    w.admin
        .patch(&path, json!({"locked": true}))
        .await
        .no_content();
    let page = w.guest.get(&path).await;
    assert_eq!(
        page.body["topic"]["pinned"], true,
        "untouched field is kept"
    );
    assert_eq!(page.body["topic"]["locked"], true);
    w.moderator
        .patch(&path, json!({"pinned": false, "locked": false}))
        .await
        .no_content();

    // The author is not a moderator of their own topic.
    w.member
        .patch(&path, json!({"locked": true}))
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    w.guest
        .patch(&path, json!({"locked": true}))
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    w.moderator
        .patch(
            &format!("/api/v1/topics/{}", Uuid::now_v7()),
            json!({"locked": true}),
        )
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn moving_a_topic_fixes_counters_and_last_posts_of_both_boards() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w
        .member
        .create_topic(w.board, "Moving topic", "first")
        .await;
    w.other.reply(topic, "second").await;
    let stay = w.other.create_topic(w.board, "Staying topic", "stay").await;
    assert_eq!(t.counts(w.board).await, (2, 3));
    assert_eq!(t.counts(w.board2).await, (0, 0));

    w.moderator
        .patch(
            &format!("/api/v1/topics/{topic}"),
            json!({"boardId": w.board2}),
        )
        .await
        .no_content();
    assert_eq!(t.counts(w.board).await, (1, 1));
    assert_eq!(t.counts(w.board2).await, (1, 2));

    let index = w.guest.get("/api/v1/categories").await;
    let boards = &index.body["categories"][0]["boards"];
    assert_eq!(boards[0]["lastPost"]["topicTitle"], "Staying topic");
    assert_eq!(boards[1]["lastPost"]["topicTitle"], "Moving topic");
    assert_eq!(boards[1]["lastPost"]["author"]["username"], "other_olga");
    let page = w.guest.get(&format!("/api/v1/boards/{}", w.board2)).await;
    assert_eq!(page.body["topics"]["items"][0]["id"], topic.to_string());
    let _ = stay;

    // pin + move in one request, and moving to the same board is a no-op
    w.admin
        .patch(
            &format!("/api/v1/topics/{topic}"),
            json!({"pinned": true, "boardId": w.board}),
        )
        .await
        .no_content();
    assert_eq!(t.counts(w.board).await, (2, 3));
    assert_eq!(t.counts(w.board2).await, (0, 0));
    w.admin
        .patch(
            &format!("/api/v1/topics/{topic}"),
            json!({"boardId": w.board}),
        )
        .await
        .no_content();
    assert_eq!(t.counts(w.board).await, (2, 3));

    // invalid destinations
    w.moderator
        .patch(
            &format!("/api/v1/topics/{topic}"),
            json!({"boardId": Uuid::now_v7()}),
        )
        .await
        .assert_field_error("boardId", "board_not_found");
    // moderators cannot move topics into boards they cannot see
    w.moderator
        .patch(
            &format!("/api/v1/topics/{topic}"),
            json!({"boardId": w.admin_board}),
        )
        .await
        .assert_field_error("boardId", "board_not_found");
    w.admin
        .patch(
            &format!("/api/v1/topics/{topic}"),
            json!({"boardId": w.admin_board}),
        )
        .await
        .no_content();
    assert_eq!(t.counts(w.admin_board).await, (1, 2));
    // members cannot move
    w.member
        .patch(
            &format!("/api/v1/topics/{topic}"),
            json!({"boardId": w.board}),
        )
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found"); // not visible any more
}

#[tokio::test]
async fn topics_can_be_deleted_by_staff_only_with_counters_fixed() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w
        .member
        .create_topic(w.board, "Doomed topic", "first")
        .await;
    w.other.reply(topic, "reply").await;
    w.other.reply(topic, "reply 2").await;
    let keep = w.other.create_topic(w.board, "Kept topic", "keep").await;

    w.member
        .delete(&format!("/api/v1/topics/{topic}"))
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    w.guest
        .delete(&format!("/api/v1/topics/{topic}"))
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");

    w.moderator
        .delete(&format!("/api/v1/topics/{topic}"))
        .await
        .no_content();
    w.guest
        .get(&format!("/api/v1/topics/{topic}"))
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
    assert_eq!(t.counts(w.board).await, (1, 1));
    assert_eq!(t.user_post_count(w.member_id).await, 0);
    assert_eq!(t.user_post_count(w.other_id).await, 1);
    assert_eq!(t.scalar_i64("select count(*) from posts").await, 1);
    let index = w.guest.get("/api/v1/categories").await;
    assert_eq!(
        index.body["categories"][0]["boards"][0]["lastPost"]["topicId"],
        keep.to_string()
    );
    w.moderator
        .delete(&format!("/api/v1/topics/{topic}"))
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn post_edit_and_delete_authorization() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w.member.create_topic(w.board, "Hello there", "first").await;
    let reply = w.member.reply(topic, "my reply").await;
    let path = format!("/api/v1/posts/{reply}");

    // someone else cannot edit or delete
    w.other
        .patch(&path, json!({"body": "hijacked"}))
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    w.other
        .delete(&path)
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    w.guest
        .patch(&path, json!({"body": "hijacked"}))
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    w.guest
        .delete(&path)
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    w.other
        .patch(
            &format!("/api/v1/posts/{}", Uuid::now_v7()),
            json!({"body": "x"}),
        )
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");

    // the author and staff can
    w.member
        .patch(&path, json!({"body": "edited by me"}))
        .await
        .no_content();
    w.moderator
        .patch(&path, json!({"body": "edited by mod"}))
        .await
        .no_content();
    w.admin
        .patch(&path, json!({"body": "edited by admin"}))
        .await
        .no_content();
    let page = w.guest.get(&format!("/api/v1/topics/{topic}")).await;
    let post = &page.body["posts"]["items"][1];
    assert_eq!(post["body"], "edited by admin");
    assert!(post["editedAt"].is_string());
    assert_eq!(post["editedBy"]["username"], "root_admin");

    w.member.delete(&path).await.no_content();
    w.moderator
        .delete(&path)
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
    // moderators can delete other people's posts
    let other_reply = w.other.reply(topic, "olga's reply").await;
    w.moderator
        .delete(&format!("/api/v1/posts/{other_reply}"))
        .await
        .no_content();
    // edit validation
    w.member
        .patch(
            &format!("/api/v1/posts/{}", t.first_post(topic).await),
            json!({"body": ""}),
        )
        .await
        .assert_field_error("body", "length");
}

impl TestApp {
    async fn first_post(&self, topic: Uuid) -> Uuid {
        let row: (Uuid,) = sqlx::query_as("select first_post_id from topics where id = $1")
            .bind(topic)
            .fetch_one(&self.pool)
            .await
            .unwrap();
        row.0
    }
}

#[tokio::test]
async fn category_and_board_management_by_admins() {
    let t = TestApp::new().await;
    let w = t.world().await;

    // create + Location + defaults
    let res = w
        .admin
        .post(
            "/api/v1/categories",
            json!({"name": "  New category  ", "description": "About it"}),
        )
        .await;
    let category = res.created_id();
    let board = w
        .admin
        .post(
            "/api/v1/boards",
            json!({"categoryId": category, "name": "New board", "adminOnly": false}),
        )
        .await
        .created_id();

    // partial update
    w.admin
        .patch(
            &format!("/api/v1/categories/{category}"),
            json!({"name": "Renamed"}),
        )
        .await
        .no_content();
    w.admin
        .patch(
            &format!("/api/v1/boards/{board}"),
            json!({"description": "Now with description", "adminOnly": true}),
        )
        .await
        .no_content();
    let (cat_name, cat_desc): (String, String) =
        sqlx::query_as("select name, description from categories where id = $1")
            .bind(category)
            .fetch_one(&t.pool)
            .await
            .unwrap();
    assert_eq!(
        (cat_name.as_str(), cat_desc.as_str()),
        ("Renamed", "About it")
    );
    let (b_name, b_admin): (String, bool) =
        sqlx::query_as("select name, admin_only from boards where id = $1")
            .bind(board)
            .fetch_one(&t.pool)
            .await
            .unwrap();
    assert_eq!((b_name.as_str(), b_admin), ("New board", true));

    // validation
    w.admin
        .post("/api/v1/categories", json!({"name": ""}))
        .await
        .assert_field_error("name", "length");
    w.admin
        .post("/api/v1/categories", json!({"name": "n".repeat(61)}))
        .await
        .assert_field_error("name", "length");
    w.admin
        .post(
            "/api/v1/categories",
            json!({"name": "ok", "description": "d".repeat(201)}),
        )
        .await
        .assert_field_error("description", "length");
    w.admin
        .post(
            "/api/v1/boards",
            json!({"categoryId": Uuid::now_v7(), "name": "Orphan"}),
        )
        .await
        .assert_field_error("categoryId", "category_not_found");
    w.admin
        .patch(
            &format!("/api/v1/boards/{}", Uuid::now_v7()),
            json!({"name": "x"}),
        )
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn reordering_moves_categories_and_boards_one_step() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let third = t.create_board(w.category, "Third board", false).await;

    let order = |rows: serde_json::Value| -> Vec<String> {
        rows["categories"][0]["boards"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b["name"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(
        order(w.admin.get("/api/v1/categories").await.body),
        ["Public board", "Second board", "Third board"]
    );
    w.admin
        .post(
            &format!("/api/v1/boards/{third}/reorder"),
            json!({"direction": "up"}),
        )
        .await
        .no_content();
    assert_eq!(
        order(w.admin.get("/api/v1/categories").await.body),
        ["Public board", "Third board", "Second board"]
    );
    // at the edge: no-op
    w.admin
        .post(
            &format!("/api/v1/boards/{}/reorder", w.board),
            json!({"direction": "up"}),
        )
        .await
        .no_content();
    w.admin
        .post(
            &format!("/api/v1/boards/{}/reorder", w.board),
            json!({"direction": "down"}),
        )
        .await
        .no_content();
    assert_eq!(
        order(w.admin.get("/api/v1/categories").await.body),
        ["Third board", "Public board", "Second board"]
    );

    // categories
    let index = w.admin.get("/api/v1/categories").await;
    assert_eq!(index.body["categories"][0]["name"], "Public category");
    w.admin
        .post(
            &format!("/api/v1/categories/{}/reorder", w.category),
            json!({"direction": "down"}),
        )
        .await
        .no_content();
    let index = w.admin.get("/api/v1/categories").await;
    assert_eq!(index.body["categories"][0]["name"], "Admins only");
    assert_eq!(index.body["categories"][1]["name"], "Public category");

    w.admin
        .post(
            &format!("/api/v1/categories/{}/reorder", w.category),
            json!({"direction": "sideways"}),
        )
        .await
        .assert_problem(StatusCode::UNPROCESSABLE_ENTITY, "validation_failed");
    w.admin
        .post(
            &format!("/api/v1/categories/{}/reorder", Uuid::now_v7()),
            json!({"direction": "up"}),
        )
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn structure_management_requires_admin() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let c = w.category;
    let b = w.board;
    let cases: Vec<(&str, String, serde_json::Value)> = vec![
        ("POST", "/api/v1/categories".into(), json!({"name": "x"})),
        (
            "PATCH",
            format!("/api/v1/categories/{c}"),
            json!({"name": "x"}),
        ),
        ("DELETE", format!("/api/v1/categories/{c}"), json!(null)),
        (
            "POST",
            format!("/api/v1/categories/{c}/reorder"),
            json!({"direction": "up"}),
        ),
        (
            "POST",
            "/api/v1/boards".into(),
            json!({"categoryId": c, "name": "x"}),
        ),
        ("PATCH", format!("/api/v1/boards/{b}"), json!({"name": "x"})),
        ("DELETE", format!("/api/v1/boards/{b}"), json!(null)),
        (
            "POST",
            format!("/api/v1/boards/{b}/reorder"),
            json!({"direction": "up"}),
        ),
    ];
    for (method, path, body) in cases {
        let method: axum::http::Method = method.parse().unwrap();
        let body = (!body.is_null()).then_some(body);
        w.guest
            .request(method.clone(), &path, body.clone())
            .await
            .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
        for client in [&w.member, &w.moderator] {
            client
                .request(method.clone(), &path, body.clone())
                .await
                .assert_problem(StatusCode::FORBIDDEN, "forbidden");
        }
    }
    assert_eq!(t.scalar_i64("select count(*) from categories").await, 2);
}
