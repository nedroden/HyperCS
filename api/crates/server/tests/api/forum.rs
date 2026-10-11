//! Reading and writing the forum: board index, boards, topics, replies,
//! input handling, body limits and flood control.

use axum::http::{Method, StatusCode};
use hypercs_api::state::FloodConfig;
use serde_json::json;
use uuid::Uuid;

use crate::common::*;

#[tokio::test]
async fn board_index_lists_categories_boards_and_hides_admin_only_from_non_admins() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w
        .member
        .create_topic(w.board, "Hello there", "First post")
        .await;
    w.other.reply(topic, "A reply").await;

    let index = w.guest.get("/api/v1/categories").await;
    index.ok();
    let categories = index.body["categories"].as_array().unwrap();
    assert_eq!(
        categories.len(),
        1,
        "admin-only category hidden from guests"
    );
    assert_eq!(categories[0]["name"], "Public category");
    let boards = categories[0]["boards"].as_array().unwrap();
    assert_eq!(boards.len(), 2);
    assert_eq!(boards[0]["name"], "Public board");
    assert_eq!(boards[0]["topicCount"], 1);
    assert_eq!(boards[0]["postCount"], 2);
    assert_eq!(boards[0]["lastPost"]["topicTitle"], "Hello there");
    assert_eq!(boards[0]["lastPost"]["author"]["username"], "other_olga");
    assert_eq!(boards[0]["lastPost"]["author"]["role"], "member");
    assert!(
        boards[1]["lastPost"].is_null(),
        "no last post for empty board"
    );

    for client in [&w.member, &w.moderator] {
        let index = client.get("/api/v1/categories").await;
        assert_eq!(index.body["categories"].as_array().unwrap().len(), 1);
    }
    let admin_index = w.admin.get("/api/v1/categories").await;
    let admin_categories = admin_index.body["categories"].as_array().unwrap();
    assert_eq!(admin_categories.len(), 2);
    assert!(
        admin_categories
            .iter()
            .any(|c| c["adminOnly"] == true && c["boards"][0]["name"] == "Staff room")
    );
}

