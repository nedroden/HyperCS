# HyperCS Web (React)

React single-page front-end for HyperCS. This project does not exist yet; this file defines how it should be built. See the root `CLAUDE.md` for product-wide rules and `planning/requirements.md` for what to build.

## Intended stack

Defaults to use when scaffolding. If you want to deviate, say why first.

- React 18+ with TypeScript in `strict` mode. No `any` without a comment explaining why.
- Build tool: Vite.
- Routing: React Router.
- Server state: TanStack Query (no hand-rolled fetch-in-useEffect). Local UI state with React state/context; avoid a global store unless clearly needed.
- API client and types: generated from the API's OpenAPI document into `src/api/generated/`. Never edit generated files; wrap them in `src/api/` if needed.
- i18n: `react-i18next` (or equivalent) with JSON message catalogs for `en-US` and `de`. `en-US` is the default and fallback.
- Styling: plain CSS (CSS modules) using design tokens taken from the mocks. No UI component library; the look comes from `mocks/`.
- Testing: Vitest + React Testing Library; Playwright for a small set of end-to-end flows.
- Lint/format: ESLint and Prettier.

## Layout

```
web/
  package.json
  vite.config.ts
  src/
    main.tsx
    app/            # router, providers, layout shell (header, nav, footer)
    api/            # client wrapper, generated types, query hooks
    i18n/           # setup and locale JSON files (en-US, de)
    features/       # one folder per area: cms, auth, forum, settings
    components/     # small shared UI pieces (Pagination, Breadcrumbs, LanguageSwitcher, ...)
    styles/         # tokens.css (colors, spacing, fonts) and global styles
```

Organize by feature, not by file type. Shared components must not import from a feature.

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

- Extract the CSS custom properties/values the mocks repeat (colors, gradients, spacing, Roboto font) into `styles/tokens.css` once and reuse them.
- The shared header, nav bar and footer (with language dropdown) live in one layout component; don't duplicate them per page.
- Match the mocks' responsive behavior (e.g. the 700px breakpoint).
- Show different header/menu content for logged-in vs logged-out users (see root `CLAUDE.md`).

## Rules

- No hard-coded user-facing strings: everything goes through i18n, with keys in both `en-US` and `de`. Use ICU/plural forms for counts ("1 topic" / "2 topics") and relative dates via `Intl.RelativeTimeFormat`.
- Language switcher: dropdown in the footer with flags, keyboard accessible, closes on outside click and Escape. Persist the choice (user setting when logged in, otherwise local storage).
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
