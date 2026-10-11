//! CSRF/origin protection, session lifetime, immediate role changes and the
//! active-sessions endpoints.

use axum::http::StatusCode;
use hypercs_domain::Role;
use serde_json::json;
use uuid::Uuid;

use crate::common::*;

#[tokio::test]
async fn csrf_header_is_required_on_every_unsafe_authenticated_request() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let body = json!({"title": "Hello there", "body": "First post"});
    let path = format!("/api/v1/boards/{}/topics", w.board);

    // missing token
    w.member
        .without_csrf()
        .post(&path, body.clone())
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    // wrong token
    w.member
        .with_csrf("not-the-token")
        .post(&path, body.clone())
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    // another session's token
    w.member
        .with_csrf(w.other.csrf.as_ref().unwrap())
        .post(&path, body.clone())
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    // every unsafe method is covered
    w.member
        .without_csrf()
        .patch("/api/v1/me/preferences", json!({"showOnline": false}))
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    w.admin
        .without_csrf()
        .put("/api/v1/cms/terms", json!({"body": "x"}))
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    w.member
        .without_csrf()
        .delete(&format!("/api/v1/me/sessions/{}", Uuid::now_v7()))
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    // nothing was written
    assert_eq!(t.scalar_i64("select count(*) from topics").await, 0);

    // with the right token it works; safe methods never need it
    w.member.post(&path, body).await.created();
    w.member.without_csrf().get("/api/v1/me").await.ok();
}

#[tokio::test]
async fn guests_get_401_before_csrf_is_considered() {
    let t = TestApp::new().await;
    let w = t.world().await;
    w.guest
        .post(
            &format!("/api/v1/boards/{}/topics", w.board),
            json!({"title": "Hello there", "body": "x"}),
        )
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
}

#[tokio::test]
async fn untrusted_origins_are_rejected_for_unsafe_requests() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let path = format!("/api/v1/boards/{}/topics", w.board);
    let body = json!({"title": "Hello there", "body": "First post"});

    for origin in [Some("https://evil.example"), Some("null"), None] {
        w.member
            .with_origin(origin)
            .post(&path, body.clone())
            .await
            .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    }
    // Sec-Fetch-Site: cross-site is rejected even with a matching Origin.
    w.member
        .with_header("sec-fetch-site", "cross-site")
        .post(&path, body.clone())
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    // No Origin but the browser says same-origin: accepted.
    w.member
        .with_origin(None)
        .with_header("sec-fetch-site", "same-origin")
        .post(&path, body)
        .await
        .created();

    // Login/register are protected by the origin check too (no session yet).
    t.guest()
        .with_origin(Some("https://evil.example"))
        .post(
            "/api/v1/auth/login",
            json!({"username": "member_max", "password": PASSWORD}),
        )
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
}

#[tokio::test]
async fn allow_listed_origin_is_trusted_even_when_host_differs() {
    let mut config = test_config();
    config.allowed_origins = vec!["http://localhost:5173".to_string()];
    let t = TestApp::with_config(config).await;
    t.create_user("alice", Role::Member).await;
    // e.g. the Vite dev server proxying with a rewritten Host header
    let spa = t.guest().with_origin(Some("http://localhost:5173"));
    spa.post(
        "/api/v1/auth/login",
        json!({"username": "alice", "password": PASSWORD}),
    )
    .await
    .no_content();
    // an origin that is neither listed nor the host is still refused
    t.guest()
        .with_origin(Some("https://evil.example"))
        .post(
            "/api/v1/auth/login",
            json!({"username": "alice", "password": PASSWORD}),
        )
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
}