#[tokio::test]
async fn board_page_paginates_topics_pinned_first_then_latest_activity() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let first = w.member.create_topic(w.board, "First topic", "one").await;
    let second = w.member.create_topic(w.board, "Second topic", "two").await;
    let third = w.member.create_topic(w.board, "Third topic", "three").await;
    // activity bumps a topic to the top
    w.other.reply(first, "bump").await;
    // pinning wins over activity
    w.moderator
        .patch(&format!("/api/v1/topics/{second}"), json!({"pinned": true}))
        .await
        .no_content();

    let page = w.guest.get(&format!("/api/v1/boards/{}", w.board)).await;
    page.ok();
    assert_eq!(page.body["board"]["name"], "Public board");
    assert_eq!(page.body["board"]["categoryName"], "Public category");
    let ids: Vec<&str> = page.body["topics"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        vec![second.to_string(), first.to_string(), third.to_string()]
    );
    let item = &page.body["topics"]["items"][1];
    assert_eq!(item["slug"], "first-topic");
    assert_eq!(item["replyCount"], 1);
    assert_eq!(item["author"]["username"], "member_max");
    assert_eq!(item["lastPost"]["author"]["username"], "other_olga");
    assert_eq!(page.body["topics"]["perPage"], 20);
    assert_eq!(page.body["topics"]["total"], 3);
    assert_eq!(page.body["topics"]["totalPages"], 1);

    let small = w
        .guest
        .get(&format!("/api/v1/boards/{}?perPage=2&page=2", w.board))
        .await;
    assert_eq!(small.body["topics"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(small.body["topics"]["totalPages"], 2);
    // perPage is capped
    let big = w
        .guest
        .get(&format!("/api/v1/boards/{}?perPage=100000", w.board))
        .await;
    assert_eq!(big.body["topics"]["perPage"], 100);
    // beyond the last page: empty, not an error
    let beyond = w
        .guest
        .get(&format!("/api/v1/boards/{}?page=99", w.board))
        .await;
    beyond.ok();
    assert!(
        beyond.body["topics"]["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn admin_only_boards_are_invisible_and_inaccessible_to_everyone_else() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let secret = w
        .admin
        .create_topic(w.admin_board, "Staff only", "secret")
        .await;
    let post_id: (Uuid,) = sqlx::query_as("select first_post_id from topics")
        .fetch_one(&t.pool)
        .await
        .unwrap();

    for client in [&w.guest, &w.member, &w.moderator] {
        client
            .get(&format!("/api/v1/boards/{}", w.admin_board))
            .await
            .assert_problem(StatusCode::NOT_FOUND, "not_found");
        client
            .get(&format!("/api/v1/topics/{secret}"))
            .await
            .assert_problem(StatusCode::NOT_FOUND, "not_found");
    }
    for client in [&w.member, &w.moderator] {
        client
            .post(
                &format!("/api/v1/boards/{}/topics", w.admin_board),
                json!({"title": "Sneaky", "body": "x"}),
            )
            .await
            .assert_problem(StatusCode::NOT_FOUND, "not_found");
        client
            .post(
                &format!("/api/v1/topics/{secret}/posts"),
                json!({"body": "x"}),
            )
            .await
            .assert_problem(StatusCode::NOT_FOUND, "not_found");
        // Moderators may not touch topics/posts they cannot see.
        client
            .patch(&format!("/api/v1/topics/{secret}"), json!({"locked": true}))
            .await
            .assert_problem(StatusCode::NOT_FOUND, "not_found");
        client
            .delete(&format!("/api/v1/topics/{secret}"))
            .await
            .assert_problem(StatusCode::NOT_FOUND, "not_found");
        client
            .delete(&format!("/api/v1/posts/{}", post_id.0))
            .await
            .assert_problem(StatusCode::NOT_FOUND, "not_found");
    }
    w.admin.get(&format!("/api/v1/topics/{secret}")).await.ok();
    // The statistics never leak the admin-only post.
    let stats = w.guest.get("/api/v1/stats").await;
    assert!(stats.body["latestPost"].is_null());
}

#[tokio::test]
async fn boards_inside_an_admin_only_category_are_hidden_too() {
    let t = TestApp::new().await;
    let w = t.world().await;
    // a board that is not flagged itself, inside an admin-only category
    let inner = t.create_board(w.admin_category, "Inherits", false).await;
    w.guest
        .get(&format!("/api/v1/boards/{inner}"))
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
    w.member
        .post(
            &format!("/api/v1/boards/{inner}/topics"),
            json!({"title": "Sneaky", "body": "x"}),
        )
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
    w.admin.get(&format!("/api/v1/boards/{inner}")).await.ok();
}

#[tokio::test]
async fn topic_creation_returns_201_location_and_the_topic_page_works() {
    let t = TestApp::new().await;
    let w = t.world().await;

    let res = w
        .member
        .post(
            &format!("/api/v1/boards/{}/topics", w.board),
            json!({"title": "  Größe & Übung!  ", "body": "# Heading\r\n\r\nSome *markdown*"}),
        )
        .await;
    let id = res.created_id();
    assert_eq!(
        res.header("location"),
        Some(format!("/api/v1/topics/{id}").as_str())
    );

    let page = w.guest.get(&format!("/api/v1/topics/{id}")).await;
    page.ok();
    let topic = &page.body["topic"];
    assert_eq!(topic["title"], "Größe & Übung!");
    assert_eq!(topic["slug"], "groesse-uebung");
    assert_eq!(topic["author"]["displayName"], "member_max");
    assert_eq!(topic["boardName"], "Public board");
    assert_eq!(topic["replyCount"], 0);
    assert_eq!(topic["locked"], false);
    let posts = page.body["posts"]["items"].as_array().unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0]["number"], 1);
    assert_eq!(posts[0]["body"], "# Heading\n\nSome *markdown*"); // \r\n -> \n
    assert_eq!(posts[0]["author"]["postCount"], 1);
    assert_eq!(posts[0]["id"], topic["firstPostId"]);
    assert!(posts[0]["editedAt"].is_null());

    // guests cannot create topics
    w.guest
        .post(
            &format!("/api/v1/boards/{}/topics", w.board),
            json!({"title": "Nope nope", "body": "x"}),
        )
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    w.member
        .post(
            &format!("/api/v1/boards/{}/topics", Uuid::now_v7()),
            json!({"title": "Nope nope", "body": "x"}),
        )
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn topic_and_post_input_is_validated_and_normalized() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let path = format!("/api/v1/boards/{}/topics", w.board);
    let create =
        |title: &str, body: &str| w.member.post(&path, json!({"title": title, "body": body}));

    create("ab", "body")
        .await
        .assert_field_error("title", "length");
    create(&"t".repeat(121), "body")
        .await
        .assert_field_error("title", "length");
    create("Valid title", "")
        .await
        .assert_field_error("body", "length");
    create("Valid title", &"b".repeat(10_001))
        .await
        .assert_field_error("body", "length");
    // whitespace-only bodies are blank
    create("Valid title", "   \n  ")
        .await
        .assert_field_error("body", "length");
    // control and invisible characters are rejected, not stripped
    create("Bad\u{0007}title", "body")
        .await
        .assert_field_error("title", "invalid_characters");
    create("Bad\ntitle", "body")
        .await
        .assert_field_error("title", "invalid_characters");
    create("Zero\u{200B}width", "body")
        .await
        .assert_field_error("title", "invalid_characters");
    create("Valid title", "bell\u{7}")
        .await
        .assert_field_error("body", "invalid_characters");
    create("Valid title", "nul\u{0}")
        .await
        .assert_field_error("body", "invalid_characters");
    // exactly at the limits is fine; newlines and tabs are allowed in bodies
    create(&"t".repeat(120), &format!("{}\n\tend", "b".repeat(9_990)))
        .await
        .created();
    assert_eq!(t.scalar_i64("select count(*) from topics").await, 1);

    // NFC normalization of titles and bodies
    let id = w
        .member
        .create_topic(w.board, "Cafe\u{0301} time", "Cafe\u{0301}")
        .await;
    let page = w.guest.get(&format!("/api/v1/topics/{id}")).await;
    assert_eq!(page.body["topic"]["title"], "Caf\u{e9} time");
    assert_eq!(page.body["posts"]["items"][0]["body"], "Caf\u{e9}");
}

#[tokio::test]
async fn replies_count_views_and_post_numbers() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w.member.create_topic(w.board, "Hello there", "first").await;
    for i in 0..16 {
        w.other.reply(topic, &format!("reply {i}")).await;
    }

    let page1 = w.guest.get(&format!("/api/v1/topics/{topic}")).await;
    assert_eq!(page1.body["posts"]["perPage"], 15);
    assert_eq!(page1.body["posts"]["total"], 17);
    assert_eq!(page1.body["posts"]["totalPages"], 2);
    assert_eq!(page1.body["posts"]["items"].as_array().unwrap().len(), 15);
    assert_eq!(page1.body["topic"]["replyCount"], 16);
    let page2 = w.guest.get(&format!("/api/v1/topics/{topic}?page=2")).await;
    let items = page2.body["posts"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["number"], 16);
    assert_eq!(items[1]["body"], "reply 15");

    // every page request is a view (the response shows the count before it)
    let views = t.scalar_i64("select view_count::bigint from topics").await;
    assert_eq!(views, 2);

    w.guest
        .post(
            &format!("/api/v1/topics/{topic}/posts"),
            json!({"body": "x"}),
        )
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    w.other
        .post(
            &format!("/api/v1/topics/{}/posts", Uuid::now_v7()),
            json!({"body": "x"}),
        )
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
    w.other
        .post(
            &format!("/api/v1/topics/{topic}/posts"),
            json!({"body": ""}),
        )
        .await
        .assert_field_error("body", "length");
}

