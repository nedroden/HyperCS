//! Shared test harness.
//!
//! One PostgreSQL container is started per test *run* (the whole `api` test
//! binary); a migrated template database is created once, and every test gets
//! its own fresh database cloned from that template. The container is removed
//! when the process exits.

#![allow(dead_code)]

use std::{
    process::Command,
    sync::{
        OnceLock,
        atomic::{AtomicU32, Ordering},
    },
};

use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Method, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use hypercs_api::{
    routes,
    state::{ApiConfig, AppState, FloodConfig},
};
use hypercs_db::PgPool;
use hypercs_domain::Role;
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ImageExt, runners::AsyncRunner},
};
use tower::ServiceExt;
use uuid::Uuid;

pub const PASSWORD: &str = "correct horse battery";
pub const ORIGIN: &str = "http://localhost";
pub const HOST: &str = "localhost";

struct ContainerInfo {
    host: String,
    port: u16,
    id: String,
}

static CONTAINER: OnceLock<ContainerInfo> = OnceLock::new();
static DB_COUNTER: AtomicU32 = AtomicU32::new(0);
static PASSWORD_HASH: OnceLock<String> = OnceLock::new();

fn admin_url(info: &ContainerInfo, database: &str) -> String {
    format!(
        "postgres://postgres:postgres@{}:{}/{database}",
        info.host, info.port
    )
}

/// Starts the shared container (once per process) on a dedicated thread with
/// its own runtime, so it outlives the per-test runtimes.
fn container() -> &'static ContainerInfo {
    CONTAINER.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("runtime");
            runtime.block_on(async move {
                let container = Postgres::default()
                    .with_tag("17-alpine")
                    .start()
                    .await
                    .expect("failed to start postgres (is Docker/Colima running? set DOCKER_HOST)");
                let info = ContainerInfo {
                    port: container.get_host_port_ipv4(5432).await.expect("port"),
                    host: container.get_host().await.expect("host").to_string(),
                    id: container.id().to_string(),
                };

                // Migrate once into a template database.
                let admin = PgPoolOptions::new()
                    .max_connections(1)
                    .connect(&admin_url(&info, "postgres"))
                    .await
                    .expect("connect admin");
                sqlx::query("create database hypercs_template")
                    .execute(&admin)
                    .await
                    .expect("create template");
                admin.close().await;
                let template = PgPoolOptions::new()
                    .max_connections(1)
                    .connect(&admin_url(&info, "hypercs_template"))
                    .await
                    .expect("connect template");
                hypercs_db::MIGRATOR.run(&template).await.expect("migrate");
                template.close().await;

                tx.send(info).expect("send info");
                // Keep the container alive for the life of the process.
                let _keep = container;
                std::future::pending::<()>().await;
            });
        });
        rx.recv().expect("container info")
    })
}

#[ctor::dtor]
fn remove_container() {
    if let Some(info) = CONTAINER.get() {
        let _ = Command::new("docker")
            .args(["rm", "-f", "-v", &info.id])
            .output();
    }
}

/// Process-wide capture of the JSON log output. A *global* subscriber is
/// installed once (thread-local `set_default` races with tracing's callsite
/// interest cache when tests run in parallel); tests pick out their own lines
/// by request id.
#[derive(Clone, Default)]
pub struct LogBuffer(pub std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for LogBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogBuffer {
    type Writer = LogBuffer;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

static LOGS: OnceLock<LogBuffer> = OnceLock::new();

pub fn logs() -> &'static LogBuffer {
    LOGS.get_or_init(|| {
        let buffer = LogBuffer::default();
        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_writer(buffer.clone())
            .with_max_level(tracing::Level::INFO)
            .finish();
        let _ = tracing::subscriber::set_global_default(subscriber);
        buffer
    })
}

/// The captured JSON log lines (parsed) whose `request_id` matches.
pub fn log_lines_for(request_id: &str) -> Vec<Value> {
    let text = String::from_utf8(logs().0.lock().unwrap().clone()).unwrap();
    text.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["fields"]["request_id"] == request_id)
        .collect()
}

pub fn all_logs() -> String {
    String::from_utf8(logs().0.lock().unwrap().clone()).unwrap()
}

pub fn password_hash() -> &'static str {
    PASSWORD_HASH.get_or_init(|| {
        hypercs_api::auth::crypto::hash_password(PASSWORD).expect("hash test password")
    })
}

