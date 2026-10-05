# HyperCS MVP – plan

This records the plan, the data model, the endpoint list, the front-end slice
map and the test plan for the MVP described in `planning/mvp-prompt.md`. It is
the implementation checklist; `planning/requirements.md` remains the
user-facing requirements doc. Decisions already made by the prompt are not
repeated here in full; only additions and choices the prompt left open are
recorded under "Decisions made while building".

## Phases (as executed)

1. DB schema, migrations, seed, config, error shape, OpenAPI plumbing, client generation.
2. Auth, sessions, CSRF, accounts, settings, roles.
3. Forum read endpoints and pages.
4. Forum write endpoints, Markdown, quote, counters, edit history.
5. Moderation and admin (structure, users/roles).
6. CMS (API, homepage, admin editing, terms).
7. Polish: logged-out states, error pages, accessibility, German translations, docs.

Each phase is implemented in both `api/` and `web/` together where practical,
so the app stays runnable after each phase.

## Data model (PostgreSQL, `api/migrations/`)

All tables: `id uuid` primary key (UUID v7, generated in the app),
`created_at timestamptz not null default now()`. Snake_case columns, camelCase
JSON.

- `users`: `username` (citext-like via `lower(username)` unique index),
  `display_name`, `email`, `password_hash`, `role` (`admin|moderator|member`
  check), `first_name`, `last_name`, `location`, `website`, `about_me`,
  `language` (`en-US|de`), `timezone`, `notify_on_reply`, `notify_on_quote`,
  `show_online`, `post_count`, `last_seen_at`.
- `sessions`: `user_id` FK, `token_hash` unique, `csrf_token`, `remember`,
  `user_agent`, `ip`, `expires_at`, `last_seen_at`.
- `categories`: `name`, `description`, `position`, `admin_only`.
- `boards`: `category_id` FK, `name`, `description`, `position`, `admin_only`,
  `topic_count`, `post_count`, `last_post_id` FK(nullable, `ON DELETE SET
  NULL`).
- `topics`: `board_id` FK, `title`, `slug`, `author_id` FK, `pinned`,
  `locked`, `reply_count`, `view_count`, `last_post_id` FK(nullable).
- `posts`: `topic_id` FK, `author_id` FK, `body`, `edited_at`, `edited_by` FK
  nullable. Cascade delete from `topics`.
- `post_revisions`: `post_id` FK cascade, `body` (body *before* the edit),
  `edited_by` FK, `created_at`.
- `news_items`: `title`, `slug`, `body`, `author_id` FK, `published_at`.
- `cms_content`: single-row-per-key table (`key text primary key`, `data
  jsonb`) for the homepage hero/features and the terms-of-service body, so
  admin edits don't need new migrations.

Denormalized counters touched transactionally: `boards.topic_count /
post_count / last_post_id`, `topics.reply_count / view_count /
last_post_id`, `users.post_count`.

Indexes: `lower(username)` unique, `lower(email)` unique, `sessions
(token_hash)` unique, `topics (board_id, pinned desc, last_post_at desc)`
(via join on last post), `posts (topic_id, created_at)`,
`boards(category_id, position)`.

## Endpoint list (`/api/v1`, OpenAPI via utoipa, `problem+json` errors)

Auth & account
- `POST /auth/register`, `POST /auth/login`, `POST /auth/logout`
- `GET /me`, `PATCH /me/profile`, `PATCH /me/account`, `PATCH
  /me/password`, `PATCH /me/preferences`
- `GET /me/sessions`, `DELETE /me/sessions/{id}`

Users
- `GET /users/{username}` (public profile)
- `GET /users` (admin: list + role), `PATCH /users/{id}/role` (admin)

Forum structure
- `GET /categories` (board index tree, counters, last post, admin-only
  filtered by role)
- `POST/PATCH/DELETE /categories`, `.../reorder` (admin)
- `POST/PATCH/DELETE /boards`, `.../reorder` (admin)
- `GET /boards/{id}` (board + paginated topics)

