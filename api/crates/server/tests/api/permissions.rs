//! The role permission matrix, driven end to end through the HTTP API, plus a
//! sweep proving every protected endpoint rejects guests and, where the
//! endpoint is role-restricted, under-privileged roles.

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::common::*;

#[derive(Clone, Copy, Debug)]
enum Action {
    CreateTopicPublic,
    CreateTopicAdminBoard,
    ReplyOpenTopic,
    ReplyLockedTopic,
    EditOwnPost,
    EditForeignPost,
    DeleteOwnPost,
    DeleteForeignPost,
    ViewForeignHistory,
    PinTopic,
    LockTopic,
    MoveTopic,
    DeleteTopic,
    ManageCategories,
    ManageBoards,
    EditHomepage,
    EditTerms,
    EditSite,
    PublishNews,
    ListUsers,
    AssignRole,
}

use Action::*;

const OK: u16 = 200;
const CREATED: u16 = 201;
const NO_CONTENT: u16 = 204;
const UNAUTH: u16 = 401;
const FORBIDDEN: u16 = 403;
const NOT_FOUND: u16 = 404;

/// Expected status per role: [guest, member, moderator, admin].
const MATRIX: &[(Action, [u16; 4])] = &[
    (CreateTopicPublic, [UNAUTH, CREATED, CREATED, CREATED]),
    // admin-only boards do not exist for anyone but admins
    (
        CreateTopicAdminBoard,
        [UNAUTH, NOT_FOUND, NOT_FOUND, CREATED],
    ),
    (ReplyOpenTopic, [UNAUTH, CREATED, CREATED, CREATED]),
    (ReplyLockedTopic, [UNAUTH, FORBIDDEN, CREATED, CREATED]),
    // own post: everyone logged in
    (EditOwnPost, [UNAUTH, NO_CONTENT, NO_CONTENT, NO_CONTENT]),
    (EditForeignPost, [UNAUTH, FORBIDDEN, NO_CONTENT, NO_CONTENT]),
    (DeleteOwnPost, [UNAUTH, NO_CONTENT, NO_CONTENT, NO_CONTENT]),
    (
        DeleteForeignPost,
        [UNAUTH, FORBIDDEN, NO_CONTENT, NO_CONTENT],
    ),
    (ViewForeignHistory, [UNAUTH, FORBIDDEN, OK, OK]),
    (PinTopic, [UNAUTH, FORBIDDEN, NO_CONTENT, NO_CONTENT]),
    (LockTopic, [UNAUTH, FORBIDDEN, NO_CONTENT, NO_CONTENT]),
    (MoveTopic, [UNAUTH, FORBIDDEN, NO_CONTENT, NO_CONTENT]),
    (DeleteTopic, [UNAUTH, FORBIDDEN, NO_CONTENT, NO_CONTENT]),
    (ManageCategories, [UNAUTH, FORBIDDEN, FORBIDDEN, CREATED]),
    (ManageBoards, [UNAUTH, FORBIDDEN, FORBIDDEN, CREATED]),
    (EditHomepage, [UNAUTH, FORBIDDEN, FORBIDDEN, NO_CONTENT]),
    (EditTerms, [UNAUTH, FORBIDDEN, FORBIDDEN, NO_CONTENT]),
    (EditSite, [UNAUTH, FORBIDDEN, FORBIDDEN, NO_CONTENT]),
    (PublishNews, [UNAUTH, FORBIDDEN, FORBIDDEN, CREATED]),
    (ListUsers, [UNAUTH, FORBIDDEN, FORBIDDEN, OK]),
    (AssignRole, [UNAUTH, FORBIDDEN, FORBIDDEN, NO_CONTENT]),
];

