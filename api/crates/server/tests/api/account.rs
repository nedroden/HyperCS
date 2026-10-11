//! `/me/*` settings, public profiles and the admin user endpoints.

use axum::http::StatusCode;
use hypercs_domain::Role;
use serde_json::json;

use crate::common::*;

#[tokio::test]
async fn profile_patch_is_partial_and_null_clears() {
    let t = TestApp::new().await;
    let (_, alice) = t.user_with_client("alice", Role::Member).await;

    alice
        .patch(
            "/api/v1/me/profile",
            json!({
                "displayName": "  Alice A.  ",
                "location": "Berlin",
                "website": "https://alice.example",
                "aboutMe": "Hello\r\nworld"
            }),
        )
        .await
        .no_content();
    let me = alice.get("/api/v1/me").await;
    assert_eq!(me.body["displayName"], "Alice A.");
    assert_eq!(me.body["location"], "Berlin");
    assert_eq!(me.body["aboutMe"], "Hello\nworld"); // line endings normalized

    // Only the sent field changes; absent fields are untouched.
    alice
        .patch("/api/v1/me/profile", json!({"location": "Hamburg"}))
        .await
        .no_content();
    let me = alice.get("/api/v1/me").await;
    assert_eq!(me.body["location"], "Hamburg");
    assert_eq!(me.body["displayName"], "Alice A.");
    assert_eq!(me.body["website"], "https://alice.example");

    // null (and "") clear an optional field.
    alice
        .patch(
            "/api/v1/me/profile",
            json!({"website": null, "aboutMe": ""}),
        )
        .await
        .no_content();
    let me = alice.get("/api/v1/me").await;
    assert!(me.body["website"].is_null());
    assert!(me.body["aboutMe"].is_null());
    assert_eq!(me.body["location"], "Hamburg");

    // the username itself is not changeable and the display name is shown in
    // the public profile
    let profile = t.guest().get("/api/v1/users/ALICE").await;
    profile.ok();
    assert_eq!(profile.body["username"], "alice");
    assert_eq!(profile.body["displayName"], "Alice A.");
    assert!(
        profile.body.get("email").is_none(),
        "email must stay private"
    );
}

#[tokio::test]
async fn profile_validation_rules() {
    let t = TestApp::new().await;
    let (_, alice) = t.user_with_client("alice", Role::Member).await;
    let patch = |body| alice.patch("/api/v1/me/profile", body);

    patch(json!({"displayName": ""}))
        .await
        .assert_field_error("displayName", "length");
    patch(json!({"displayName": "x".repeat(51)}))
        .await
        .assert_field_error("displayName", "length");
    patch(json!({"displayName": "Zero\u{200B}Width"}))
        .await
        .assert_field_error("displayName", "invalid_characters");
    patch(json!({"displayName": "Line\nbreak"}))
        .await
        .assert_field_error("displayName", "invalid_characters");
    patch(json!({"displayName": "Bell\u{7}"}))
        .await
        .assert_field_error("displayName", "invalid_characters");
    patch(json!({"website": "javascript:alert(1)"}))
        .await
        .assert_field_error("website", "url");
    patch(json!({"website": "ftp://example.com"}))
        .await
        .assert_field_error("website", "url");
    patch(json!({"website": format!("https://e.com/{}", "a".repeat(200))}))
        .await
        .assert_field_error("website", "length");
    patch(json!({"aboutMe": "a".repeat(501)}))
        .await
        .assert_field_error("aboutMe", "length");
    patch(json!({"location": "l".repeat(101)}))
        .await
        .assert_field_error("location", "length");

    // Unicode (NFC) display names are fine and normalized.
    patch(json!({"displayName": "Jose\u{0301} Ünal 日本"}))
        .await
        .no_content();
    assert_eq!(
        alice.get("/api/v1/me").await.body["displayName"],
        "José Ünal 日本"
    );
}

#[tokio::test]
async fn account_patch_and_email_uniqueness() {
    let t = TestApp::new().await;
    let (_, alice) = t.user_with_client("alice", Role::Member).await;
    t.create_user("bob", Role::Member).await;

    alice
        .patch(
            "/api/v1/me/account",
            json!({"firstName": "Alice", "lastName": "Anderson", "email": "ALICE.NEW@Example.com"}),
        )
        .await
        .no_content();
    let me = alice.get("/api/v1/me").await;
    assert_eq!(me.body["firstName"], "Alice");
    assert_eq!(me.body["email"], "alice.new@example.com");

    alice
        .patch("/api/v1/me/account", json!({"email": "Bob@example.com"}))
        .await
        .assert_field_error("email", "email_taken");
    alice
        .patch("/api/v1/me/account", json!({"email": "nope"}))
        .await
        .assert_field_error("email", "email");
    // keeping your own email is fine
    alice
        .patch(
            "/api/v1/me/account",
            json!({"email": "alice.new@example.com"}),
        )
        .await
        .no_content();
    alice
        .patch("/api/v1/me/account", json!({"firstName": null}))
        .await
        .no_content();
    assert!(alice.get("/api/v1/me").await.body["firstName"].is_null());
    assert_eq!(alice.get("/api/v1/me").await.body["lastName"], "Anderson");
}

