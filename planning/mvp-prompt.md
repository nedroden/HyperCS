# Prompt: build the HyperCS MVP

You are a very experienced senior Rust and React/TypeScript engineer with years of production experience building web APIs (axum, sqlx, PostgreSQL) and React front-ends. You write secure, idiomatic, well-tested code and you care about architecture, accessibility and maintainability. You are joining the HyperCS project and you will build its MVP, end to end, autonomously.

Work in the repository at `/Users/robertmonden/Dev/hypercs`. Read this whole prompt first, then read the files listed under "Start here" before writing any code.

## 1. Product

HyperCS is open-source community software: a small CMS (homepage with news) combined with a discussion forum, sharing one account system. It supports two languages: English (US, default) and German.

## 2. Start here (read before coding)

1. `CLAUDE.md`, `api/CLAUDE.md`, `web/CLAUDE.md`: architecture, conventions, commands. These are binding. Update them whenever you change a convention, a command or the project structure.
2. `planning/requirements.md`: numbered requirements (R-G*, R-C*, R-A*, R-F*) and open questions. This is the source of truth for *what* to build; the decisions in sections 3 and 4 of this prompt resolve its open questions and override it where they differ. Update the file so it reflects the final decisions.
3. `mocks/*.html`: the visual and behavioural reference for *how it looks*. Open them in the browser pane to see them. Reuse the visual language; do not invent a new style.
4. The existing code in `api/` and `web/`: a bootstrapped skeleton (health endpoint, FSD layers, i18n, language switcher, themes, placeholder pages with lorem ipsum data, a stub session entity).

## 3. Decisions (already made; do not re-litigate)

**Stack**
- Database: PostgreSQL 17 (already in Docker Compose). Access via `sqlx` with SQL migrations in `api/migrations/`. Use compile-time-checked queries where practical (set up `cargo sqlx prepare` / offline mode so builds and Docker work without a live DB) or runtime-checked `query_as` consistently; pick one and document it.
- API: Rust, axum, tokio, sqlx, serde, thiserror, tracing, `validator`, `utoipa`. Restructure the existing single crate into a Cargo workspace as specified in section 4 ("API specification").
- Front-end: React + TypeScript (strict), Vite, React Router, TanStack Query, react-i18next, Feature-Sliced Design, CSS only in `web/themes/<theme>/`.
- Add: `utoipa` (+ Swagger UI or just the JSON served at `/api/v1/openapi.json`) and a **generated TypeScript client/types** from the OpenAPI document into `web/src/shared/api/generated/` (e.g. `openapi-typescript` + a thin typed fetch wrapper). Provide one command to regenerate it; never hand-edit generated files; do not hand-write request/response types.
- Front-end additions: `react-hook-form` + `zod` for forms (client validation mirroring the API), `react-markdown` + `rehype-sanitize` for rendering posts, with a live preview in the post forms.
- IDs: **UUID v7** primary keys, generated in the application (the `uuid` crate with the `v7` feature); Postgres 17 has no built-in `uuidv7()`.
- Time: store `timestamptz` in UTC. The UI shows relative times ("7 days ago", localized with `Intl.RelativeTimeFormat`) with the absolute timestamp in a tooltip, formatted in the user's configured time zone.

**Authentication and security**
- Server-side sessions: random opaque session token in an HTTP-only, `Secure` (in production), `SameSite=Lax` cookie; store only a hash of the token in the database. "Remember me" = 30-day persistent cookie; otherwise a browser-session cookie with 24 h idle expiry. Logout deletes the session. No JWTs.
- Passwords: Argon2id (`argon2` crate), minimum length 10, no composition rules, maximum length bounded (e.g. 128). Login responses must not reveal whether the username exists.
- CSRF protection for all state-changing requests (per-session token plus `Origin` check; see section 4).
- Security headers (CSP, `X-Content-Type-Options`, frame protection, `Referrer-Policy`) and strict CORS configuration.
- Not in the MVP: rate limiting, email sending, email verification, password reset (password reset was deliberately removed; do not add it), avatar uploads.
- Never log secrets or passwords. Configuration only via environment variables; document in `.env.example` files.