#[tokio::test]
async fn idle_sessions_expire_after_24_hours_and_slide_on_activity() {
    let t = TestApp::new().await;
    t.create_user("alice", Role::Member).await;
    let alice = t.login("alice").await;

    // 23h idle: still valid, and activity slides the expiry forward.
    sqlx::query(
        "update sessions set last_seen_at = now() - interval '23 hours', \
         expires_at = now() + interval '1 hour'",
    )
    .execute(&t.pool)
    .await
    .unwrap();
    alice.get("/api/v1/me").await.ok();
    let remaining: (f64,) = sqlx::query_as(
        "select extract(epoch from (expires_at - now()))::float8 / 3600 from sessions",
    )
    .fetch_one(&t.pool)
    .await
    .unwrap();
    assert!(
        remaining.0 > 23.0,
        "expiry slid to ~24h, got {}h",
        remaining.0
    );

    // 25h idle (expiry was 24h after the last activity): expired and purged.
    sqlx::query(
        "update sessions set last_seen_at = now() - interval '25 hours', \
         expires_at = now() - interval '1 hour'",
    )
    .execute(&t.pool)
    .await
    .unwrap();
    alice
        .get("/api/v1/me")
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    assert_eq!(t.scalar_i64("select count(*) from sessions").await, 0);
}

#[tokio::test]
async fn remember_me_sessions_live_30_days_without_sliding() {
    let t = TestApp::new().await;
    t.create_user("alice", Role::Member).await;
    let alice = Client::login(t.app.clone(), "alice", true).await;

    let (days,): (f64,) = sqlx::query_as(
        "select extract(epoch from (expires_at - created_at))::float8 / 86400 from sessions",
    )
    .fetch_one(&t.pool)
    .await
    .unwrap();
    assert!((29.9..=30.1).contains(&days), "{days}");

    // 29 days in, idle for 5 days: still valid (not subject to the 24h idle
    // rule); expiry stays fixed.
    sqlx::query(
        "update sessions set created_at = now() - interval '29 days', \
         last_seen_at = now() - interval '5 days', expires_at = now() + interval '1 day'",
    )
    .execute(&t.pool)
    .await
    .unwrap();
    alice.get("/api/v1/me").await.ok();
    let (left,): (f64,) = sqlx::query_as(
        "select extract(epoch from (expires_at - now()))::float8 / 3600 from sessions",
    )
    .fetch_one(&t.pool)
    .await
    .unwrap();
    assert!(left < 25.0, "remember sessions must not slide, got {left}h");

    // past 30 days: expired
    sqlx::query("update sessions set expires_at = now() - interval '1 minute'")
        .execute(&t.pool)
        .await
        .unwrap();
    alice
        .get("/api/v1/me")
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
}

#[tokio::test]
async fn last_seen_is_updated_at_most_once_a_minute() {
    let t = TestApp::new().await;
    t.create_user("alice", Role::Member).await;
    let alice = t.login("alice").await;
    let fetch = || async {
        let row: (chrono::DateTime<chrono::Utc>,) =
            sqlx::query_as("select last_seen_at from sessions")
                .fetch_one(&t.pool)
                .await
                .unwrap();
        row.0
    };

    // Fresh session: a burst of requests does not touch last_seen_at.
    let initial = fetch().await;
    for _ in 0..3 {
        alice.get("/api/v1/me").await.ok();
    }
    assert_eq!(fetch().await, initial);

    // After a minute of inactivity the next request records it.
    sqlx::query("update sessions set last_seen_at = now() - interval '2 minutes'")
        .execute(&t.pool)
        .await
        .unwrap();
    let stale = fetch().await;
    alice.get("/api/v1/me").await.ok();
    let touched = fetch().await;
    assert!(
        touched > stale + chrono::Duration::seconds(60),
        "{stale} -> {touched}"
    );
    alice.get("/api/v1/me").await.ok();
    assert_eq!(
        fetch().await,
        touched,
        "second request within a minute: no write"
    );
}

#[tokio::test]
async fn role_changes_apply_immediately_to_open_sessions() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let path = format!("/api/v1/categories/{}", w.category);

    // The member's session cannot manage structure...
    w.member
        .patch(&path, json!({"name": "Renamed"}))
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    // ...until an admin promotes them: the same session works right away.
    w.admin
        .patch("/api/v1/users/member_max/role", json!({"role": "admin"}))
        .await
        .no_content();
    w.member
        .patch(&path, json!({"name": "Renamed"}))
        .await
        .no_content();
    assert_eq!(w.member.get("/api/v1/me").await.body["role"], "admin");

    // And demotion takes effect immediately too.
    w.admin
        .patch("/api/v1/users/member_max/role", json!({"role": "member"}))
        .await
        .no_content();
    w.member
        .patch(&path, json!({"name": "Again"}))
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
}