async fn perform(t: &TestApp, w: &World, action: Action, role: usize) -> StatusCode {
    let actor: &Client = [&w.guest, &w.member, &w.moderator, &w.admin][role];
    let actor_id = [Uuid::nil(), w.member_id, w.moderator_id, w.admin_id][role];
    // Fresh fixtures per cell so destructive actions do not interfere. Content
    // by "someone else" is authored by the fixed user `other`.
    let foreign_topic = w
        .other
        .create_topic(w.board, "Foreign topic", &Uuid::now_v7().to_string())
        .await;
    let foreign_reply = w
        .other
        .reply(foreign_topic, &Uuid::now_v7().to_string())
        .await;
    let _ = t;

    let res = match action {
        CreateTopicPublic => {
            actor
                .post(
                    &format!("/api/v1/boards/{}/topics", w.board),
                    json!({"title": "Matrix topic", "body": Uuid::now_v7().to_string()}),
                )
                .await
        }
        CreateTopicAdminBoard => {
            actor
                .post(
                    &format!("/api/v1/boards/{}/topics", w.admin_board),
                    json!({"title": "Matrix topic", "body": Uuid::now_v7().to_string()}),
                )
                .await
        }
        ReplyOpenTopic => {
            actor
                .post(
                    &format!("/api/v1/topics/{foreign_topic}/posts"),
                    json!({"body": Uuid::now_v7().to_string()}),
                )
                .await
        }
        ReplyLockedTopic => {
            w.admin
                .patch(&format!("/api/v1/topics/{foreign_topic}"), json!({"locked": true}))
                .await
                .no_content();
            actor
                .post(
                    &format!("/api/v1/topics/{foreign_topic}/posts"),
                    json!({"body": Uuid::now_v7().to_string()}),
                )
                .await
        }
        EditOwnPost | DeleteOwnPost => {
            // a post authored by the acting user (guests have none: use a
            // random id, the 401 comes first anyway)
            let own = if role == 0 {
                Uuid::now_v7()
            } else {
                actor.reply(foreign_topic, &Uuid::now_v7().to_string()).await
            };
            let _ = actor_id;
            match action {
                EditOwnPost => {
                    actor
                        .patch(&format!("/api/v1/posts/{own}"), json!({"body": "edited"}))
                        .await
                }
                _ => actor.delete(&format!("/api/v1/posts/{own}")).await,
            }
        }
        EditForeignPost => {
            actor
                .patch(&format!("/api/v1/posts/{foreign_reply}"), json!({"body": "edited"}))
                .await
        }
        DeleteForeignPost => actor.delete(&format!("/api/v1/posts/{foreign_reply}")).await,
        ViewForeignHistory => {
            w.other
                .patch(&format!("/api/v1/posts/{foreign_reply}"), json!({"body": "v2"}))
                .await
                .no_content();
            actor.get(&format!("/api/v1/posts/{foreign_reply}/revisions")).await
        }
        PinTopic => {
            actor
                .patch(&format!("/api/v1/topics/{foreign_topic}"), json!({"pinned": true}))
                .await
        }
        LockTopic => {
            actor
                .patch(&format!("/api/v1/topics/{foreign_topic}"), json!({"locked": true}))
                .await
        }
        MoveTopic => {
            actor
                .patch(
                    &format!("/api/v1/topics/{foreign_topic}"),
                    json!({"boardId": w.board2}),
                )
                .await
        }
        DeleteTopic => actor.delete(&format!("/api/v1/topics/{foreign_topic}")).await,
        ManageCategories => {
            actor
                .post("/api/v1/categories", json!({"name": format!("Matrix {}", Uuid::now_v7())}))
                .await
        }
        ManageBoards => {
            actor
                .post(
                    "/api/v1/boards",
                    json!({"categoryId": w.category, "name": format!("Matrix {}", Uuid::now_v7())}),
                )
                .await
        }
        EditHomepage => {
            actor
                .put(
                    "/api/v1/cms/homepage",
                    json!({"heroHeading": "h", "heroButtonLabel": "b",
                           "features": [{"title": "a", "text": "a"}, {"title": "b", "text": "b"}, {"title": "c", "text": "c"}]}),
                )
                .await
        }
        EditTerms => actor.put("/api/v1/cms/terms", json!({"body": "terms"})).await,
        EditSite => {
            actor
                .put(
                    "/api/v1/cms/site",
                    json!({"siteTitle": "t", "slogan": "s", "defaultLanguage": "de"}),
                )
                .await
        }
        PublishNews => actor.post("/api/v1/news", json!({"title": "News", "body": ""})).await,
        ListUsers => actor.get("/api/v1/users").await,
        AssignRole => {
            actor
                .patch("/api/v1/users/other_olga/role", json!({"role": "member"}))
                .await
        }
    };
    res.status
}

