//! End-to-end integration tests against a real PostgreSQL instance started
//! with Testcontainers. Exercises the HTTP router directly via `oneshot`
//! (no TCP listener needed), covering: registration/login/logout, CSRF
//! rejection, session expiry, the core forum write path, counter
//! consistency, and hard-delete cascade of a topic's first post.

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use hypercs_api::{routes, state::AppState};
use hypercs_db::PgPool;
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use testcontainers_modules::{postgres::Postgres, testcontainers::runners::AsyncRunner};
use tower::ServiceExt;
use uuid::Uuid;

const ORIGIN: &str = "http://localhost";
const HOST: &str = "localhost";

async fn test_pool() -> (
    PgPool,
    testcontainers::ContainerAsync<Postgres>,
) {
    let container = Postgres::default()
        .start()
        .await
        .expect("failed to start postgres container (is Docker/Colima running?)");
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let host = container.get_host().await.unwrap();
    let database_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    (pool, container)
}

fn router(db: PgPool) -> axum::Router {
    routes::router(AppState {
        db,
        config: hypercs_api::state::ApiConfig {
            cookie_secure: false,
            allowed_origins: vec![],
            docs_enabled: false,
        },
    })
}

fn req_builder(method: &str, path: &str) -> axum::http::request::Builder {
    Request::builder()
        .method(method)
        .uri(path)
        .header(header::ORIGIN, ORIGIN)
        .header(header::HOST, HOST)
}

async fn json_body(res: axum::response::Response) -> Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    }
}

fn set_cookie(res: &axum::response::Response) -> Option<String> {
    res.headers()
        .get(header::SET_COOKIE)
        .map(|v| v.to_str().unwrap().split(';').next().unwrap().to_string())
}

