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
- Use the `@/` alias for cross-slice imports (`@/shared/api`).
- Translations stay in `shared/i18n/locales/*.json`; add keys for both languages.
- Mapping from the planned domain: Forum categories/boards/topics/posts and users become `entities/*`; Post reply, Login, Register, Edit profile become `features/*`; Board/Topic lists and the settings form blocks become `widgets/*` or stay in their `pages/*` until reused.

```
web/src/
  main.tsx
  app/        # App.tsx, layouts/, lib/ (providers)
  pages/      # home, login, register, settings, forum-index, board, topic (placeholder content)
  widgets/    # header/, footer/
  features/   # switch-language/
  entities/   # api-health/, session/ (placeholder login state)
  shared/     # api/, i18n/, lib/ (relative time), ui/ (Avatar, Breadcrumbs, Flag, FormBox, Pagination)

web/themes/   # all CSS, outside src (default/...); selected in themes/index.ts
web/test/     # test setup
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