**Accounts and roles**
- Roles: `admin`, `moderator`, `member`. Guests (not logged in) may read the forum; members write.
  - Members: create topics, reply, edit/delete their own posts, manage their settings.
  - Moderators: additionally edit/delete any post, pin/unpin and lock/unlock topics, reply in locked topics.
  - Admins: everything above, plus manage categories and boards (including an **admin-only board flag**; boards/categories flagged that way are invisible and inaccessible to everyone else, like the "Administration and Moderation" category in the mocks), edit CMS content, and assign roles (user list with role dropdown).
- Authorization is enforced in the API service layer, never trusted from the client. Hide inaccessible actions in the UI too.
- Identity: the **username** is unique, case-insensitive, used for login and profile URLs, and cannot be changed. The **display name** is free text (not unique), defaults to the username, and is what the UI shows everywhere (posts, last post, member lists, header).
- Bootstrap: on first start the API creates the first admin from environment variables (`ADMIN_USERNAME`, `ADMIN_EMAIL`, `ADMIN_PASSWORD`) if no admin exists, and a seed migration/command creates the sample structure from the mocks (an admin-only "Administration and Moderation" category with "Forum administration", "My first category" with "General Discussion" and "Second board", a welcome topic, sample CMS content). Seed content is English; it is user content and is not translated.
- Language preference: single source of truth. Guests: remembered in `localStorage`. Logged-in users: stored in their profile and applied on login. The footer language switcher and the Preferences dropdown update the same value. Supported: `en-US`, `de`.

**Forum behaviour**
- Hierarchy: Category -> Board -> Topic -> Post. Posts are **Markdown**: store the raw text as submitted, render it sanitized in the front-end. Maximum length 10,000 characters (configurable constant, enforced in the API and the form). The first post's title is the topic title; reply titles ("Re: ...") are derived in the UI, not stored.
- Deletion is **hard delete with cascade**. Deleting the first post of a topic deletes the topic; counters are adjusted in the same transaction.
- **Denormalized counters** (board topic/post counts, last-post pointer, topic reply count, user post count, topic view count) maintained transactionally on insert/delete. Statistics (topics, posts, members, boards, newest member) can be queried directly. Add a test that proves counters stay consistent after create/delete/move sequences.
- Topic views: increment a view counter per topic page request (simple; no per-user dedupe required).
- Users online: a user counts as online if they had an authenticated request in the last 5 minutes, unless their "show me in the list of users online" preference is off. Update `last_seen` at most once per minute per session.
- Locked topics accept replies from moderators and admins only. Pinned topics sort first.
- Pagination everywhere: **20 topics per page, 15 posts per page**; API enforces a maximum page size; constants are easy to change.
- Notification preferences (reply/quote emails) are stored and editable, but nothing is sent.
- Quote: the Quote button inserts the post as a Markdown blockquote into the reply form.
- Public member profile page: display name, avatar (initial letter), join date, post count, "About me". Avatars are initial-letter placeholders only; hide the upload/remove controls from the settings page.
- Terms of service page: static page linked from registration; placeholder text, stored as CMS content editable by admins.
- Not in the MVP, so hide rather than leave dead controls: Report, avatar upload, password reset, search, email.

**CMS**
- The homepage (hero title/intro/button, three feature blocks, "Latest news" list) is loaded from the API. Admins edit it in the React app (an admin UI: edit hero and features, create/edit/delete news items). Apart from the "Forum" menu link, CMS pages must not mention the forum.

**i18n**
- Every user-facing string goes through i18n with keys in both `en-US` and `de`; use ICU/i18next plural keys for counts. The API returns machine-readable error `code`s; the front-end translates them. User-generated content is not translated. Write proper German (informal "du", as in the existing catalog).

## 4. API specification (decisions made; do not re-litigate)