#[tokio::test]
async fn role_permission_matrix() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let roles = ["guest", "member", "moderator", "admin"];
    let mut failures = Vec::new();
    for (action, expected) in MATRIX {
        for (role, expected) in expected.iter().enumerate() {
            let got = perform(&t, &w, *action, role).await.as_u16();
            if got != *expected {
                failures.push(format!(
                    "{action:?} as {}: expected {expected}, got {got}",
                    roles[role]
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "permission matrix mismatches:\n{}",
        failures.join("\n")
    );
}

/// Every endpoint that needs a session: (method, path template, body).
fn authenticated_endpoints(w: &World) -> Vec<(Method, String, Option<Value>)> {
    let id = Uuid::now_v7();
    let (c, b) = (w.category, w.board);
    vec![
        (Method::POST, "/api/v1/auth/logout".into(), None),
        (Method::GET, "/api/v1/me".into(), None),
        (Method::PATCH, "/api/v1/me/profile".into(), Some(json!({}))),
        (Method::PATCH, "/api/v1/me/account".into(), Some(json!({}))),
        (
            Method::PATCH,
            "/api/v1/me/password".into(),
            Some(json!({"currentPassword": "x", "newPassword": "long enough password"})),
        ),
        (
            Method::PATCH,
            "/api/v1/me/preferences".into(),
            Some(json!({})),
        ),
        (Method::GET, "/api/v1/me/sessions".into(), None),
        (Method::DELETE, format!("/api/v1/me/sessions/{id}"), None),
        (Method::GET, "/api/v1/users".into(), None),
        (
            Method::PATCH,
            "/api/v1/users/other_olga/role".into(),
            Some(json!({"role": "member"})),
        ),
        (
            Method::POST,
            "/api/v1/categories".into(),
            Some(json!({"name": "x"})),
        ),
        (
            Method::PATCH,
            format!("/api/v1/categories/{c}"),
            Some(json!({})),
        ),
        (Method::DELETE, format!("/api/v1/categories/{c}"), None),
        (
            Method::POST,
            format!("/api/v1/categories/{c}/reorder"),
            Some(json!({"direction": "up"})),
        ),
        (
            Method::POST,
            "/api/v1/boards".into(),
            Some(json!({"categoryId": c, "name": "x"})),
        ),
        (
            Method::PATCH,
            format!("/api/v1/boards/{b}"),
            Some(json!({})),
        ),
        (Method::DELETE, format!("/api/v1/boards/{b}"), None),
        (
            Method::POST,
            format!("/api/v1/boards/{b}/reorder"),
            Some(json!({"direction": "up"})),
        ),
        (
            Method::POST,
            format!("/api/v1/boards/{b}/topics"),
            Some(json!({"title": "abc", "body": "x"})),
        ),
        (
            Method::PATCH,
            format!("/api/v1/topics/{id}"),
            Some(json!({})),
        ),
        (Method::DELETE, format!("/api/v1/topics/{id}"), None),
        (
            Method::POST,
            format!("/api/v1/topics/{id}/posts"),
            Some(json!({"body": "x"})),
        ),
        (
            Method::PATCH,
            format!("/api/v1/posts/{id}"),
            Some(json!({"body": "x"})),
        ),
        (Method::DELETE, format!("/api/v1/posts/{id}"), None),
        (Method::GET, format!("/api/v1/posts/{id}/revisions"), None),
        (
            Method::POST,
            format!("/api/v1/posts/{id}/revisions/{id}/restore"),
            None,
        ),
        (
            Method::POST,
            "/api/v1/news".into(),
            Some(json!({"title": "x", "body": ""})),
        ),
        (Method::PATCH, format!("/api/v1/news/{id}"), Some(json!({}))),
        (Method::DELETE, format!("/api/v1/news/{id}"), None),
        (Method::PUT, "/api/v1/cms/homepage".into(), Some(json!({}))),
        (
            Method::PUT,
            "/api/v1/cms/terms".into(),
            Some(json!({"body": "x"})),
        ),
    ]
}

#[tokio::test]
async fn every_authenticated_endpoint_rejects_guests_with_401() {
    let t = TestApp::new().await;
    let w = t.world().await;
    for (method, path, body) in authenticated_endpoints(&w) {
        let res = w.guest.request(method.clone(), &path, body).await;
        assert_eq!(
            (res.status, res.code()),
            (StatusCode::UNAUTHORIZED, "unauthorized"),
            "{method} {path} must reject guests; body: {}",
            res.body
        );
    }
}

#[tokio::test]
async fn every_unsafe_endpoint_requires_the_csrf_token() {
    let t = TestApp::new().await;
    let w = t.world().await;
    for (method, path, body) in authenticated_endpoints(&w) {
        if method == Method::GET {
            continue;
        }
        // Even a fully authorized admin is refused without the CSRF token.
        let res = w
            .admin
            .without_csrf()
            .request(method.clone(), &path, body)
            .await;
        assert_eq!(
            (res.status, res.code()),
            (StatusCode::FORBIDDEN, "forbidden"),
            "{method} {path} must require X-CSRF-Token; body: {}",
            res.body
        );
    }
}

#[tokio::test]
async fn public_endpoints_work_for_guests() {
    let t = TestApp::new().await;
    let w = t.world().await;
    let topic = w.member.create_topic(w.board, "Hello there", "first").await;
    for path in [
        "/api/v1/health".to_string(),
        "/api/v1/stats".to_string(),
        "/api/v1/categories".to_string(),
        format!("/api/v1/boards/{}", w.board),
        format!("/api/v1/topics/{topic}"),
        "/api/v1/users/member_max".to_string(),
        "/api/v1/news".to_string(),
        "/api/v1/cms/homepage".to_string(),
        "/api/v1/cms/terms".to_string(),
        "/api/v1/openapi.json".to_string(),
    ] {
        let res = w.guest.get(&path).await;
        assert_eq!(res.status, StatusCode::OK, "{path}: {}", res.body);
    }
}