/// A fresh database (cloned from the migrated template) plus the router.
pub struct TestApp {
    pub app: Router,
    pub pool: PgPool,
    pub config: ApiConfig,
}

pub fn test_config() -> ApiConfig {
    ApiConfig {
        cookie_secure: false,
        allowed_origins: vec![],
        docs_enabled: false,
        // Most tests post quickly; flood control has its own tests.
        flood: FloodConfig::disabled(),
    }
}

impl TestApp {
    pub async fn new() -> Self {
        Self::with_config(test_config()).await
    }

    pub async fn with_config(config: ApiConfig) -> Self {
        let _ = logs();
        let info = container();
        let name = format!(
            "test_{}_{}",
            std::process::id(),
            DB_COUNTER.fetch_add(1, Ordering::SeqCst)
        );
        let admin = PgPoolOptions::new()
            .max_connections(1)
            .connect(&admin_url(info, "postgres"))
            .await
            .expect("connect admin");
        // The name is generated here (pid + counter), never user input.
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "create database {name} template hypercs_template"
        )))
        .execute(&admin)
        .await
        .expect("create test database");
        admin.close().await;

        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(&admin_url(info, &name))
            .await
            .expect("connect test database");
        let app = routes::router(AppState {
            db: pool.clone(),
            config: config.clone(),
        });
        Self { app, pool, config }
    }

    pub fn guest(&self) -> Client {
        Client::new(self.app.clone())
    }

    /// Inserts an account directly (fast; the password is [`PASSWORD`]).
    pub async fn create_user(&self, username: &str, role: Role) -> Uuid {
        let id = Uuid::now_v7();
        hypercs_db::users::create_user(
            &self.pool,
            id,
            username,
            &format!("{}@example.com", username.to_lowercase()),
            password_hash(),
            role,
            "en-US",
        )
        .await
        .expect("create user");
        id
    }

    /// Logs in through the real endpoint and fetches the CSRF token.
    pub async fn login(&self, username: &str) -> Client {
        Client::login(self.app.clone(), username, false).await
    }

    pub async fn user_with_client(&self, username: &str, role: Role) -> (Uuid, Client) {
        let id = self.create_user(username, role).await;
        (id, self.login(username).await)
    }

    pub async fn create_category(&self, name: &str, admin_only: bool) -> Uuid {
        let id = Uuid::now_v7();
        let position = hypercs_db::structure::next_category_position(&self.pool)
            .await
            .unwrap();
        hypercs_db::structure::create_category(&self.pool, id, name, "", admin_only, position)
            .await
            .unwrap();
        id
    }

    pub async fn create_board(&self, category: Uuid, name: &str, admin_only: bool) -> Uuid {
        let id = Uuid::now_v7();
        let position = hypercs_db::structure::next_board_position(&self.pool, category)
            .await
            .unwrap();
        hypercs_db::structure::create_board(
            &self.pool, id, category, name, "", admin_only, position,
        )
        .await
        .unwrap();
        id
    }

    /// Users of every role, a public and an admin-only board.
    pub async fn world(&self) -> World {
        let (admin_id, admin) = self.user_with_client("root_admin", Role::Admin).await;
        let (moderator_id, moderator) = self.user_with_client("mod_mia", Role::Moderator).await;
        let (member_id, member) = self.user_with_client("member_max", Role::Member).await;
        let (other_id, other) = self.user_with_client("other_olga", Role::Member).await;
        let category = self.create_category("Public category", false).await;
        let board = self.create_board(category, "Public board", false).await;
        let board2 = self.create_board(category, "Second board", false).await;
        let admin_category = self.create_category("Admins only", true).await;
        let admin_board = self.create_board(admin_category, "Staff room", true).await;
        World {
            admin,
            moderator,
            member,
            other,
            guest: self.guest(),
            admin_id,
            moderator_id,
            member_id,
            other_id,
            category,
            board,
            board2,
            admin_category,
            admin_board,
        }
    }

    pub async fn counts(&self, board: Uuid) -> (i32, i32) {
        let row: (i32, i32) =
            sqlx::query_as("select topic_count, post_count from boards where id = $1")
                .bind(board)
                .fetch_one(&self.pool)
                .await
                .unwrap();
        row
    }

    pub async fn user_post_count(&self, user: Uuid) -> i32 {
        let row: (i32,) = sqlx::query_as("select post_count from users where id = $1")
            .bind(user)
            .fetch_one(&self.pool)
            .await
            .unwrap();
        row.0
    }

    /// `select count(*) from <table>` (table names come from test code only).
    pub async fn count(&self, table: &str) -> i64 {
        let row: (i64,) =
            sqlx::query_as(sqlx::AssertSqlSafe(format!("select count(*) from {table}")))
                .fetch_one(&self.pool)
                .await
                .unwrap();
        row.0
    }

    pub async fn scalar_i64(&self, sql: &'static str) -> i64 {
        let row: (i64,) = sqlx::query_as(sql).fetch_one(&self.pool).await.unwrap();
        row.0
    }
}