**Workspace.** Convert `api/` into a Cargo workspace with four crates and a strict dependency direction (`api -> domain`, `db -> domain`, `server -> api, db, domain`):
- `hypercs-domain`: models, value types/newtypes, validation limits as constants, the authorization **policy module**, service traits/error types. No I/O, no HTTP, no SQL.
- `hypercs-db`: sqlx repositories and the SQL migrations.
- `hypercs-api`: axum routes, extractors, middleware, OpenAPI (`utoipa`), error mapping.
- `hypercs-server`: the binary: configuration, wiring, tracing setup, background tasks and the CLI. Subcommands: `serve` (default), `migrate`, `recount`, `openapi` (prints the OpenAPI JSON to stdout so client generation and drift checks need no running server). Migrations still run on startup behind an env flag (default on in dev).
Update the Dockerfile, `.dockerignore`, `lint.sh`, `format.sh` and all docs for the workspace.

**Conventions**
- JSON uses **camelCase** (`serde(rename_all = "camelCase")`); database columns stay snake_case. Same-origin deployment behind a reverse proxy (Vite proxy in dev): no CORS in practice, but keep an explicit, empty-by-default allow-list.
- Updates use **PATCH** (partial, with a tri-state type to distinguish absent from `null`); `PUT` only where replacing a whole resource makes sense (e.g. homepage blocks, structure ordering). `POST` creates and returns `201` with a `Location` header; `DELETE` returns `204`. Concurrency: **last write wins** (no optimistic locking, no `If-Match`, no 409 conflict flow); the edit history (below) is the safety net against lost content.
- Errors use **RFC 9457 `application/problem+json`**: `type`, `title`, `status`, `detail`, plus extensions `code` (stable, machine-readable, translated by the front-end) and `errors: [{ field, code, message }]` for validation failures. Document the error responses in OpenAPI and generate matching types.
- Lists use **page-number pagination** (`?page=2&perPage=20`) returning `{ items, page, perPage, total, totalPages }` with a max page size.
- URLs address topics, boards and categories by **UUID plus an optional cosmetic slug** (`/forum/topics/{uuid}/{slug}`; the UUID is authoritative and a wrong slug redirects/is ignored). Users are addressed by username in profile URLs.
- Responses **embed small summaries** (`author: { id, username, displayName, role }`, last-post summaries) joined in SQL so a page needs one request.
- Categories and boards have an explicit `position`; admins reorder with up/down buttons (no drag-and-drop). Moderators can **move a topic** to another board; counters are fixed in the same transaction.