#[tokio::test]
async fn preferences_patch_validates_language_and_timezone() {
    let t = TestApp::new().await;
    let (_, alice) = t.user_with_client("alice", Role::Member).await;

    alice
        .patch(
            "/api/v1/me/preferences",
            json!({"language": "de", "timezone": "Europe/Berlin", "showOnline": false}),
        )
        .await
        .no_content();
    let me = alice.get("/api/v1/me").await;
    assert_eq!(me.body["language"], "de");
    assert_eq!(me.body["timezone"], "Europe/Berlin");
    assert_eq!(me.body["showOnline"], false);
    assert_eq!(me.body["notifyOnReply"], true); // untouched

    alice
        .patch("/api/v1/me/preferences", json!({"language": "fr"}))
        .await
        .assert_field_error("language", "invalid_language");
    alice
        .patch(
            "/api/v1/me/preferences",
            json!({"timezone": "Mars/Olympus"}),
        )
        .await
        .assert_field_error("timezone", "invalid_timezone");
    alice
        .patch("/api/v1/me/preferences", json!({"notifyOnQuote": false}))
        .await
        .no_content();
    assert_eq!(alice.get("/api/v1/me").await.body["notifyOnQuote"], false);
}

#[tokio::test]
async fn me_endpoints_reject_guests_and_missing_csrf() {
    let t = TestApp::new().await;
    let (_, alice) = t.user_with_client("alice", Role::Member).await;
    let guest = t.guest();
    for (method, path) in [
        ("PATCH", "/api/v1/me/profile"),
        ("PATCH", "/api/v1/me/account"),
        ("PATCH", "/api/v1/me/password"),
        ("PATCH", "/api/v1/me/preferences"),
    ] {
        let res = guest
            .request(method.parse().unwrap(), path, Some(json!({})))
            .await;
        res.assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
        let res = alice
            .without_csrf()
            .request(method.parse().unwrap(), path, Some(json!({})))
            .await;
        res.assert_problem(StatusCode::FORBIDDEN, "forbidden");
    }
}

#[tokio::test]
async fn public_profile_404s_for_unknown_users() {
    let t = TestApp::new().await;
    t.guest()
        .get("/api/v1/users/nobody")
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn admin_user_list_is_paginated_and_admin_only() {
    let t = TestApp::new().await;
    let w = t.world().await; // 4 users

    let page = w.admin.get("/api/v1/users?perPage=3").await;
    page.ok();
    assert_eq!(page.body["items"].as_array().unwrap().len(), 3);
    assert_eq!(page.body["total"], 4);
    assert_eq!(page.body["totalPages"], 2);
    assert_eq!(page.body["page"], 1);
    assert_eq!(page.body["perPage"], 3);
    assert!(page.body["items"][0]["email"].is_string());
    let page2 = w.admin.get("/api/v1/users?perPage=3&page=2").await;
    assert_eq!(page2.body["items"].as_array().unwrap().len(), 1);

    for client in [&w.moderator, &w.member] {
        client
            .get("/api/v1/users")
            .await
            .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    }
    w.guest
        .get("/api/v1/users")
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    w.admin
        .get("/api/v1/users?page=abc")
        .await
        .assert_problem(StatusCode::BAD_REQUEST, "invalid_query");
}

#[tokio::test]
async fn role_assignment_validation_and_last_admin_protection() {
    let t = TestApp::new().await;
    let w = t.world().await;

    w.admin
        .patch("/api/v1/users/mod_mia/role", json!({"role": "member"}))
        .await
        .no_content();
    let (role,): (String,) = sqlx::query_as("select role from users where username = 'mod_mia'")
        .fetch_one(&t.pool)
        .await
        .unwrap();
    assert_eq!(role, "member");

    w.admin
        .patch("/api/v1/users/mod_mia/role", json!({"role": "root"}))
        .await
        .assert_problem(StatusCode::UNPROCESSABLE_ENTITY, "validation_failed");
    w.admin
        .patch("/api/v1/users/ghost/role", json!({"role": "member"}))
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");

    // The only admin cannot demote themselves; with a second admin they can.
    w.admin
        .patch("/api/v1/users/root_admin/role", json!({"role": "member"}))
        .await
        .assert_problem(StatusCode::CONFLICT, "last_admin");
    w.admin
        .patch("/api/v1/users/other_olga/role", json!({"role": "admin"}))
        .await
        .no_content();
    w.admin
        .patch("/api/v1/users/root_admin/role", json!({"role": "member"}))
        .await
        .no_content();

    // moderators and members cannot assign roles
    w.member
        .patch("/api/v1/users/member_max/role", json!({"role": "admin"}))
        .await
        .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    w.guest
        .patch("/api/v1/users/member_max/role", json!({"role": "admin"}))
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
}
