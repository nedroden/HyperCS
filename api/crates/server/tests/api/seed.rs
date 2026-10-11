//! First-start bootstrap: admin account, sample structure and migrations.

use hypercs_api::state::FloodConfig;
use hypercs_server::{config::Config, run_migrations, seed};
use serde_json::json;

use crate::common::*;

fn config(admin: Option<(&str, &str, &str)>) -> Config {
    Config {
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: String::new(),
        cookie_secure: false,
        allowed_origins: vec![],
        docs_enabled: false,
        run_migrations: true,
        json_logs: false,
        flood: FloodConfig::disabled(),
        admin_username: admin.map(|a| a.0.to_string()),
        admin_email: admin.map(|a| a.1.to_string()),
        admin_password: admin.map(|a| a.2.to_string()),
    }
}

#[tokio::test]
async fn admin_bootstrap_is_idempotent_and_allows_the_reserved_name_admin() {
    let t = TestApp::new().await;
    let cfg = config(Some(("admin", "Admin@Example.com", "change-me-now")));

    assert!(
        seed::bootstrap_admin(&t.pool, &cfg).await.unwrap(),
        "first run creates"
    );
    assert!(
        !seed::bootstrap_admin(&t.pool, &cfg).await.unwrap(),
        "second run is a no-op"
    );
    assert_eq!(t.count("users").await, 1);

    let (role, email): (String, String) = sqlx::query_as("select role, email from users")
        .fetch_one(&t.pool)
        .await
        .unwrap();
    assert_eq!(
        (role.as_str(), email.as_str()),
        ("admin", "admin@example.com")
    );

    // The bootstrapped admin can log in with the configured password.
    let client = Client::new(t.app.clone());
    client
        .post(
            "/api/v1/auth/login",
            json!({"username": "admin", "password": "change-me-now"}),
        )
        .await
        .no_content();

    // Ordinary registration of that name is still refused.
    t.guest()
        .post(
            "/api/v1/auth/register",
            json!({"username": "Admin", "email": "x@example.com", "password": PASSWORD, "acceptTerms": true}),
        )
        .await
        .assert_field_error("username", "username_taken");
}

#[tokio::test]
async fn admin_bootstrap_skips_when_unconfigured_or_when_an_admin_exists() {
    let t = TestApp::new().await;
    assert!(!seed::bootstrap_admin(&t.pool, &config(None)).await.unwrap());
    assert_eq!(t.count("users").await, 0);

    t.create_user("existing", hypercs_domain::Role::Admin).await;
    let cfg = config(Some(("another", "a@example.com", "change-me-now")));
    assert!(!seed::bootstrap_admin(&t.pool, &cfg).await.unwrap());
    assert_eq!(t.count("users").await, 1);

    // weak configured passwords are refused rather than silently accepted
    let t2 = TestApp::new().await;
    assert!(
        seed::bootstrap_admin(&t2.pool, &config(Some(("root1", "r@example.com", "short"))))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn sample_structure_is_seeded_once_and_counters_are_consistent() {
    let t = TestApp::new().await;
    // no admin yet: nothing to attribute the welcome topic to
    assert!(!seed::seed_sample_structure(&t.pool).await.unwrap());
    assert_eq!(t.count("categories").await, 0);

    seed::bootstrap_admin(
        &t.pool,
        &config(Some(("admin", "a@example.com", "change-me-now"))),
    )
    .await
    .unwrap();
    assert!(seed::seed_sample_structure(&t.pool).await.unwrap());
    assert!(
        !seed::seed_sample_structure(&t.pool).await.unwrap(),
        "idempotent"
    );
    assert_eq!(t.count("categories").await, 2);
    assert_eq!(t.count("boards").await, 3);
    assert_eq!(t.count("topics").await, 1);
    assert_eq!(t.count("posts").await, 1);

    // guests see the public category with the welcome topic; the admin-only
    // category is invisible to them
    let index = t.guest().get("/api/v1/categories").await;
    let categories = index.body["categories"].as_array().unwrap();
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0]["name"], "My first category");
    assert_eq!(categories[0]["boards"][0]["name"], "General Discussion");
    assert_eq!(categories[0]["boards"][0]["topicCount"], 1);
    assert_eq!(
        categories[0]["boards"][0]["lastPost"]["topicTitle"],
        "Welcome to HyperCS!"
    );
    let admin = Client::login_with(t.app.clone(), "admin", "change-me-now", false).await;
    let index = admin.get("/api/v1/categories").await;
    assert_eq!(index.body["categories"].as_array().unwrap().len(), 2);
    assert_eq!(
        index.body["categories"][0]["name"],
        "Administration and Moderation"
    );
    assert_eq!(index.body["categories"][0]["adminOnly"], true);

    let mut tx = t.pool.begin().await.unwrap();
    let report = hypercs_db::maintenance::recount_all_in(&mut tx)
        .await
        .unwrap();
    assert_eq!(report, hypercs_db::maintenance::RecountReport::default());
}

#[tokio::test]
async fn migrations_are_forward_only_and_safe_to_run_again() {
    let t = TestApp::new().await;
    // the test database is already migrated; running again is a no-op
    run_migrations(&t.pool).await.unwrap();
    let applied = t.count("_sqlx_migrations").await;
    assert_eq!(applied, hypercs_db::MIGRATOR.iter().count() as i64);
}