**Validation and input handling**
- Typed request DTOs validated with the `validator` derive, run through a custom axum extractor that returns the problem+json shape. Domain rules (permissions, uniqueness, state) live in services.
- Limits, defined once as constants in the domain crate and mirrored in the OpenAPI schema (min/max) and the generated zod schemas: username 3-32; display name 1-50; email max 254; password 10-128; topic title 3-120; post body 1-10,000; category/board name 1-60; descriptions up to 200; About me up to 500; website `http(s)` up to 200; news title 1-120, news body up to 5,000.
- Usernames: `[A-Za-z0-9_-]`, reserved names blocked (`admin`, `root`, `api`, `system`, `moderator` and similar), case-insensitive unique. Display names: Unicode allowed, trimmed, control and zero-width characters rejected. Emails: syntactic check, lowercased, case-insensitive unique.
- Case-insensitive uniqueness via a **unique index on `lower(column)`**; store the original casing.
- Normalization: trim single-line fields, normalize all text to Unicode **NFC**, reject control characters (allow newlines and tabs in post bodies), normalize line endings to `\n`; Markdown bodies are otherwise stored as submitted.
- Abuse limits (not rate limiting): request **body size limits** (global default such as 64 KB with per-route overrides, `413` on violation); **flood control** for posting (minimum 5 s between a member's posts, stricter for accounts newer than 10 minutes; database-based, not IP-based); a **duplicate-post guard** (reject an identical consecutive post by the same user within a short window).
- Registration tells the user when a username or email is taken (`username_taken`, `email_taken`); login errors stay generic.

**Sessions and CSRF**
- Per-session CSRF token stored with the session and returned by `GET /api/v1/me`; the SPA sends it as `X-CSRF-Token` on every unsafe request. The API also checks `Origin` / `Sec-Fetch-Site`.
- **Rotate the session on login**, **revoke all other sessions on password change**, and provide an **active-sessions list** in Settings (device/browser hint, created and last-seen time, current marker, revoke button). Roles are read from the database on each request so role changes apply immediately.
- Housekeeping runs as in-process `tokio` tasks: a periodic purge of expired sessions, and a throttled `last_seen` update used for the users-online list.

**Data layer**
- **Denormalized counters are maintained in the service layer inside one transaction** (no DB triggers); the `recount` CLI recomputes all of them from the source tables as a repair tool, and tests prove consistency.
- **Full edit history** for posts: every edit stores a revision (old body, editor, timestamp); posts carry `editedAt` / `editedBy` and the UI shows an "edited" marker. Authors, moderators and admins can open the history view of a post (needs a mock and UI). Deleting a post removes its revisions.
  - An edit runs in one transaction that locks the post row (`SELECT ... FOR UPDATE`), inserts a revision containing the body as it was before the edit, then writes the new body. Two concurrent saves therefore serialize and both versions end up in the history; nothing is silently lost. Add a test that fires concurrent edits and asserts the revision chain is complete.
  - The history view has a **"Restore this version"** action, available to the same roles that may edit the post. It is a normal edit: it writes the old body back as a new revision (it does not rewrite or delete history) and goes through the same authorization, validation and locking.
- Authorization lives in a **central policy module** of pure, exhaustively unit-tested functions (`can_edit_post(user, post)`, `can_view_board(user, board)`, and so on) called from services; extractors only authenticate.
- Forward-only migrations (no down migrations), with foreign keys, `ON DELETE CASCADE` where hard deletes are specified, `CHECK` constraints for limits and enums, and indexes for the real query patterns.

**Observability**
- `X-Request-Id` generated or propagated, one tracing span per request (method, path, status, latency, user id; never secrets or bodies), JSON log output switchable by environment variable.
- Swagger UI at `/api/v1/docs`, enabled by an environment flag (on in development, off by default in production). `GET /api/v1/openapi.json` always available. A liveness health endpoint exists already; no readiness probe or Prometheus metrics for the MVP.

**Account lifecycle gap.** Account deletion is not part of the MVP. Record it in `planning/requirements.md` as a known gap (GDPR) to schedule later.

## 5. Process

1. **Mock first for unmocked screens.** Add static HTML mocks to `mocks/` in the existing visual style (same header/nav/footer/theme, English only) for: logged-out forum views, New topic, edit post, admin screens (categories/boards management, homepage/news editing, users and roles), public member profile, terms of service, the active-sessions section of Settings, the post edit-history view, the move-topic action, and 404/error states. Update `planning/requirements.md` with the new requirements and resolved questions. Then implement the screens in React to match.
2. **Plan, then build autonomously.** Write a short phased plan to `planning/mvp-plan.md` (data model with an ERD or table list, endpoint list, front-end slices, test plan) and then proceed without waiting for approval. Suggested phases: (1) DB schema, migrations, seed, config, error shape, OpenAPI plumbing, client generation; (2) auth, sessions, CSRF, accounts, settings, roles; (3) forum read endpoints and pages; (4) forum write endpoints, Markdown, quote, counters; (5) moderation and admin (structure, users/roles); (6) CMS (API, homepage, admin editing, terms); (7) polish: logged-out states, error pages, accessibility, German translations, docs.
3. Ask only if truly blocked; otherwise choose the most conventional option, record the decision in `planning/requirements.md` or `planning/mvp-plan.md`, and move on.
4. **Do not commit, push or create branches.** Leave all changes in the working tree for review.

## 6. Engineering standards

**Rust API**
- Edition 2024, no `unwrap()`/`expect()` in request paths (fine in tests and startup), no `unsafe`. Typed errors via `thiserror`, mapped in one place to RFC 9457 `application/problem+json` (see section 4); correct HTTP status codes.
- Thin handlers; business rules and authorization in `services/`; SQL only in `db/`; use transactions for multi-row invariants (counters, deleting topics, moving data).
- Validate and bound all input; parameterized queries only; add indexes for the actual query patterns (topics by board ordered by pinned/last post, posts by topic ordered by creation, sessions by token hash, case-insensitive unique username and email).
- Endpoints are under `/api/v1`, annotated for OpenAPI, using the same pagination shape everywhere. Provide a health endpoint (already exists).
- Tests: unit tests for pure logic; integration tests that run the real router against a real PostgreSQL database started with Testcontainers (a throwaway container per test run, with a fresh database or schema per test; the Docker daemon on the developer machine is Colima, so make sure it works there). Every endpoint gets a happy-path test and an authorization-failure test; add tests for CSRF rejection, session expiry, permission matrix by role, counter consistency and hard-delete cascades.

**React front-end**
- Strict Feature-Sliced Design as described in `web/CLAUDE.md`: respect layer import direction, public APIs via `index.ts`, segments by purpose, `steiger` must stay green. Replace the stub in `entities/session` with real session state backed by the API (`/me`), keep pages thin and compose widgets/features/entities.
- Likely slices (adjust as you see fit): entities `session`, `user`, `category`, `board`, `topic`, `post`, `news-item`; features `sign-in`, `sign-up`, `sign-out`, `create-topic`, `reply-to-topic`, `edit-post`, `delete-post`, `quote-post`, `moderate-topic`, `edit-profile`, `change-password`, `edit-preferences`, `manage-structure`, `manage-users`, `edit-homepage`, `switch-language`; pages for every route.
- Server state with TanStack Query (no ad-hoc fetch in effects); forms with react-hook-form + zod; surface server field errors from the API error shape; handle loading, empty, error and unauthorized states for every screen.
- All CSS in `web/themes/default/` (components never import CSS); colors, fonts and gradients come from `tokens.css` variables. While you are there, move the hard-coded colors and gradients still present in the theme files into tokens so a new theme can restyle everything.
- Markdown rendering must be sanitized; never use `dangerouslySetInnerHTML` with user content.
- Accessibility: semantic HTML, labelled controls, visible focus, keyboard-operable menus (including the language dropdown), sufficient contrast, correct heading order, proper `aria-current` and live regions for form errors.
- Responsive per the mocks (700px breakpoint), including a smaller header logo on narrow screens.
- Tests: component tests (Vitest + React Testing Library) for forms, permission-dependent UI, pagination, the language switcher and the Markdown preview. Mock the network at the fetch boundary using the generated types.

**Repository hygiene**
- Run `./format.sh` and `./lint.sh` (clippy with warnings denied, oxlint, steiger; Prettier with 4-space indent and single quotes; rustfmt with 4 spaces) and keep them green. Also run `cargo test`, `npm run typecheck`, `npm test` and `npm run build`.
- Docker: keep `docker-compose.yml` working (`docker-compose up --build` is the command on the developer's machine, because only the standalone binary is installed; `docker compose` works on machines with the plugin). Update it for new environment variables (admin bootstrap, cookie/CORS settings). Make sure the API image builds offline of a live DB if you use sqlx compile-time checks. Verify the full stack actually runs and a real end-to-end flow works (register, login, create topic, reply, quote, edit, delete, moderate, admin edit homepage, language switch); tear the stack down afterwards.
- Keep `README.md`, `CLAUDE.md`, `api/CLAUDE.md`, `web/CLAUDE.md`, `.env.example` files and `planning/requirements.md` accurate.
- Do not add dependencies without a reason; prefer widely used, maintained crates and packages and list every new dependency (with justification) in your final report. Never commit secrets.

## 7. Out of scope for the MVP

GitHub Actions CI, production Docker images, rate limiting, SMTP/email, password reset, avatar upload, search, report-post, soft delete, JWT, any additional languages.

## 8. Definition of done

- All MVP features in sections 3, 4 and 5 work end to end in the browser against the Docker Compose stack, in both English and German, as a guest, member, moderator and admin.
- `./format.sh --check`, `./lint.sh`, `cargo test`, and the web `typecheck`, `test` and `build` all pass.
- The OpenAPI document and generated client are in sync, with a documented command to regenerate them.
- Docs and requirements reflect the final state, and `planning/mvp-plan.md` records the plan, decisions and anything deferred.

## 9. Final report

When finished, report: what was built (by phase), the verification you actually ran with the real results (including anything that failed or was skipped), decisions you made where this prompt was silent, new dependencies with reasons, known limitations, and recommended next steps. State plainly what is verified and what is not.
