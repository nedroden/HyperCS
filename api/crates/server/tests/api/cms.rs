//! Homepage, terms and news.

use axum::http::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth::register_body;
use crate::common::*;

fn homepage() -> Value {
    json!({
        "heroHeading": "Hello world",
        "heroButtonLabel": "Go",
        "features": [
            {"title": "One", "text": "First"},
            {"title": "Two", "text": "Second"},
            {"title": "Three", "text": "Third"}
        ]
    })
}

#[tokio::test]
async fn homepage_has_defaults_and_is_editable_by_admins() {
    let t = TestApp::new().await;
    let w = t.world().await;

    let default = w.guest.get("/api/v1/cms/homepage").await;
    default.ok();
    assert_eq!(default.body["features"].as_array().unwrap().len(), 3);
    // CMS content never mentions the forum
    assert!(!default.body.to_string().to_lowercase().contains("forum"));

    w.admin
        .put("/api/v1/cms/homepage", homepage())
        .await
        .no_content();
    let saved = w.guest.get("/api/v1/cms/homepage").await;
    assert_eq!(saved.body, homepage());

    // replaced wholesale, and text is normalized
    let mut changed = homepage();
    changed["heroHeading"] = json!("  Spaced  ");
    w.admin
        .put("/api/v1/cms/homepage", changed)
        .await
        .no_content();
    assert_eq!(
        w.guest.get("/api/v1/cms/homepage").await.body["heroHeading"],
        "Spaced"
    );
}

#[tokio::test]
async fn homepage_validation_reports_nested_field_paths() {
    let t = TestApp::new().await;
    let w = t.world().await;

    let mut two = homepage();
    two["features"].as_array_mut().unwrap().pop();
    w.admin
        .put("/api/v1/cms/homepage", two)
        .await
        .assert_field_error("features", "length");

    let mut long = homepage();
    long["features"][1]["title"] = json!("t".repeat(61));
    w.admin
        .put("/api/v1/cms/homepage", long)
        .await
        .assert_field_error("features[1].title", "length");

    let mut empty = homepage();
    empty["heroHeading"] = json!("");
    w.admin
        .put("/api/v1/cms/homepage", empty)
        .await
        .assert_field_error("heroHeading", "length");

    let mut bad = homepage();
    bad["heroHeading"] = json!("bell\u{7}");
    w.admin
        .put("/api/v1/cms/homepage", bad)
        .await
        .assert_field_error("heroHeading", "invalid_characters");
}

#[tokio::test]
async fn site_settings_have_defaults_and_are_editable_by_admins() {
    let t = TestApp::new().await;
    let w = t.world().await;

    let default = w.guest.get("/api/v1/cms/site").await;
    default.ok();
    assert_eq!(default.body["siteTitle"], "HyperCS");
    assert_eq!(default.body["defaultLanguage"], "en-US");

    let settings =
        json!({"siteTitle": "  My Club ", "slogan": "Hello there", "defaultLanguage": "de"});
    w.admin.put("/api/v1/cms/site", settings).await.no_content();
    let saved = w.guest.get("/api/v1/cms/site").await;
    assert_eq!(saved.body["siteTitle"], "My Club");
    assert_eq!(saved.body["slogan"], "Hello there");
    assert_eq!(saved.body["defaultLanguage"], "de");

    // new accounts start in the default language
    t.guest()
        .post(
            "/api/v1/auth/register",
            register_body("Newbie", "n@example.com"),
        )
        .await
        .created();
    let (language,): (String,) =
        sqlx::query_as("select language from users where username = 'Newbie'")
            .fetch_one(&t.pool)
            .await
            .unwrap();
    assert_eq!(language, "de");
}

#[tokio::test]
async fn site_settings_slogan_falls_back_to_the_legacy_homepage_intro() {
    let t = TestApp::new().await;
    hypercs_db::cms::put(&t.pool, "homepage", &json!({"heroIntro": "Legacy intro"}))
        .await
        .unwrap();
    let res = t.guest().get("/api/v1/cms/site").await;
    assert_eq!(res.body["slogan"], "Legacy intro");
}

#[tokio::test]
async fn site_settings_validation() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let valid = json!({"siteTitle": "T", "slogan": "S", "defaultLanguage": "en-US"});

    let mut bad = valid.clone();
    bad["defaultLanguage"] = json!("fr");
    w.admin
        .put("/api/v1/cms/site", bad)
        .await
        .assert_field_error("defaultLanguage", "invalid_language");

    let mut bad = valid.clone();
    bad["siteTitle"] = json!("");
    w.admin
        .put("/api/v1/cms/site", bad)
        .await
        .assert_field_error("siteTitle", "length");

    let mut bad = valid;
    bad["slogan"] = json!("bell\u{7}");
    w.admin
        .put("/api/v1/cms/site", bad)
        .await
        .assert_field_error("slogan", "invalid_characters");
}