pub struct World {
    pub admin: Client,
    pub moderator: Client,
    pub member: Client,
    pub other: Client,
    pub guest: Client,
    pub admin_id: Uuid,
    pub moderator_id: Uuid,
    pub member_id: Uuid,
    pub other_id: Uuid,
    pub category: Uuid,
    pub board: Uuid,
    pub board2: Uuid,
    pub admin_category: Uuid,
    pub admin_board: Uuid,
}

#[derive(Debug)]
pub struct Res {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Value,
}

impl Res {
    /// Asserts the status, printing the body on failure.
    #[track_caller]
    pub fn assert_status(&self, expected: StatusCode) -> &Self {
        assert_eq!(
            self.status, expected,
            "unexpected status; body: {}",
            self.body
        );
        self
    }

    pub fn ok(&self) -> &Self {
        self.assert_status(StatusCode::OK)
    }
    pub fn created(&self) -> &Self {
        self.assert_status(StatusCode::CREATED)
    }
    pub fn no_content(&self) -> &Self {
        self.assert_status(StatusCode::NO_CONTENT)
    }

    /// The problem+json `code`.
    pub fn code(&self) -> &str {
        self.body["code"].as_str().unwrap_or("<no code>")
    }

    /// Asserts a problem+json response with this status and code.
    #[track_caller]
    pub fn assert_problem(&self, status: StatusCode, code: &str) -> &Self {
        self.assert_status(status);
        assert_eq!(self.code(), code, "unexpected code; body: {}", self.body);
        assert_eq!(
            self.headers
                .get(header::CONTENT_TYPE)
                .map(|v| v.to_str().unwrap()),
            Some("application/problem+json")
        );
        assert_eq!(self.body["status"], status.as_u16());
        self
    }

