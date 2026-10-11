# HyperCS Web (React)

React single-page front-end for HyperCS. This file defines how it is built. See the root `CLAUDE.md` for product-wide rules and `planning/requirements.md` for what to build.

## Intended stack

Defaults to use when scaffolding. If you want to deviate, say why first.

- React 18+ with TypeScript in `strict` mode. No `any` without a comment explaining why.
- Build tool: Vite.
- Routing: React Router.
- Server state: TanStack Query (no hand-rolled fetch-in-useEffect). Local UI state with React state/context; avoid a global store unless clearly needed.
- API client and types: generated from the API's OpenAPI document into `src/shared/api/generated/`. Never edit generated files; wrap them in `shared/api/` if needed. Entity-specific requests live in the entity's `api/` segment.
- i18n: `react-i18next` (or equivalent) with JSON message catalogs for `en-US` and `de`. `en-US` is the default and fallback.
- Styling: plain CSS (CSS modules) using design tokens taken from the mocks. No UI component library; the look comes from `mocks/`.
- Testing: Vitest + React Testing Library; Playwright for a small set of end-to-end flows.
- Lint: oxlint (recommended rules) plus `steiger` (the FSD architecture linter), both run by `npm run lint`. Format: Prettier with default options except `tabWidth: 4` and `singleQuote: true` (`npm run format`, `npm run format:check`). Prefer the root scripts `./lint.sh web` and `./format.sh web`.

## Architecture: Feature-Sliced Design