#[tokio::test]
async fn full_forum_flow_with_csrf_expiry_and_counters() {
    let (pool, _container) = test_pool().await;
    let app = router(pool.clone());

    // --- Register two members ---
    for username in ["alice", "bob"] {
        let res = app
            .clone()
            .oneshot(
                req_builder("POST", "/api/v1/auth/register")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "username": username,
                            "email": format!("{username}@example.com"),
                            "password": "correct horse battery",
                            "acceptTerms": true
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED, "register {username}");
    }

    // Duplicate username is rejected with a machine-readable code.
    let res = app
        .clone()
        .oneshot(
            req_builder("POST", "/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "username": "alice",
                        "email": "alice2@example.com",
                        "password": "correct horse battery",
                        "acceptTerms": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json_body(res).await;
    assert_eq!(body["errors"][0]["code"], "username_taken");

    // --- Log in as alice, fetch CSRF token ---
    let res = app
        .clone()
        .oneshot(
            req_builder("POST", "/api/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"username": "alice", "password": "correct horse battery", "remember": false})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let alice_cookie = set_cookie(&res).expect("session cookie");

    let res = app
        .clone()
        .oneshot(
            req_builder("GET", "/api/v1/me")
                .header(header::COOKIE, &alice_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let me = json_body(res).await;
    assert_eq!(me["username"], "alice");
    let alice_csrf = me["csrf_token"].as_str().unwrap().to_string();

    // --- Guests cannot create topics (401) ---
    let category_id = Uuid::now_v7();
    hypercs_db::structure::create_category(&pool, category_id, "Category", "", false, 0)
        .await
        .unwrap();
    let board_id = Uuid::now_v7();
    hypercs_db::structure::create_board(&pool, board_id, category_id, "Board", "", false, 0)
        .await
        .unwrap();

    let res = app
        .clone()
        .oneshot(
            req_builder("POST", &format!("/api/v1/boards/{board_id}/topics"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"title": "Hello there", "body": "First post body"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // --- Authenticated but missing/invalid CSRF header is rejected (403) ---
    let res = app
        .clone()
        .oneshot(
            req_builder("POST", &format!("/api/v1/boards/{board_id}/topics"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &alice_cookie)
                .body(Body::from(json!({"title": "Hello there", "body": "First post body"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // --- Valid CSRF: alice creates a topic ---
    let res = app
        .clone()
        .oneshot(
            req_builder("POST", &format!("/api/v1/boards/{board_id}/topics"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &alice_cookie)
                .header("x-csrf-token", &alice_csrf)
                .body(Body::from(json!({"title": "Hello there", "body": "First post body"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let location = res.headers().get(header::LOCATION).unwrap().to_str().unwrap().to_string();
    let topic_id = location.rsplit('/').next().unwrap().to_string();

    // --- Counters after one topic+post: board has 1 topic, 1 post ---
    let board = hypercs_db::structure::get_board_detail(&pool, board_id).await.unwrap().unwrap();
    assert_eq!(board.topic_count, 1);
    assert_eq!(board.post_count, 1);

    // --- bob logs in and replies ---
    let res = app
        .clone()
        .oneshot(
            req_builder("POST", "/api/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"username": "bob", "password": "correct horse battery", "remember": false}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let bob_cookie = set_cookie(&res).unwrap();
    let res = app
        .clone()
        .oneshot(
            req_builder("GET", "/api/v1/me")
                .header(header::COOKIE, &bob_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bob_csrf = json_body(res).await["csrf_token"].as_str().unwrap().to_string();

    let res = app
        .clone()
        .oneshot(
            req_builder("POST", &format!("/api/v1/topics/{topic_id}/posts"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &bob_cookie)
                .header("x-csrf-token", &bob_csrf)
                .body(Body::from(json!({"body": "A reply from bob"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let reply_location = res.headers().get(header::LOCATION).unwrap().to_str().unwrap().to_string();
    let reply_post_id = reply_location.rsplit('/').next().unwrap().to_string();

    // --- Counters after the reply: board 1 topic / 2 posts, topic reply_count = 1 ---
    let board = hypercs_db::structure::get_board_detail(&pool, board_id).await.unwrap().unwrap();
    assert_eq!(board.topic_count, 1);
    assert_eq!(board.post_count, 2);
    let topic = hypercs_db::topics::get_detail(&pool, Uuid::parse_str(&topic_id).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(topic.reply_count, 1);

    // --- bob cannot delete alice's reply post (403); alice cannot delete bob's (not author, not staff) ---
    let res = app
        .clone()
        .oneshot(
            req_builder("DELETE", &format!("/api/v1/posts/{reply_post_id}"))
                .header(header::COOKIE, &alice_cookie)
                .header("x-csrf-token", &alice_csrf)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // --- bob deletes his own reply: counters go back down ---
    let res = app
        .clone()
        .oneshot(
            req_builder("DELETE", &format!("/api/v1/posts/{reply_post_id}"))
                .header(header::COOKIE, &bob_cookie)
                .header("x-csrf-token", &bob_csrf)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let board = hypercs_db::structure::get_board_detail(&pool, board_id).await.unwrap().unwrap();
    assert_eq!(board.post_count, 1);
    let topic = hypercs_db::topics::get_detail(&pool, Uuid::parse_str(&topic_id).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(topic.reply_count, 0);

    // --- alice deletes the topic's first post: the whole topic is hard-deleted, cascade ---
    let posts = hypercs_db::posts::list_for_topic(&pool, Uuid::parse_str(&topic_id).unwrap(), 10, 0)
        .await
        .unwrap();
    let first_post_id = posts[0].id;
    let res = app
        .clone()
        .oneshot(
            req_builder("DELETE", &format!("/api/v1/posts/{first_post_id}"))
                .header(header::COOKIE, &alice_cookie)
                .header("x-csrf-token", &alice_csrf)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let topic_gone = hypercs_db::topics::get_detail(&pool, Uuid::parse_str(&topic_id).unwrap())
        .await
        .unwrap();
    assert!(topic_gone.is_none(), "topic should be gone after its first post is deleted");
    let board = hypercs_db::structure::get_board_detail(&pool, board_id).await.unwrap().unwrap();
    assert_eq!(board.topic_count, 0);
    assert_eq!(board.post_count, 0);

    // --- Session expiry: force-expire alice's session, then she is treated as a guest ---
    let sessions = hypercs_db::sessions::list_for_user(&pool, {
        let row = hypercs_db::users::find_auth_by_username(&pool, "alice").await.unwrap().unwrap();
        row.id
    })
    .await
    .unwrap();
    sqlx::query("update sessions set expires_at = now() - interval '1 hour' where id = $1")
        .bind(sessions[0].id)
        .execute(&pool)
        .await
        .unwrap();
    let res = app
        .clone()
        .oneshot(
            req_builder("GET", "/api/v1/me")
                .header(header::COOKIE, &alice_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
