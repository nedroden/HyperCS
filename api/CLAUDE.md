# HyperCS API (Rust)

Rust JSON web API for the HyperCS CMS and forum. This file defines how it is built. Not yet added from the intended stack: OpenAPI generation (`utoipa`), auth, validation. See the root `CLAUDE.md` for product-wide rules and `planning/requirements.md` for what to build.

## Intended stack

Defaults to use when scaffolding. If you want to deviate, say why first.

- Edition 2021 or later, stable toolchain (pin it in `rust-toolchain.toml`).
- Web framework: `axum` on `tokio`.
- Database: PostgreSQL via `sqlx` (compile-time checked queries, SQL migrations in `migrations/`).
- Serialization: `serde` / `serde_json`.
- Validation: `validator` or hand-written checks at the boundary.
- Auth: server-side sessions with an HTTP-only, `SameSite` cookie, or a short-lived token plus refresh; passwords hashed with `argon2`. Never store or log plaintext passwords.
- Errors: `thiserror` for library/domain errors, mapped to the API error shape in one place (an `IntoResponse` impl).
- Config: environment variables, loaded once at startup into a typed config struct (`.env.example` documents them).
- Logging: `tracing` + `tracing-subscriber`.
- OpenAPI: generate with `utoipa` (or equivalent) and serve the spec; the front-end client is generated from it.

## Layout

```
api/
  Cargo.toml
  migrations/        # sqlx migrations, one per change, never edited after merge
  src/
    main.rs          # startup only: config, tracing, DB pool, router, serve
    config.rs
    error.rs         # AppError and its HTTP mapping
    routes/          # HTTP layer: extract, call a service, shape the response
    services/        # business logic, no HTTP types
    db/              # queries and row types
    models/          # domain types shared between layers
  tests/             # integration tests against a real test database
```

Keep the layers separate: handlers stay thin, business rules live in services, SQL lives in `db/`.

## Domain

Categories → Boards → Topics → Posts, plus Users (roles, e.g. admin), CMS content (homepage hero, features, news), and user settings (profile, account, preferences including language and time zone). Counts (topics, posts, last post per board, statistics) must be correct; decide deliberately between computing them and storing denormalized counters, and keep them consistent in a transaction.

## Rules

- Authorization is enforced in the API, never trusted from the client. Edit/delete of posts is limited to the author and admins; check this in the service layer.
- Validate and bound all input (lengths, page sizes). Paginate every list endpoint with a maximum page size.
- Use parameterized queries only (`sqlx` macros); no string-built SQL.
- Escape/sanitize nothing silently: store post content as submitted and render safely in the front-end. Decide the formatting syntax (plain text vs markup) before building posts; it is an open question in the requirements.
- No `unwrap()`/`expect()` in request paths; return `AppError`. `unwrap` is fine in tests and in startup code that should crash on bad config.
- No `unsafe` unless there is a documented reason.
- Public items that form the API contract carry doc comments, and OpenAPI annotations must stay in sync with the handlers.
- Time: store UTC (`timestamptz`); the front-end handles display and time zones.

## Testing

- Unit tests next to the code (`#[cfg(test)]`); integration tests in `tests/` that run the real router against a test database (use a fresh schema/transaction per test).
- Every endpoint gets at least a happy-path and an authorization-failure test.
- Before finishing a change, all of these must pass:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
