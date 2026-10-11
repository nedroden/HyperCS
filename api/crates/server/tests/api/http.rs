//! Cross-cutting HTTP behavior: security headers, request ids, logging,
//! CORS, OpenAPI/Swagger exposure and error shapes.

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use crate::common::*;

#[tokio::test]
async fn health_reports_database_status() {
    let t = TestApp::new().await;
    let res = t.guest().get("/api/v1/health").await;
    res.ok();
    assert_eq!(res.body["status"], "ok");
    assert_eq!(res.body["database"], "up");
    assert!(res.body["version"].is_string());
}

#[tokio::test]
async fn every_response_carries_security_headers_and_a_strict_csp() {
    let t = TestApp::new().await;
    let guest = t.guest();
    for res in [
        guest.get("/api/v1/health").await,
        guest.get("/api/v1/me").await,             // 401
        guest.get("/api/v1/does-not-exist").await, // 404
        guest.get("/api/v1/openapi.json").await,
    ] {
        assert_eq!(res.header("x-content-type-options"), Some("nosniff"));
        assert_eq!(res.header("x-frame-options"), Some("DENY"));
        assert_eq!(res.header("referrer-policy"), Some("no-referrer"));
        assert_eq!(res.header("cache-control"), Some("no-store"));
        let csp = res.header("content-security-policy").unwrap();
        assert!(csp.contains("default-src 'none'"), "{csp}");
        assert!(csp.contains("frame-ancestors 'none'"), "{csp}");
        assert!(res.header("permissions-policy").is_some());
        assert!(res.header("x-request-id").is_some());
        assert!(
            res.header("strict-transport-security").is_none(),
            "HSTS only with COOKIE_SECURE"
        );
    }
}

#[tokio::test]
async fn openapi_json_is_always_served_but_swagger_ui_only_when_enabled() {
    let off = TestApp::new().await;
    let spec = off.guest().get("/api/v1/openapi.json").await;
    spec.ok();
    assert_eq!(
        spec.body["openapi"].as_str().map(|v| v.starts_with("3.")),
        Some(true)
    );
    assert!(spec.body["paths"]["/api/v1/topics/{id}"].is_object());
    off.guest()
        .get("/api/v1/docs/")
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");

    let mut config = test_config();
    config.docs_enabled = true;
    let on = TestApp::with_config(config).await;
    on.guest().get("/api/v1/openapi.json").await.ok();
    let page = on.guest().get("/api/v1/docs/").await;
    page.ok();
    // Swagger UI gets a relaxed CSP so its own scripts/styles work, but it is
    // still locked down (own origin only, no framing).
    let csp = page.header("content-security-policy").unwrap();
    assert!(csp.contains("script-src 'self'"), "{csp}");
    assert!(csp.contains("style-src 'self' 'unsafe-inline'"), "{csp}");
    assert!(csp.contains("frame-ancestors 'none'"), "{csp}");
    assert!(!csp.contains("script-src 'self' 'unsafe-inline'"));
    assert!(!csp.contains("unsafe-eval"));
    assert_eq!(page.header("x-content-type-options"), Some("nosniff"));
    // The page itself must not rely on inline scripts (they would be blocked).
    let html = page.body.as_str().unwrap_or_default().to_string();
    assert!(
        !html.contains("<script>") && !html.contains("<script type"),
        "inline <script> would be blocked by the CSP"
    );
    // ...and its JS bundle is served from the same origin.
    let bundle = on.guest().get("/api/v1/docs/swagger-ui-bundle.js").await;
    bundle.ok();
    assert_eq!(bundle.header("x-content-type-options"), Some("nosniff"));
    assert!(
        bundle
            .header("content-security-policy")
            .unwrap()
            .contains("script-src 'self'")
    );
}

#[tokio::test]
async fn request_ids_are_generated_and_propagated() {
    let t = TestApp::new().await;
    let guest = t.guest();

    let generated = guest.get("/api/v1/health").await;
    let id = generated.header("x-request-id").unwrap().to_string();
    assert!(uuid::Uuid::parse_str(&id).is_ok(), "{id}");
    let other = guest.get("/api/v1/health").await;
    assert_ne!(other.header("x-request-id").unwrap(), id);

    let propagated = guest
        .with_header("x-request-id", "client-id_123.v2")
        .get("/api/v1/health")
        .await;
    assert_eq!(propagated.header("x-request-id"), Some("client-id_123.v2"));

    // unsafe values are replaced rather than echoed
    let replaced = guest
        .with_header("x-request-id", &"x".repeat(200))
        .get("/api/v1/health")
        .await;
    assert_ne!(replaced.header("x-request-id").unwrap().len(), 200);
}