The code in `src/` follows [Feature-Sliced Design](https://feature-sliced.design) (FSD). Layers, from highest to lowest:

| Layer       | Purpose                                                                 | Example here                    |
| ----------- | ----------------------------------------------------------------------- | ------------------------------- |
| `app/`      | App-wide setup: providers, routing, layouts, global styles              | `App`, `MainLayout`, `tokens.css` |
| `pages/`    | One slice per route/screen; composes lower layers                       | `pages/home`                    |
| `widgets/`  | Large self-contained UI blocks reused across pages                      | `header`, `footer`              |
| `features/` | User actions with business value (verbs)                                | `switch-language`               |
| `entities/` | Business concepts (nouns): their data, API calls and basic UI           | `api-health`                    |
| `shared/`   | Reusable code with no business logic: API client, i18n, UI kit, helpers | `shared/api`, `shared/ui/Flag`   |

(`app` and `shared` have no slices, only segments.) Rules:

- **Import direction:** a module may only import from layers *below* its own. Never import sideways between slices of the same layer, and never upwards. Enforced by `steiger` (part of `npm run lint`).
- **Public API:** every slice exposes what others may use through its `index.ts`. Other slices import only from that (`@/features/switch-language`), never from internal paths. Inside a slice use relative imports.
- **Segments** inside a slice are named by purpose: `ui/` (components, styles), `model/` (state, hooks, business logic), `api/` (requests), `lib/` (helpers), `config/` (constants). Don't name segments by type (`components`, `hooks`, `types`).
- Put code in the lowest layer where it makes sense; promote it upwards (or extract into a lower layer) only when needed. Don't create entities or features speculatively. Ask "is this a noun with its own data (entity), a user action (feature), or just page content (keep it in the page)?".
- **Styles live in a theme folder, not next to components.** All CSS is in `web/themes/<theme>/` (outside `src/`) (currently only `default/`), one file per component or area, pulled together by that theme's `index.css`; `themes/index.ts` selects the active theme. Components never import CSS; they only use class names (keep them unique and descriptive, e.g. `site-header`, `lang-menu`). A new theme provides the same files, or overrides the CSS variables from `tokens.css`. Every color, font and gradient must come from `tokens.css` variables so themes can restyle without touching components. When adding a component, add its stylesheet to each theme and list it in the theme's `index.css`.
- Flags are SVG files in `shared/ui/Flag/flags/` (from the MIT-licensed `flag-icons` set, license included), rendered by the `Flag` component. To add a language: drop in `<code>.svg`, register it in `Flag.tsx`, and add it to `shared/i18n/languages.ts`. Don't draw flags in CSS.
- **Theme tokens:** no literal colors, gradients, fonts, shadows or logo/header sizes in component stylesheets; add a token to `tokens.css` first. Focus styles use `--focus-ring`. Utility classes in `base.css`: `.sr-only`, `.skip-link`, `.btn.secondary`.
- Use the `@/` alias for cross-slice imports (`@/shared/api`).
- **Translations are split by area:** `shared/i18n/locales/<lang>/<namespace>.json` (`common`, `errors`, `auth`, `forum`, `settings`, `admin`, `cms`, `profile`; add a new file for a new area, in both `en-US/` and `de/`). Each file owns its own top-level keys and all files of a language are merged, so keys stay `forum.newTopic` (no `ns:` prefix). Feature work edits only its own namespace file(s); `common.json` and `errors.json` are shared (small, append-only edits). A test enforces that no top-level key lives in two files, identical key sets across languages and `_one`/`_other` pairs. Plurals use i18next `_one`/`_other` keys (en-US and de share the same categories). API error codes map to `errors.<code>`; add a key for every code the API can return. German uses informal "du".
- **Forms:** react-hook-form + zod. Validation messages in zod schemas and API field errors are i18n *keys* (`errors.required`, `errors.<code>`), never literal text. Use `Field` (label, hint, error, `aria-invalid`/`aria-describedby`; spread the render-prop argument onto the control), `FormErrorSummary` (`role="alert"`) and `applyProblemToForm(problem, setError, knownFields)` from `shared/lib` to map the API's `errors[{field, code}]` onto fields. Use `Controller` + `MarkdownEditor` for post bodies.
- **User content:** render Markdown only through `shared/ui` `Markdown` (react-markdown + rehype-sanitize; links get `rel="noopener nofollow ugc"`, raw HTML is not rendered, images become links). Body length limit: `MARKDOWN_MAX_LENGTH` in `shared/config`.
- **Times:** show timestamps with `RelativeTime` (relative text, absolute time in `title`). Wrap the app in `TimeZoneProvider` with the user's IANA time zone once settings exist; the default is UTC.
- **States:** use `Loading`, `EmptyState`, `ErrorState` from `shared/ui` for every data-driven screen. `app/lib/ErrorBoundary` catches render errors; the `*` route renders `pages/not-found`.
- `Pagination` takes `current`, `total` and optional `getHref(page)` (default `?page=N`) and renders real links.
- **Routes are all registered up front** in `app/App.tsx` (lazy page per route; auth/admin groups behind `RequireAuth` / `RequireRole` from `app/guards`). Feature work only fills in its `pages/<name>` slice and keeps the exported component name; do not edit `App.tsx`. Pages that exist only as placeholders: new-topic, edit-post, post-history, profile, terms, admin-*.
- **Session:** `useSession()` from `entities/session` (`status`: loading | guest | authenticated | error, `user`: `MeResponse`, `isAdmin`, `isModerator`, `refresh()`), backed by `GET /me` in the Query cache. It also installs the CSRF token (state-changing requests get `x-csrf-token` automatically). After login call `useSessionTransitions().afterLogin()` (refetches `/me` and everything else), after logout `afterLogout()`. Guests keep their language in localStorage; a logged-in user's profile language is applied on login, and the language switcher writes both.
- **API data:** query keys come only from `shared/api/queryKeys.ts` (`[resource, ...scope, params]`; add your resource there, append-only). Requests go through the typed `api` client wrapped in `unwrap(...)` (throws `ApiError` with the problem+json); mutations use `useApiMutation({ mutationFn, invalidates: [queryKeys.x.all] })`; map errors to forms with `problemOf(error)` + `applyProblemToForm`. Types come from `Schema<'Name'>`. Limits shared with the API: `LIMITS` in `shared/config`. **One query key = one cached data shape**: when two screens read the same endpoint under the same key, both `queryFn`s must return the raw response, and a screen that wants a part of it uses `select` (a mismatch crashes only on client-side navigation and disappears after a reload).
- **Tests:** `test/setup.ts` initializes i18n (en-US) for every test, clears localStorage, resets the language and fails a test whose request had no mock route. Use `test/utils.tsx`: `renderApp(ui, { route, me, mock })` renders with Providers + MemoryRouter + a fresh QueryClient (`me` null = guest, a `makeMe({...})` = logged in, `'error'` = `/me` fails) and returns `{ mock, user, queryClient, ...RTL }`. `mockApi()` (`test/mockApi.ts`) stubs `fetch`: `mock.on('get', '/api/v1/stats', 200, {...})` is typed from the generated schema (path, method, status, body); `mock.onUnlisted(...)` for statuses the schema does not list; `mock.calls(method, path)` returns recorded requests (`body`, `headers`, `params`, `search`); `problem(422, 'validation_failed', [{ field, code }])` builds an error body. Register routes before rendering when the first request matters; later registrations win. Wait for async UI with `findBy*`/`waitFor`. Tests of a slice must not import from a higher layer (steiger): test pages in `pages/*`.
- Mapping from the planned domain: Forum categories/boards/topics/posts and users become `entities/*`; Post reply, Login, Register, Edit profile become `features/*`; Board/Topic lists and the settings form blocks become `widgets/*` or stay in their `pages/*` until reused.

```
web/src/
  main.tsx
  app/        # App.tsx, layouts/, lib/ (providers)
  pages/      # one slice per route (see App.tsx); several are placeholders until built; not-found
  widgets/    # header/, footer/
  features/   # switch-language/, sign-in/, sign-up/, sign-out/
  entities/   # api-health/, session/ (GET /me state, login/logout/preferences requests)
  shared/     # api/, config/, i18n/, lib/ (relative time, time zone, API problem -> form errors), ui/ (Avatar, Breadcrumbs, Flag, Form, FormBox, Markdown, MarkdownEditor, Pagination, RelativeTime, states)

web/themes/   # all CSS, outside src (default/...); selected in themes/index.ts
web/test/     # setup.ts, utils.tsx (renderApp), mockApi.ts
```

## UI source of truth: the mocks

`mocks/*.html` is the design reference. Each page maps to a route:

| Mock                | Route (suggested)                |
| ------------------- | -------------------------------- |
| `home.html`         | `/`                              |
| `login.html`        | `/login`                         |
| `register.html`     | `/register`                      |
| `settings.html`     | `/settings`                      |
| `board-index.html`  | `/forum`                         |
| `board.html`        | `/forum/boards/:boardId`         |
| `topic.html`        | `/forum/topics/:topicId`         |

- Extract the CSS custom properties/values the mocks repeat (colors, gradients, spacing, Roboto font) into `themes/default/tokens.css` once and reuse them.
- The shared header, nav bar and footer (with language dropdown) live in `widgets/header` and `widgets/footer`, composed by `app/layouts/MainLayout`; don't duplicate them per page.
- Match the mocks' responsive behavior (e.g. the 700px breakpoint).
- Show different header/menu content for logged-in vs logged-out users (see root `CLAUDE.md`).

## Rules

- No hard-coded user-facing strings: everything goes through i18n, with keys in both `en-US` and `de`. The one deliberate exception is the footer credit "Powered by HyperCS | © 2026 Robert Monden" (`widgets/footer`), which is a brand line that must never be translated. Use ICU/plural forms for counts ("1 topic" / "2 topics") and relative dates via `Intl.RelativeTimeFormat`.
- Language switcher: menu button in the footer with flags (`aria-expanded`, arrow keys/Home/End, Escape closes and returns focus, closes on outside click and Tab). Persist the choice (user setting when logged in, otherwise local storage).
- Accessibility: semantic HTML, labels for every form control, visible focus states, sufficient contrast, usable by keyboard only.
- Never render user-generated content as raw HTML. Render it as text, or sanitize if formatting is added later.
- Never trust the UI for authorization. Hide actions the user can't perform (e.g. Edit/Delete), but the API is the real gate.
- Forms: validate client-side for UX, show server field errors from the API's error shape.
- Keep components small and typed; props interfaces are explicit. Prefer composition over configuration flags.
- Config via `import.meta.env` (`VITE_*`); document in `.env.example`. No secrets in the front-end.

## Testing

- Component tests for forms, pagination, the language switcher and permission-dependent UI.
- E2E for: register → login → post a reply → change settings.
- Before finishing a change, all of these must pass:

```bash
npm run typecheck
npm run lint
npm test
```
