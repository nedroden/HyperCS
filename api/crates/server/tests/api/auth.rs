//! Registration, login, logout and `/me`.

use axum::http::StatusCode;
use hypercs_domain::Role;
use serde_json::json;

use crate::common::*;

pub fn register_body(username: &str, email: &str) -> serde_json::Value {
    json!({"username": username, "email": email, "password": PASSWORD, "acceptTerms": true})
}

#[tokio::test]
async fn register_happy_path_returns_201_location_and_summary() {
    let t = TestApp::new().await;
    let res = t
        .guest()
        .post(
            "/api/v1/auth/register",
            register_body("Alice", "Alice@Example.com"),
        )
        .await;
    res.created();
    assert_eq!(res.header("location"), Some("/api/v1/users/Alice"));
    assert_eq!(res.body["username"], "Alice");
    assert_eq!(res.body["displayName"], "Alice"); // defaults to the username
    assert_eq!(res.body["role"], "member");

    // Email is stored lowercased; the original username casing is kept.
    let (email,): (String,) = sqlx::query_as("select email from users where username = 'Alice'")
        .fetch_one(&t.pool)
        .await
        .unwrap();
    assert_eq!(email, "alice@example.com");
}

#[tokio::test]
async fn register_rejects_duplicates_case_insensitively_with_codes() {
    let t = TestApp::new().await;
    t.create_user("alice", Role::Member).await;
    let guest = t.guest();

    let res = guest
        .post(
            "/api/v1/auth/register",
            register_body("ALICE", "new@example.com"),
        )
        .await;
    res.assert_field_error("username", "username_taken");

    let res = guest
        .post(
            "/api/v1/auth/register",
            register_body("bob", "Alice@example.com"),
        )
        .await;
    res.assert_field_error("email", "email_taken");

    // Both problems are reported together.
    let res = guest
        .post(
            "/api/v1/auth/register",
            register_body("alice", "alice@example.com"),
        )
        .await;
    res.assert_field_error("username", "username_taken");
    res.assert_field_error("email", "email_taken");
}

#[tokio::test]
async fn register_validates_limits_charset_reserved_names_and_terms() {
    let t = TestApp::new().await;
    let guest = t.guest();

    // length limits (from the domain constants)
    guest
        .post(
            "/api/v1/auth/register",
            register_body("ab", "a@example.com"),
        )
        .await
        .assert_field_error("username", "length");
    guest
        .post(
            "/api/v1/auth/register",
            register_body(&"a".repeat(33), "a@example.com"),
        )
        .await
        .assert_field_error("username", "length");
    let short_pw =
        json!({"username":"carol","email":"c@example.com","password":"short","acceptTerms":true});
    guest
        .post("/api/v1/auth/register", short_pw)
        .await
        .assert_field_error("password", "length");
    let long_pw = json!({"username":"carol","email":"c@example.com","password":"x".repeat(129),"acceptTerms":true});
    guest
        .post("/api/v1/auth/register", long_pw)
        .await
        .assert_field_error("password", "length");
    guest
        .post(
            "/api/v1/auth/register",
            register_body("carol", "not-an-email"),
        )
        .await
        .assert_field_error("email", "email");

    // charset and reserved names
    guest
        .post(
            "/api/v1/auth/register",
            register_body("röbert", "r@example.com"),
        )
        .await
        .assert_field_error("username", "invalid_username");
    guest
        .post(
            "/api/v1/auth/register",
            register_body("bad name", "r@example.com"),
        )
        .await
        .assert_field_error("username", "invalid_username");
    for reserved in ["admin", "Root", "MODERATOR", "api", "system"] {
        guest
            .post(
                "/api/v1/auth/register",
                register_body(reserved, "r@example.com"),
            )
            .await
            .assert_field_error("username", "username_taken");
    }

    // terms must be accepted
    let no_terms =
        json!({"username":"dave","email":"d@example.com","password":PASSWORD,"acceptTerms":false});
    guest
        .post("/api/v1/auth/register", no_terms)
        .await
        .assert_field_error("acceptTerms", "terms_not_accepted");
}

#[tokio::test]
async fn login_sets_httponly_samesite_cookie_and_me_works() {
    let t = TestApp::new().await;
    t.create_user("alice", Role::Member).await;
    let guest = t.guest();

    let res = guest
        .post(
            "/api/v1/auth/login",
            json!({"username": "ALICE", "password": PASSWORD, "remember": false}),
        )
        .await;
    res.no_content();
    let cookie = set_cookie_header(&res);
    assert!(cookie.contains("HttpOnly"), "{cookie}");
    assert!(cookie.contains("SameSite=Lax"), "{cookie}");
    assert!(cookie.contains("Path=/"), "{cookie}");
    assert!(
        !cookie.contains("Secure"),
        "Secure only in production: {cookie}"
    );
    assert!(
        !cookie.contains("Max-Age"),
        "browser-session cookie: {cookie}"
    );

    let mut client = guest.clone();
    client.cookie = Some(session_cookie_pair(&res));
    let me = client.get("/api/v1/me").await;
    me.ok();
    assert_eq!(me.body["username"], "alice");
    assert_eq!(me.body["role"], "member");
    assert_eq!(me.body["language"], "en-US");
    assert!(me.body["csrfToken"].as_str().unwrap().len() >= 32);

    // Only a hash of the token is stored.
    let token = client.cookie.as_ref().unwrap().split('=').nth(1).unwrap();
    let stored: (String,) = sqlx::query_as("select token_hash from sessions")
        .fetch_one(&t.pool)
        .await
        .unwrap();
    assert_ne!(stored.0, token);
}