#[tokio::test]
async fn locked_topics_accept_replies_from_staff_only() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w.member.create_topic(w.board, "Hello there", "first").await;
    w.moderator
        .patch(&format!("/api/v1/topics/{topic}"), json!({"locked": true}))
        .await
        .no_content();
    let path = format!("/api/v1/topics/{topic}/posts");

    for client in [&w.member, &w.other] {
        client
            .post(&path, json!({"body": "let me in"}))
            .await
            .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    }
    w.moderator
        .post(&path, json!({"body": "mod reply"}))
        .await
        .created();
    w.admin
        .post(&path, json!({"body": "admin reply"}))
        .await
        .created();

    w.moderator
        .patch(&format!("/api/v1/topics/{topic}"), json!({"locked": false}))
        .await
        .no_content();
    w.other
        .post(&path, json!({"body": "open again"}))
        .await
        .created();
}

#[tokio::test]
async fn oversized_bodies_get_413_problem_json_with_per_route_overrides() {
    let t = TestApp::new().await;
    let w = t.world().await;

    // global default is 64 KB
    let big = json!({"title": "Hello there", "body": "x".repeat(70 * 1024)});
    w.member
        .post(&format!("/api/v1/boards/{}/topics", w.board), big)
        .await
        .assert_problem(StatusCode::PAYLOAD_TOO_LARGE, "payload_too_large");

    // the auth forms have a much smaller limit (4 KB)
    t.guest()
        .post(
            "/api/v1/auth/login",
            json!({"username": "x".repeat(5000), "password": PASSWORD}),
        )
        .await
        .assert_problem(StatusCode::PAYLOAD_TOO_LARGE, "payload_too_large");

    // the terms route raises the limit: ~80 KB of valid text goes through
    let terms = "ä".repeat(20_000); // 40 KB in UTF-8
    w.admin
        .put("/api/v1/cms/terms", json!({"body": terms}))
        .await
        .no_content();
    let too_big = "x".repeat(130 * 1024);
    w.admin
        .put("/api/v1/cms/terms", json!({"body": too_big}))
        .await
        .assert_problem(StatusCode::PAYLOAD_TOO_LARGE, "payload_too_large");
}