#[tokio::test]
async fn each_request_logs_one_json_line_with_context_and_no_secrets() {
    let t = TestApp::new().await;
    let (user_id, alice) = t
        .user_with_client("alice", hypercs_domain::Role::Member)
        .await;
    let me_id = format!("log-me-{}", uuid::Uuid::now_v7());
    let login_id = format!("log-login-{}", uuid::Uuid::now_v7());

    let res = alice
        .with_header("x-request-id", &me_id)
        .get("/api/v1/me")
        .await;
    res.ok();
    t.guest()
        .with_header("x-request-id", &login_id)
        .post(
            "/api/v1/auth/login",
            json!({"username": "alice", "password": "a very secret wrong password"}),
        )
        .await
        .assert_problem(StatusCode::UNAUTHORIZED, "invalid_credentials");

    let completed = |id: &str| -> Vec<Value> {
        log_lines_for(id)
            .into_iter()
            .filter(|v| v["fields"]["message"] == "request completed")
            .collect()
    };
    let me = completed(&me_id);
    assert_eq!(me.len(), 1, "one completion line per request");
    let fields = &me[0]["fields"];
    assert_eq!(fields["method"], "GET");
    assert_eq!(fields["path"], "/api/v1/me");
    assert_eq!(fields["route"], "/api/v1/me");
    assert_eq!(fields["status"], 200);
    assert!(fields["latency_ms"].is_u64());
    assert_eq!(fields["user_id"], user_id.to_string());
    // one span per request carries the request id for any nested log events
    assert_eq!(me[0]["span"]["request_id"], me_id.as_str());

    let login = completed(&login_id);
    assert_eq!(login.len(), 1);
    assert_eq!(login[0]["fields"]["status"], 401);
    assert!(
        login[0]["fields"]["user_id"].is_null(),
        "guests have no user id"
    );

    // never secrets or bodies, anywhere in the output
    let text = all_logs();
    for secret in ["a very secret wrong password", "hypercs_session", PASSWORD] {
        assert!(!text.contains(secret), "log leaked {secret:?}");
    }
    if let Some(csrf) = alice.csrf.as_deref() {
        assert!(!text.contains(csrf));
    }
}

#[tokio::test]
async fn unknown_routes_and_methods_use_problem_json() {
    let t = TestApp::new().await;
    let guest = t.guest();
    guest
        .get("/api/v1/nope")
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
    guest
        .get("/elsewhere")
        .await
        .assert_problem(StatusCode::NOT_FOUND, "not_found");
    guest
        .request(Method::DELETE, "/api/v1/health", None)
        .await
        .assert_problem(StatusCode::METHOD_NOT_ALLOWED, "method_not_allowed");
}

#[tokio::test]
async fn cors_is_closed_by_default_and_opens_only_for_the_allow_list() {
    let preflight = |origin: &'static str| {
        move |t: &TestApp| {
            let client = t
                .guest()
                .with_origin(Some(origin))
                .with_header("access-control-request-method", "POST")
                .with_header(
                    "access-control-request-headers",
                    "content-type,x-csrf-token",
                );
            async move {
                client
                    .request(Method::OPTIONS, "/api/v1/auth/login", None)
                    .await
            }
        }
    };

    // empty allow-list: no CORS headers at all, for any origin
    let closed = TestApp::new().await;
    let res = preflight("https://evil.example")(&closed).await;
    assert!(res.header("access-control-allow-origin").is_none());
    let get = closed
        .guest()
        .with_origin(Some("https://evil.example"))
        .get("/api/v1/health")
        .await;
    assert!(get.header("access-control-allow-origin").is_none());

    // explicit allow-list: only listed origins are echoed, with credentials
    let mut config = test_config();
    config.allowed_origins = vec!["https://app.example".to_string()];
    let open = TestApp::with_config(config).await;
    let res = preflight("https://app.example")(&open).await;
    assert_eq!(
        res.header("access-control-allow-origin"),
        Some("https://app.example")
    );
    assert_eq!(res.header("access-control-allow-credentials"), Some("true"));
    assert!(
        res.header("access-control-allow-headers")
            .unwrap()
            .contains("x-csrf-token")
    );
    let res = preflight("https://evil.example")(&open).await;
    assert!(res.header("access-control-allow-origin").is_none());
}

#[tokio::test]
async fn validation_errors_have_the_documented_problem_shape() {
    let t = TestApp::new().await;
    let res = t
        .guest()
        .post(
            "/api/v1/auth/register",
            json!({"username": "a", "email": "nope", "password": "short", "acceptTerms": false}),
        )
        .await;
    res.assert_problem(StatusCode::UNPROCESSABLE_ENTITY, "validation_failed");
    assert_eq!(res.body["type"], "about:blank");
    assert_eq!(res.body["title"], "Validation failed");
    assert!(res.body["detail"].is_string());
    let errors = res.body["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 4);
    for error in errors {
        assert!(
            error["field"].is_string() && error["code"].is_string() && error["message"].is_string()
        );
    }
    // field names are camelCase, the same as the request body
    assert!(errors.iter().any(|e| e["field"] == "acceptTerms"));
    // non-validation problems have no `errors`
    let not_found = t.guest().get("/api/v1/nope").await;
    assert!(not_found.body.get("errors").is_none());
}