#[tokio::test]
async fn remember_me_issues_a_30_day_persistent_cookie() {
    let t = TestApp::new().await;
    t.create_user("alice", Role::Member).await;
    let res = t
        .guest()
        .post(
            "/api/v1/auth/login",
            json!({"username": "alice", "password": PASSWORD, "remember": true}),
        )
        .await;
    res.no_content();
    assert!(
        set_cookie_header(&res).contains("Max-Age=2592000"),
        "{}",
        set_cookie_header(&res)
    );
}

#[tokio::test]
async fn secure_flag_is_set_when_configured() {
    let mut config = test_config();
    config.cookie_secure = true;
    let t = TestApp::with_config(config).await;
    t.create_user("alice", Role::Member).await;
    let res = t
        .guest()
        .post(
            "/api/v1/auth/login",
            json!({"username": "alice", "password": PASSWORD}),
        )
        .await;
    res.no_content();
    assert!(set_cookie_header(&res).contains("Secure"));
    assert!(res.header("strict-transport-security").is_some());
}

#[tokio::test]
async fn login_errors_do_not_reveal_whether_the_username_exists() {
    let t = TestApp::new().await;
    t.create_user("alice", Role::Member).await;
    let guest = t.guest();

    let wrong_password = guest
        .post(
            "/api/v1/auth/login",
            json!({"username": "alice", "password": "wrong password!"}),
        )
        .await;
    let unknown_user = guest
        .post(
            "/api/v1/auth/login",
            json!({"username": "nobody", "password": "wrong password!"}),
        )
        .await;
    wrong_password.assert_problem(StatusCode::UNAUTHORIZED, "invalid_credentials");
    unknown_user.assert_problem(StatusCode::UNAUTHORIZED, "invalid_credentials");
    assert_eq!(wrong_password.body, unknown_user.body);
    assert!(wrong_password.headers.get("set-cookie").is_none());
}

#[tokio::test]
async fn login_rotates_the_session() {
    let t = TestApp::new().await;
    t.create_user("alice", Role::Member).await;
    let first = t.login("alice").await;

    // Log in again presenting the old cookie: the old session must die.
    let res = first
        .post(
            "/api/v1/auth/login",
            json!({"username": "alice", "password": PASSWORD}),
        )
        .await;
    res.no_content();
    let new_cookie = session_cookie_pair(&res);
    assert_ne!(Some(&new_cookie), first.cookie.as_ref());

    first
        .get("/api/v1/me")
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    let mut second = first.clone();
    second.cookie = Some(new_cookie);
    second.get("/api/v1/me").await.ok();
    assert_eq!(t.scalar_i64("select count(*) from sessions").await, 1);
}

#[tokio::test]
async fn logout_deletes_the_session_and_clears_the_cookie() {
    let t = TestApp::new().await;
    t.create_user("alice", Role::Member).await;
    let alice = t.login("alice").await;

    let res = alice.post_empty("/api/v1/auth/logout").await;
    res.no_content();
    assert!(set_cookie_header(&res).contains("Max-Age=0"));
    assert_eq!(t.scalar_i64("select count(*) from sessions").await, 0);
    alice
        .get("/api/v1/me")
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
}

#[tokio::test]
async fn auth_failures_are_401_for_guests_and_garbage_cookies() {
    let t = TestApp::new().await;
    let guest = t.guest();
    guest
        .get("/api/v1/me")
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    guest
        .post_empty("/api/v1/auth/logout")
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    let mut bad = guest.clone();
    bad.cookie = Some("hypercs_session=not-a-real-token".to_string());
    bad.get("/api/v1/me")
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
}

#[tokio::test]
async fn passwords_are_hashed_with_argon2id() {
    let t = TestApp::new().await;
    t.guest()
        .post(
            "/api/v1/auth/register",
            register_body("alice", "a@example.com"),
        )
        .await
        .created();
    let (hash,): (String,) = sqlx::query_as("select password_hash from users")
        .fetch_one(&t.pool)
        .await
        .unwrap();
    assert!(hash.starts_with("$argon2id$"), "{hash}");
    assert!(!hash.contains(PASSWORD));
}