Topics & posts
- `POST /boards/{id}/topics` (create topic = board id + title + body)
- `GET /topics/{id}` (topic + paginated posts, increments view counter)
- `PATCH /topics/{id}` (moderation: pinned/locked), `PATCH
  /topics/{id}/move`, `DELETE /topics/{id}`
- `POST /topics/{id}/posts` (reply)
- `PATCH /posts/{id}`, `DELETE /posts/{id}`
- `GET /posts/{id}/revisions`

CMS
- `GET /cms/homepage`, `PUT /cms/homepage` (admin)
- `GET /cms/terms`, `PUT /cms/terms` (admin)
- `GET /news`, `GET /news/{slug}`, `POST/PATCH/DELETE /news` (admin)

Misc
- `GET /stats` (topics/posts/members/boards/newest member/online users)
- `GET /health` (existing)
- `GET /openapi.json`, `GET /docs` (Swagger UI, dev-only flag)

## Front-end slice map (FSD)

- `entities`: `session`, `user`, `category`, `board`, `topic`, `post`,
  `news-item`, `cms-page`.
- `features`: `sign-in`, `sign-up`, `sign-out`, `create-topic`,
  `reply-to-topic`, `edit-post`, `delete-post`, `quote-post`,
  `moderate-topic`, `move-topic`, `edit-profile`, `change-password`,
  `edit-preferences`, `manage-sessions`, `manage-structure`, `manage-users`,
  `edit-homepage`, `manage-news`, `switch-language`.
- `pages`: `home`, `login`, `register`, `terms`, `settings`, `forum-index`,
  `board`, `topic`, `new-topic`, `profile`, `admin-structure`,
  `admin-users`, `admin-homepage`, `admin-news`, `post-history`,
  `not-found`.
- `widgets`: `header`, `footer`, `post-list`, `topic-list`.

Generated OpenAPI client lives in `web/src/shared/api/generated/` (command:
`npm run generate:api`, calls the server's `openapi` CLI subcommand into a
temp file, then `openapi-typescript`). A thin typed fetch wrapper in
`shared/api/http.ts` adds the CSRF header and credentials.

## Test plan

Rust: domain unit tests for the policy module (exhaustive per role/action)
and validation/normalization; service-level tests for counters,
cascades and edit history using a real Postgres via `testcontainers`;
router integration tests per endpoint (happy path + one authorization
failure), plus dedicated tests for CSRF rejection, session expiry and
counter consistency across create/delete/move.

Web: Vitest + RTL component tests for forms (sign-in/sign-up/reply),
permission-dependent UI (post actions by role), pagination, the language
switcher and Markdown preview/sanitization.

## Decisions made while building (prompt silent or ambiguous)

- Slug generation: lowercase, ASCII, non-alphanumeric runs collapsed to a
  single `-`, truncated to 80 chars; collisions are fine (slug is cosmetic,
  UUID is authoritative), no uniqueness constraint.
- `recount` and `migrate` and `openapi` CLI subcommands live in
  `hypercs-server` as clap subcommands; `serve` is the default when no
  subcommand is given, to keep `cargo run` working unchanged.
- Reserved usernames list: `admin, administrator, root, system, api,
  moderator, mod, support, staff, help, null, undefined, hypercs`.
- Flood control / duplicate-post guard thresholds are domain constants
  (`MIN_POST_INTERVAL`, `NEW_ACCOUNT_POST_INTERVAL`, `DUPLICATE_POST_WINDOW`).
- Avatars: initial-letter only, computed from `display_name` first grapheme,
  colour derived deterministically from the user id (front-end only, no
  stored colour).
- "Users online" query: sessions with `last_seen_at` within 5 minutes
  joined to `show_online = true`, deduplicated by user.
- CMS content versioning: homepage/terms are last-write-wins rows (no
  history), consistent with "no optimistic locking" for the MVP; only post
  edits get full revision history per spec.
- Member list / search are out of scope (not required by section 3/4); the
  public profile page is reachable from post author links and admin user
  list only.

## Known limitations / deferred (see final report for the authoritative list)

Tracked here as they are discovered during the build; the final report to
the user is the source of truth for what shipped vs. what is deferred.
