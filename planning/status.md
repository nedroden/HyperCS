# Status and handoff

**Read this first.** It says where the project stands, how to verify it, and what to do next.
Last updated: 2026-10-07.

## One-paragraph summary

The MVP is implemented in both projects and works end to end against the Docker Compose stack.
All of it is **uncommitted in the working tree** (no commits, branches or pushes were made on
request: review the diff, then commit). Automated checks are green. A manual browser pass covered
guest, member, moderator and admin, in English and German. What remains is polish, deeper tests,
a German proofread and the items under "Next steps".

## What exists

- `api/`: Rust workspace (`domain`, `db`, `api`, `server`). Auth, sessions, CSRF, roles, forum,
  moderation, edit history, CMS, stats, OpenAPI. See `api/CLAUDE.md`.
- `web/`: React + TypeScript SPA (Feature-Sliced Design). Every screen from `mocks/`, plus admin,
  settings, profile, history and 404. See `web/CLAUDE.md`.
- General site settings (admin, `/admin/settings`; `GET`/`PUT /api/v1/cms/site`, stored in `cms_content`
  under key `site`): site title (header logo alt text, browser tab), slogan (homepage hero; replaces the old
  hero intro text, which is used as the slogan until the first save) and default language (visitors who have
  not chosen one, and new accounts). Not covered by the mocks or the requirements file.
- `mocks/`: static mocks for every screen (the visual reference).
- `planning/`:
  - `requirements.md`: what to build (R-* numbers, resolved questions, known gaps).
  - `mvp-plan.md`: data model, endpoint list, slice map, decisions made while building.
  - `mvp-brief.md`: the original brief (historical; its section 3 and 4 decisions still apply).
  - `status.md`: this file.

## How to verify (all passed at the time of writing)

```bash
./format.sh --check && ./lint.sh
(cd api && cargo test --workspace)          # 16 api + 23 domain + 1 server unit, 89 integration tests
(cd web && npm run typecheck && npm test && npm run build && npm run check:api)   # 199 tests
docker-compose up --build                   # web :5173, API :8080, Swagger UI :8080/api/v1/docs
```

Environment notes:
- This machine uses Colima and has no `docker compose` plugin: use `docker-compose`. Integration
  tests need `DOCKER_HOST=unix://$HOME/.colima/default/docker.sock TESTCONTAINERS_RYUK_DISABLED=true`.
- Local dev admin: `ADMIN_USERNAME`/`ADMIN_PASSWORD` in `docker-compose.yml` (development only).
- The Vite dev proxy rewrites Host, so the API needs `ALLOWED_ORIGINS=http://localhost:5173`
  (already set in compose and `api/.env.example`).
- `web/package.json` has an npm `overrides` entry because `openapi-typescript@7.13` declares a
  TypeScript 5 peer while the project uses TypeScript 6.
- `0001_init.sql` was edited in place (pre-release). An old dev DB volume needs
  `docker-compose down -v`.
- Never hand-edit `web/src/shared/api/generated/schema.ts`. After API changes run
  `cd web && npm run generate:api`; `npm run check:api` fails on drift.

## What was verified, and how

- **Automated:** everything in "How to verify".
- **Browser, against the real stack:**
  - Guest: browsing, admin-only content hidden, home page from the CMS.
  - Member: register, login, logout, create topic with Markdown preview, quote, reply, flood-control
    error (429), edit, history by direct URL, restore, settings (profile, password with other
    sessions revoked, preferences, active sessions with revoke), public profile and its 404.
  - Moderator: pin, lock, edit/delete others' posts, replying in a locked topic; no admin access.
  - Admin: users and roles, structure (create, reorder, delete), news, homepage editor, terms page,
    move topic (counters follow), hard-delete topic (counters and stats return).
  - Language: footer switcher and Preferences stay in sync; guests use `localStorage`; the profile
    language is applied on login.
  - Mobile (375px): no horizontal overflow on the main routes and admin screens (one bug fixed).
- **Not covered:** see the next section.

## Known gaps and open items

- Account deletion (GDPR) is not implemented (`requirements.md` section 8).
- Not manually verified: a full German proofread of every screen; a keyboard-only walkthrough; the
  `/admin/users` mobile fix was confirmed live but not re-checked after the CSS change;
  the admin "move/delete board" paths beyond what the integration tests cover.
- The Testcontainers matrix was meant to be extended later. 89 tests exist and pass; extending
  them (more per-endpoint authorization cases, session expiry edge cases) is optional follow-up.
- Deleting a board while someone replies in one of its topics can deadlock; Postgres aborts one
  request with a 500. Rare, accepted.
- The seed creates no news items, so the home page starts with an empty "Latest news".
- Small UX polish: settings status messages accumulate across forms (a stale "session revoked"
  stays visible after saving another form); the sessions list does not refetch on its own when
  another device signs in.
- The hero button on the home page links to `/register` for guests (the API stores only a label).
- `web/test/utils.tsx` has 4 oxlint `only-export-components` warnings (helpers exported next to
  `renderApp`); harmless, can be split into two files.
- "Powered by HyperCS" in the footer is deliberately hard-coded English (never translated).

## Fixed after the first handoff

- Navigating forum <-> admin by clicking links crashed with "something went wrong" (reload fixed it):
  two queries shared the `['categories']` cache key with different data shapes. Fixed in
  `features/manage-structure` (`select`), with a regression test, and the rule is now in `web/CLAUDE.md`.
  Lesson: the dev container's bind mount can serve stale modules after quick edit/revert cycles;
  `docker-compose restart web` clears it.

## Next steps (suggested order)

1. Review and commit the working tree (it is one large uncommitted change; consider splitting by
   area: api, web, mocks/planning, docs).
2. German proofread and a keyboard-only accessibility walkthrough; fix what they find.
3. Fix the small UX polish items above; seed a sample news item.
4. Extend the integration test matrix as wanted.
5. Plan account deletion / personal data export (GDPR).
6. Out of scope for the MVP and so far unplanned: rate limiting, email (verification, notifications;
   the preferences are stored but nothing is sent), avatar upload, search, report post, CI, production
   images.
7. Deployment.

## Rules of thumb for the next agent

- Requirements first (`requirements.md`); update it in the same change as any behaviour change.
- Mocks are the visual reference; reuse tokens in `web/themes/default/tokens.css`, no new styles.
- The API owns the contract: change Rust, regenerate the client, never hand-write request types.
- Keep `./format.sh --check`, `./lint.sh`, the Rust tests and the web checks green.
- Do not commit unless asked.