#[tokio::test]
async fn malformed_requests_use_problem_json() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let path = format!("/api/v1/boards/{}/topics", w.board);

    // not JSON syntax
    w.member
        .raw(
            Method::POST,
            &path,
            Some(b"{not json".to_vec()),
            Some("application/json"),
        )
        .await
        .assert_problem(StatusCode::BAD_REQUEST, "invalid_json");
    // wrong content type
    w.member
        .raw(
            Method::POST,
            &path,
            Some(b"{}".to_vec()),
            Some("text/plain"),
        )
        .await
        .assert_problem(StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_media_type");
    // missing field
    w.member
        .post(&path, json!({"title": "Hello there"}))
        .await
        .assert_field_error("body", "invalid_body");
    // wrong type
    w.member
        .post(&path, json!({"title": 5, "body": "x"}))
        .await
        .assert_field_error("body", "invalid_body");
    // malformed id
    w.member
        .get("/api/v1/topics/not-a-uuid")
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn flood_control_enforces_minimum_intervals_per_account() {
    let mut config = test_config();
    config.flood = FloodConfig {
        min_interval_secs: 5,
        new_account_interval_secs: 30,
        new_account_age_mins: 10,
        duplicate_window_secs: 120,
    };
    let t = TestApp::with_config(config).await;
    let w = t.world().await;
    let topic_path = format!("/api/v1/boards/{}/topics", w.board);

    // A brand-new account (just created) must wait 30 s between posts.
    let topic = w.member.create_topic(w.board, "First topic", "hello").await;
    let reply_path = format!("/api/v1/topics/{topic}/posts");
    w.member
        .post(&reply_path, json!({"body": "too soon"}))
        .await
        .assert_problem(StatusCode::TOO_MANY_REQUESTS, "posting_too_fast");
    w.member
        .post(
            &topic_path,
            json!({"title": "Second topic", "body": "too soon"}),
        )
        .await
        .assert_problem(StatusCode::TOO_MANY_REQUESTS, "posting_too_fast");
    // other accounts are unaffected (the limit is per account)
    w.other
        .post(&reply_path, json!({"body": "I am fine"}))
        .await
        .created();

    // 10 s later a new account still waits; an old account would not.
    sqlx::query("update posts set created_at = now() - interval '10 seconds'")
        .execute(&t.pool)
        .await
        .unwrap();
    w.member
        .post(&reply_path, json!({"body": "still too soon"}))
        .await
        .assert_problem(StatusCode::TOO_MANY_REQUESTS, "posting_too_fast");
    sqlx::query("update users set created_at = now() - interval '1 hour'")
        .execute(&t.pool)
        .await
        .unwrap();
    w.member
        .post(&reply_path, json!({"body": "old enough"}))
        .await
        .created();

    // ...but the 5 s minimum still applies to established accounts
    w.member
        .post(&reply_path, json!({"body": "immediately again"}))
        .await
        .assert_problem(StatusCode::TOO_MANY_REQUESTS, "posting_too_fast");
    sqlx::query("update posts set created_at = now() - interval '6 seconds' where author_id = $1")
        .bind(w.member_id)
        .execute(&t.pool)
        .await
        .unwrap();
    w.member
        .post(&reply_path, json!({"body": "after 6 seconds"}))
        .await
        .created();
}

#[tokio::test]
async fn duplicate_consecutive_posts_are_rejected() {
    let mut config = test_config();
    config.flood = FloodConfig {
        min_interval_secs: 0,
        new_account_interval_secs: 0,
        new_account_age_mins: 0,
        duplicate_window_secs: 120,
    };
    let t = TestApp::with_config(config).await;
    let w = t.world().await;
    let topic = w.member.create_topic(w.board, "Hello there", "first").await;
    let path = format!("/api/v1/topics/{topic}/posts");

    w.member
        .post(&path, json!({"body": "same text"}))
        .await
        .created();
    w.member
        .post(&path, json!({"body": "same text"}))
        .await
        .assert_problem(StatusCode::CONFLICT, "duplicate_post");
    // a different body is fine, and so is another user posting the same text
    w.member
        .post(&path, json!({"body": "different"}))
        .await
        .created();
    w.other
        .post(&path, json!({"body": "different"}))
        .await
        .created();
    // outside the window the same text is allowed again
    sqlx::query("update posts set created_at = now() - interval '3 minutes'")
        .execute(&t.pool)
        .await
        .unwrap();
    w.member
        .post(&path, json!({"body": "different"}))
        .await
        .created();
    assert_eq!(t.scalar_i64("select count(*) from posts").await, 5);
}

#[tokio::test]
async fn concurrent_posting_by_one_account_cannot_bypass_flood_control() {
    let mut config = test_config();
    config.flood = FloodConfig {
        min_interval_secs: 5,
        new_account_interval_secs: 0,
        new_account_age_mins: 0,
        duplicate_window_secs: 0,
    };
    let t = TestApp::with_config(config).await;
    let w = t.world().await;
    let topic = {
        // seed a topic by someone else so the member starts with no posts
        w.other.create_topic(w.board, "Hello there", "first").await
    };
    let path = format!("/api/v1/topics/{topic}/posts");

    let mut tasks = Vec::new();
    for i in 0..8 {
        let client = w.member.clone();
        let path = path.clone();
        tasks.push(tokio::spawn(async move {
            client
                .post(&path, json!({"body": format!("racing {i}")}))
                .await
                .status
        }));
    }
    let mut created = 0;
    for task in tasks {
        if task.await.unwrap() == StatusCode::CREATED {
            created += 1;
        }
    }
    assert_eq!(created, 1, "exactly one of the racing posts may succeed");
}

#[tokio::test]
async fn stats_report_totals_newest_member_latest_post_and_online_users() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w.member.create_topic(w.board, "Hello there", "first").await;
    w.other.reply(topic, "reply").await;
    // opt one user out of the online list
    w.other
        .patch("/api/v1/me/preferences", json!({"showOnline": false}))
        .await
        .no_content();
    // make every session "recently seen"
    sqlx::query("update sessions set last_seen_at = now()")
        .execute(&t.pool)
        .await
        .unwrap();

    let stats = t.guest().get("/api/v1/stats").await;
    stats.ok();
    assert_eq!(stats.body["topics"], 1);
    assert_eq!(stats.body["posts"], 2);
    assert_eq!(stats.body["members"], 4);
    assert_eq!(stats.body["boards"], 3);
    assert_eq!(stats.body["newestMember"]["username"], "other_olga");
    assert_eq!(stats.body["latestPost"]["topicTitle"], "Hello there");
    assert_eq!(stats.body["latestPost"]["author"]["username"], "other_olga");
    assert_eq!(stats.body["onlineCount"], 3);
    let names: Vec<&str> = stats.body["onlineUsers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["username"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["member_max", "mod_mia", "root_admin"]);

    // sessions not seen for more than 5 minutes are not "online"
    sqlx::query("update sessions set last_seen_at = now() - interval '6 minutes'")
        .execute(&t.pool)
        .await
        .unwrap();
    assert_eq!(t.guest().get("/api/v1/stats").await.body["onlineCount"], 0);
}