#[tokio::test]
async fn changing_the_password_revokes_all_other_sessions() {
    let t = TestApp::new().await;
    t.create_user("alice", Role::Member).await;
    let laptop = t.login("alice").await;
    let phone = t.login("alice").await;
    assert_eq!(t.scalar_i64("select count(*) from sessions").await, 2);

    // wrong current password
    laptop
        .patch(
            "/api/v1/me/password",
            json!({"currentPassword": "nope nope nope", "newPassword": "a brand new password"}),
        )
        .await
        .assert_field_error("currentPassword", "invalid_password");
    // too short
    laptop
        .patch(
            "/api/v1/me/password",
            json!({"currentPassword": PASSWORD, "newPassword": "short"}),
        )
        .await
        .assert_field_error("newPassword", "length");

    laptop
        .patch(
            "/api/v1/me/password",
            json!({"currentPassword": PASSWORD, "newPassword": "a brand new password"}),
        )
        .await
        .no_content();

    laptop.get("/api/v1/me").await.ok(); // the current session survives
    phone
        .get("/api/v1/me")
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");

    // old password no longer works, new one does
    t.guest()
        .post(
            "/api/v1/auth/login",
            json!({"username": "alice", "password": PASSWORD}),
        )
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "invalid_credentials");
    t.guest()
        .post(
            "/api/v1/auth/login",
            json!({"username": "alice", "password": "a brand new password"}),
        )
        .await
        .no_content();
}

#[tokio::test]
async fn sessions_list_marks_the_current_one_and_revoke_works() {
    let t = TestApp::new().await;
    t.create_user("alice", Role::Member).await;
    t.create_user("bob", Role::Member).await;
    let laptop = t
        .login("alice")
        .await
        .with_header("user-agent", "TestBrowser/1.0");
    let phone = t.login("alice").await;
    let bob = t.login("bob").await;

    let list = laptop.get("/api/v1/me/sessions").await;
    list.ok();
    let sessions = list.body.as_array().unwrap();
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions.iter().filter(|s| s["current"] == true).count(), 1);
    // no IP address is exposed or stored
    assert!(sessions[0].get("ip").is_none());
    let columns: Vec<(String,)> = sqlx::query_as(
        "select column_name::text from information_schema.columns where table_name = 'sessions'",
    )
    .fetch_all(&t.pool)
    .await
    .unwrap();
    assert!(columns.iter().all(|c| c.0 != "ip"));

    let phone_id = {
        let list = phone.get("/api/v1/me/sessions").await;
        list.body
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["current"] == true)
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    };

    // Someone else's session id looks like an unknown one.
    bob.delete(&format!("/api/v1/me/sessions/{phone_id}"))
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
    phone.get("/api/v1/me").await.ok();

    laptop
        .delete(&format!("/api/v1/me/sessions/{phone_id}"))
        .await
        .no_content();
    phone
        .get("/api/v1/me")
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");

    // guests cannot list or revoke
    t.guest()
        .get("/api/v1/me/sessions")
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
}

#[tokio::test]
async fn the_user_agent_is_stored_as_a_device_hint_and_truncated() {
    let t = TestApp::new().await;
    t.create_user("alice", Role::Member).await;
    let long_agent = "A".repeat(400);
    let res = t
        .guest()
        .with_header("user-agent", &long_agent)
        .post(
            "/api/v1/auth/login",
            json!({"username": "alice", "password": PASSWORD}),
        )
        .await;
    res.no_content();
    let (agent,): (Option<String>,) = sqlx::query_as("select user_agent from sessions")
        .fetch_one(&t.pool)
        .await
        .unwrap();
    assert_eq!(agent.unwrap().len(), 255);
}