    /// The field-error codes for one field of a 422.
    pub fn field_codes(&self, field: &str) -> Vec<String> {
        self.body["errors"]
            .as_array()
            .map(|errors| {
                errors
                    .iter()
                    .filter(|e| e["field"] == field)
                    .map(|e| e["code"].as_str().unwrap_or("").to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    #[track_caller]
    pub fn assert_field_error(&self, field: &str, code: &str) -> &Self {
        self.assert_problem(StatusCode::UNPROCESSABLE_ENTITY, "validation_failed");
        assert!(
            self.field_codes(field).iter().any(|c| c == code),
            "expected field error {field}/{code}; body: {}",
            self.body
        );
        self
    }

    /// The id from a 201 body (also checks the `Location` header ends with it).
    pub fn created_id(&self) -> Uuid {
        self.created();
        let id: Uuid = self.body["id"].as_str().unwrap().parse().unwrap();
        let location = self
            .headers
            .get(header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.ends_with(&id.to_string()), "Location: {location}");
        id
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }
}

/// An HTTP client against the in-process router, behaving like the SPA:
/// sends the session cookie, the CSRF token and a same-origin `Origin`.
#[derive(Clone)]
pub struct Client {
    app: Router,
    pub cookie: Option<String>,
    pub csrf: Option<String>,
    pub origin: Option<String>,
    pub extra_headers: Vec<(String, String)>,
}

impl Client {
    pub fn new(app: Router) -> Self {
        Self {
            app,
            cookie: None,
            csrf: None,
            origin: Some(ORIGIN.to_string()),
            extra_headers: vec![],
        }
    }

    pub async fn login(app: Router, username: &str, remember: bool) -> Self {
        Self::login_with(app, username, PASSWORD, remember).await
    }

    pub async fn login_with(app: Router, username: &str, password: &str, remember: bool) -> Self {
        let mut client = Client::new(app);
        let res = client
            .post(
                "/api/v1/auth/login",
                json!({"username": username, "password": password, "remember": remember}),
            )
            .await;
        res.no_content();
        client.cookie = Some(session_cookie_pair(&res));
        client.refresh_csrf().await;
        client
    }

    pub async fn refresh_csrf(&mut self) {
        let res = self.get("/api/v1/me").await;
        res.ok();
        self.csrf = Some(res.body["csrfToken"].as_str().unwrap().to_string());
    }

    pub fn without_csrf(&self) -> Self {
        Self {
            csrf: None,
            ..self.clone()
        }
    }

    pub fn with_csrf(&self, token: &str) -> Self {
        Self {
            csrf: Some(token.to_string()),
            ..self.clone()
        }
    }

    pub fn with_origin(&self, origin: Option<&str>) -> Self {
        Self {
            origin: origin.map(str::to_string),
            ..self.clone()
        }
    }

    pub fn with_header(&self, name: &str, value: &str) -> Self {
        let mut client = self.clone();
        client
            .extra_headers
            .push((name.to_string(), value.to_string()));
        client
    }

    pub async fn get(&self, path: &str) -> Res {
        self.request(Method::GET, path, None).await
    }
    pub async fn post(&self, path: &str, body: Value) -> Res {
        self.request(Method::POST, path, Some(body)).await
    }
    pub async fn post_empty(&self, path: &str) -> Res {
        self.request(Method::POST, path, None).await
    }
    pub async fn put(&self, path: &str, body: Value) -> Res {
        self.request(Method::PUT, path, Some(body)).await
    }
    pub async fn patch(&self, path: &str, body: Value) -> Res {
        self.request(Method::PATCH, path, Some(body)).await
    }
    pub async fn delete(&self, path: &str) -> Res {
        self.request(Method::DELETE, path, None).await
    }

    pub async fn request(&self, method: Method, path: &str, body: Option<Value>) -> Res {
        let body = body.map(|v| v.to_string());
        self.raw(
            method,
            path,
            body.map(String::into_bytes),
            Some("application/json"),
        )
        .await
    }

    /// Sends exact bytes with an optional content type.
    pub async fn raw(
        &self,
        method: Method,
        path: &str,
        body: Option<Vec<u8>>,
        content_type: Option<&str>,
    ) -> Res {
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header(header::HOST, HOST);
        if let Some(origin) = &self.origin {
            builder = builder.header(header::ORIGIN, origin);
        }
        if let Some(cookie) = &self.cookie {
            builder = builder.header(header::COOKIE, cookie);
        }
        if let Some(csrf) = &self.csrf {
            builder = builder.header("x-csrf-token", csrf);
        }
        for (name, value) in &self.extra_headers {
            builder = builder.header(name, value);
        }
        let has_body = body.is_some();
        if has_body && let Some(content_type) = content_type {
            builder = builder.header(header::CONTENT_TYPE, content_type);
        }
        let request = builder
            .body(Body::from(body.unwrap_or_default()))
            .expect("request");
        let response = self.app.clone().oneshot(request).await.expect("response");
        let (parts, body) = response.into_parts();
        let bytes = body.collect().await.expect("body").to_bytes();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
        };
        Res {
            status: parts.status,
            headers: parts.headers,
            body,
        }
    }

    // ---- API conveniences (assert success) ----

    pub async fn create_topic(&self, board: Uuid, title: &str, body: &str) -> Uuid {
        self.post(
            &format!("/api/v1/boards/{board}/topics"),
            json!({"title": title, "body": body}),
        )
        .await
        .created_id()
    }

    pub async fn reply(&self, topic: Uuid, body: &str) -> Uuid {
        self.post(
            &format!("/api/v1/topics/{topic}/posts"),
            json!({"body": body}),
        )
        .await
        .created_id()
    }
}

/// `hypercs_session=<token>` from a login response's `Set-Cookie`.
pub fn session_cookie_pair(res: &Res) -> String {
    res.headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.starts_with("hypercs_session="))
        .expect("session Set-Cookie")
        .split(';')
        .next()
        .unwrap()
        .to_string()
}

pub fn set_cookie_header(res: &Res) -> String {
    res.headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.starts_with("hypercs_session="))
        .expect("session Set-Cookie")
        .to_string()
}
