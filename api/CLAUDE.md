# HyperCS API (Rust)

Rust JSON web API for the HyperCS CMS and forum. See the root `CLAUDE.md` for
product-wide rules, `planning/requirements.md` for what to build, and
`planning/mvp-plan.md` for the data model, endpoint list and test plan.

## Stack

- Edition 2024, stable toolchain (pinned in `rust-toolchain.toml`).
- Web framework: `axum` on `tokio`.
- Database: PostgreSQL via `sqlx`. Queries are **runtime-checked**
  (`sqlx::query_as` with `#[derive(FromRow)]` structs, not the compile-time
  `query!`/`query_as!` macros): this was chosen over compile-time checking so
  the Docker image builds without a live database and so no `cargo sqlx
  prepare` / `.sqlx` cache needs to be kept in sync. All queries are still
  parameterized (no string-built SQL); sqlx itself additionally refuses to
  build a dynamic (non-`'static`) SQL string, so ad-hoc query strings must be
  `const_format::concatcp!`'d at compile time if they need composing.
- IDs: UUID v7 (`uuid` crate, `v7` feature), generated in the app.
- Validation: `validator` derive on request DTOs, run through a custom axum
  extractor (`ValidatedJson`) that returns the problem+json shape.
- Auth: server-side sessions, opaque token in an HTTP-only `SameSite=Lax`
  cookie (only a hash of the token is stored); passwords hashed with
  `argon2`. No JWTs, no password reset.
- Errors: `thiserror` domain errors in `hypercs-domain`, mapped once in
  `hypercs-api::error` to RFC 9457 `application/problem+json`.
- Config: environment variables loaded once into a typed `Config` in
  `hypercs-server` (`.env.example` documents them).
- Logging: `tracing` + `tracing-subscriber`.
- OpenAPI: `utoipa`, served at `/api/v1/openapi.json` (always) and Swagger UI
  at `/api/v1/docs` (behind `API_DOCS_ENABLED`). The front-end client is
  generated from `cargo run -p hypercs-server -- openapi` (no running server
  or env needed): `cd web && npm run generate:api`; `npm run check:api` fails
  if the checked-in `schema.ts` is stale. Every handler documents typed
  success bodies, params, `201` + `Location`, `204` for deletes and
  `application/problem+json` errors (`ProblemDetails`); schema limits are
  repeated as `#[schema(...)]` and a unit test checks them against the domain
  constants.
- Conventions: DB row structs (`FromRow`) live in `hypercs-db`; response DTOs
  (named, concrete, e.g. `TopicSummaryPage`) live in `hypercs-api::dto`.
  Updates are partial PATCH (`null` clears nullable fields). Command-style
  actions are `POST` sub-resources (`/reorder`, `/revisions/{id}/restore`);
  moving a topic is `PATCH /topics/{id}` with `boardId`.
- Posts: `GET /posts/{id}` returns `PostDetail` (`post` + topic/board/category
  `context` incl. `isFirstPost` and the `page` of the topic it is on); hidden
  boards give 404 like everywhere else. Also `PATCH|DELETE /posts/{id}` and
  `GET /posts/{id}/revisions`.
- Lock order in transactions: posting advisory lock, topic, board(s) by id,
  user counters.

## Workspace layout

Four crates under `crates/`, with a strict dependency direction
(`hypercs-api -> hypercs-domain`, `hypercs-db -> hypercs-domain`,
`hypercs-server -> hypercs-api, hypercs-db, hypercs-domain`):

```
api/
  Cargo.toml          # workspace manifest + shared dependency versions
  crates/
    domain/            # models, newtypes, constants, the policy module, DomainError. No I/O.
      src/
        constants.rs   # validation limits, pagination sizes, flood-control thresholds
        models.rs       # Role, AuthUser
        pagination.rs   # PageParams
        policy.rs       # can_edit_post(), can_view_board(), ... - pure, exhaustively unit tested
        validation.rs   # normalize_text, slugify, username/display-name checks
        errors.rs       # DomainError (validation/not_found/forbidden/conflict/...)
    db/                # sqlx repositories and the SQL migrations
      migrations/       # 0001_init.sql, forward-only, no down migrations
      src/
        users.rs, sessions.rs, structure.rs, topics.rs, posts.rs, news.rs, cms.rs, stats.rs
        maintenance.rs  # recount (counter repair)
    api/                # axum routes, extractors, services, OpenAPI
      src/
        auth/            # password hashing, session cookie, AuthUser/OptionalAuthUser/SameOrigin extractors
        dto/             # request/response DTOs (validator + utoipa::ToSchema)
        middleware.rs    # request id + tracing span, security headers
        services/        # business rules + authorization calls into policy; one file per domain area
        routes/          # thin handlers: extract, call a service, shape the response
        error.rs         # ApiError -> problem+json
        openapi.rs       # utoipa OpenApi aggregating every route + schema
    server/             # the binary: config, wiring, CLI, background tasks
      src/
        main.rs          # clap CLI: `serve` (default), `migrate`, `recount`, `openapi`
        lib.rs, config.rs, seed.rs, housekeeping.rs, recount.rs  # library shared with tests
      tests/             # integration tests (Testcontainers Postgres)
```

## Domain

Categories -> Boards -> Topics -> Posts, Users (roles: `admin`, `moderator`,
`member`), sessions, post edit history (`post_revisions`), CMS content
(`cms_content` key/value table for the homepage and terms; `news_items`).
Denormalized counters (board topic/post counts + last post, topic reply/view
counts + last post, user post count) are maintained transactionally by the
service layer on every insert/delete/move; the `recount` CLI recomputes them
all from source tables as a repair tool (`cargo run -p hypercs-server --
recount`).

## Rules

- Authorization lives in `hypercs_domain::policy` (pure functions, exhaustively
  unit tested) and is called from `hypercs-api::services`; extractors
  (`AuthUser`, `OptionalAuthUser`) only authenticate. Never trust the client.
- Thin handlers in `routes/`; business rules and transactions in `services/`;
  SQL only in `hypercs-db`.
- CSRF: `AuthUser` enforces same-origin + a matching `X-CSRF-Token` header for
  any state-changing request on an authenticated route; `SameOrigin` does the
  origin check alone for register/login (no session yet to carry a token).
- No `unwrap()`/`expect()` in request paths (fine in tests and startup code
  that should crash on bad config). No `unsafe`.
- Validate and bound all input; every list endpoint is paginated with a
  `hypercs_db::pagination::Page<T>` response and a maximum page size.
- Time: store UTC (`timestamptz`, `chrono`); the front-end handles display
  and time zones.

## Testing

- Unit tests next to the code (`#[cfg(test)]`), notably the policy module and
  `hypercs_domain::validation`.
- Integration tests in `crates/server/tests/api/` (one test binary, shared
  helpers in `common/`) run the real axum router (via
  `tower::ServiceExt::oneshot`) against PostgreSQL 17 started once per run with
  `testcontainers-modules`; each test gets a fresh database cloned from a
  migrated template. Requires
  Docker; on a machine using Colima, make sure `DOCKER_HOST` points at its
  socket, e.g. `export DOCKER_HOST=unix://$HOME/.colima/default/docker.sock`.
- Before finishing a change, all of these must pass:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
```

## Commands

```bash
cargo run -p hypercs-server                  # serve (equivalent to `-- serve`)
cargo run -p hypercs-server -- migrate       # run pending migrations and exit
cargo run -p hypercs-server -- recount       # recompute denormalized counters
cargo run -p hypercs-server -- openapi       # print the OpenAPI document to stdout
```