#[tokio::test]
async fn terms_have_defaults_and_are_editable_by_admins() {
    let t = TestApp::new().await;
    let w = t.world().await;
    assert!(
        w.guest.get("/api/v1/cms/terms").await.body["body"]
            .as_str()
            .unwrap()
            .len()
            > 10
    );

    w.admin
        .put("/api/v1/cms/terms", json!({"body": "Line 1\r\nLine 2"}))
        .await
        .no_content();
    assert_eq!(
        w.guest.get("/api/v1/cms/terms").await.body["body"],
        "Line 1\nLine 2"
    );
    w.admin
        .put("/api/v1/cms/terms", json!({"body": ""}))
        .await
        .assert_field_error("body", "length");
    w.admin
        .put("/api/v1/cms/terms", json!({"body": "x".repeat(20_001)}))
        .await
        .assert_field_error("body", "length");
}

#[tokio::test]
async fn cms_writes_require_an_admin() {
    let t = TestApp::new().await;
    let w = t.world().await;
    for client in [&w.member, &w.moderator] {
        client
            .put("/api/v1/cms/homepage", homepage())
            .await
            .assert_problem(StatusCode::FORBIDDEN, "forbidden");
        client
            .put("/api/v1/cms/terms", json!({"body": "x"}))
            .await
            .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    }
    w.guest
        .put("/api/v1/cms/homepage", homepage())
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    w.guest
        .put("/api/v1/cms/terms", json!({"body": "x"}))
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
}

#[tokio::test]
async fn news_crud_by_id_with_cosmetic_slugs() {
    let t = TestApp::new().await;
    let w = t.world().await;

    let res = w
        .admin
        .post(
            "/api/v1/news",
            json!({"title": "Größere Übung für Müller", "body": "Some **text**\r\nmore"}),
        )
        .await;
    let id = res.created_id();
    assert_eq!(
        res.header("location"),
        Some(format!("/api/v1/news/{id}").as_str())
    );

    let item = w.guest.get(&format!("/api/v1/news/{id}")).await;
    item.ok();
    assert_eq!(item.body["id"], id.to_string());
    assert_eq!(item.body["slug"], "groessere-uebung-fuer-mueller");
    assert_eq!(item.body["body"], "Some **text**\nmore");
    assert_eq!(item.body["author"]["username"], "root_admin");
    assert!(item.body["publishedAt"].is_string());

    // two items may share a title (and so a slug): the id is authoritative
    let twin = w
        .admin
        .post(
            "/api/v1/news",
            json!({"title": "Größere Übung für Müller", "body": ""}),
        )
        .await
        .created_id();
    assert_ne!(twin, id);

    // partial update: body only keeps the title; title change updates the slug
    w.admin
        .patch(&format!("/api/v1/news/{id}"), json!({"body": "new body"}))
        .await
        .no_content();
    let item = w.guest.get(&format!("/api/v1/news/{id}")).await;
    assert_eq!(item.body["body"], "new body");
    assert_eq!(item.body["title"], "Größere Übung für Müller");
    w.admin
        .patch(
            &format!("/api/v1/news/{id}"),
            json!({"title": "Release 2.0"}),
        )
        .await
        .no_content();
    assert_eq!(
        w.guest.get(&format!("/api/v1/news/{id}")).await.body["slug"],
        "release-2-0"
    );

    // list: newest first, paginated
    let list = w.guest.get("/api/v1/news?perPage=1").await;
    list.ok();
    assert_eq!(list.body["items"][0]["id"], twin.to_string());
    assert_eq!(list.body["total"], 2);
    assert_eq!(list.body["totalPages"], 2);
    assert_eq!(list.body["perPage"], 1);

    w.admin
        .delete(&format!("/api/v1/news/{id}"))
        .await
        .no_content();
    w.guest
        .get(&format!("/api/v1/news/{id}"))
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
    w.admin
        .delete(&format!("/api/v1/news/{id}"))
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
    w.admin
        .patch(
            &format!("/api/v1/news/{}", Uuid::now_v7()),
            json!({"title": "x"}),
        )
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn news_validation_and_authorization() {
    let t = TestApp::new().await;
    let w = t.world().await;

    w.admin
        .post("/api/v1/news", json!({"title": "", "body": ""}))
        .await
        .assert_field_error("title", "length");
    w.admin
        .post(
            "/api/v1/news",
            json!({"title": "t".repeat(121), "body": ""}),
        )
        .await
        .assert_field_error("title", "length");
    w.admin
        .post(
            "/api/v1/news",
            json!({"title": "ok", "body": "b".repeat(5_001)}),
        )
        .await
        .assert_field_error("body", "length");
    // a title that cannot be turned into a slug still gets one
    let id = w
        .admin
        .post(
            "/api/v1/news",
            json!({"title": "日本語のニュース", "body": ""}),
        )
        .await
        .created_id();
    assert_eq!(
        w.guest.get(&format!("/api/v1/news/{id}")).await.body["slug"],
        "news"
    );

    let path = format!("/api/v1/news/{id}");
    for client in [&w.member, &w.moderator] {
        client
            .post("/api/v1/news", json!({"title": "x", "body": ""}))
            .await
            .assert_problem(StatusCode::FORBIDDEN, "forbidden");
        client
            .patch(&path, json!({"title": "x"}))
            .await
            .assert_problem(StatusCode::FORBIDDEN, "forbidden");
        client
            .delete(&path)
            .await
            .assert_problem(StatusCode::FORBIDDEN, "forbidden");
    }
    w.guest
        .post("/api/v1/news", json!({"title": "x", "body": ""}))
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    w.guest
        .patch(&path, json!({"title": "x"}))
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
    w.guest
        .delete(&path)
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "unauthorized");
}
