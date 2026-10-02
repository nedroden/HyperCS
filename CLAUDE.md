# HyperCS

Open-source community software: a small CMS (homepage, news) combined with a discussion forum, sharing one account system. Started as a Hacktoberfest project.

## Status

Both projects are bootstrapped (health endpoint, app shell, i18n, language switcher) but no product features exist yet.

- `mocks/` – static HTML mockups of every page (English only, plain HTML/CSS, one file per page). These are the visual and behavioural reference for the UI.
- `planning/requirements.md` – numbered requirements (R-G*, R-C*, R-A*, R-F*) derived from the mocks, with open questions at the end.
- `api/` – Rust API (axum, sqlx). `web/` – React + Vite + TypeScript app.
- `docker-compose.yml` – local stack: PostgreSQL, API, and the web dev server (`docker compose up --build`; web on :5173, API on :8080).

## Target architecture

A monorepo with two independent projects:

| Path   | What                | Stack                          |
| ------ | ------------------- | ------------------------------ |
| `api/` | JSON web API        | Rust                           |
| `web/` | Browser front-end   | React + TypeScript (SPA), Feature-Sliced Design       |

The front-end talks to the API over HTTP/JSON only. No shared code between the two; the API contract is the interface (see "API contract").

Each project has its own `CLAUDE.md` with stack-specific rules – read it before working in that directory.

## Source of truth

1. `planning/requirements.md` defines *what* to build. If a task conflicts with it, or the requirements are silent or ambiguous, ask or note it. When a requirement changes, update the file in the same change.
2. `mocks/*.html` defines *how it looks*. Reuse their visual design (colors, spacing, typography, layout) in the React components. Don't invent a new style.
3. Open questions in the requirements file are undecided. Don't silently decide them; surface them.

## Product rules to keep in mind

- The CMS pages must not mention the forum, except for the "Forum" menu link.
- Navigation: logged in = Home, Forum, Logout; logged out = Home, Forum, Login.
- There is no password reset flow (deliberately removed).
- Supported languages: English (US) (default) and German. All user-facing strings go through the i18n layer from the start; never hard-code UI text in components. Use US spelling in English strings.
- User-generated content (posts, topic titles) is never translated.
- Dates are shown relative ("7 days ago") and counts are pluralized correctly in both languages.

## API contract

- REST over JSON, versioned under `/api/v1`.
- The API owns the contract. Once it exists, keep an OpenAPI document in `api/` and generate the front-end client/types from it rather than hand-writing request types in `web/`.
- Errors use one consistent JSON shape (machine-readable `code`, human-readable `message`, optional field errors). HTTP status codes are meaningful.
- Pagination is consistent across list endpoints (topics in a board, posts in a topic).
- The API returns data and error codes, not translated text. Translation happens in the front-end. Exception: emails and other server-originated text are localized in the API based on the user's language.

## Working conventions

- Make small, focused changes; keep the API and front-end working after each one.
- Don't add dependencies without a reason; prefer well-maintained, widely used crates and packages and mention new ones in your summary.
- Never commit secrets. Configuration comes from environment variables; keep a documented `.env.example` per project.
- Write tests alongside features: unit/integration tests in `api/`, component tests in `web/`.
- Commits: imperative, concise subject line. Don't commit unless asked.
- Code style: formatters and linters use their recommended defaults with a 4-space indent (rustfmt, Prettier) and single quotes in the web code. Run `./format.sh` and `./lint.sh` before declaring a change done. `mocks/` and Markdown files are not auto-formatted.
- Verify before declaring done: run the relevant build, lint and test commands (see the project `CLAUDE.md` files) and report real results.

## Commands

Run from the repository root unless noted.

```bash
# Lint and format (both projects; optionally pass `api` or `web`)
./lint.sh
./format.sh            # rewrites files
./format.sh --check    # verify only

# Full local stack
docker compose up --build

# API
cd api && cargo run            # start the API
cd api && cargo test           # tests
cd api && cargo clippy --all-targets -- -D warnings
cd api && cargo fmt --check

# Web
cd web && npm run dev          # dev server
cd web && npm test             # tests
cd web && npm run lint
cd web && npm run typecheck
```
