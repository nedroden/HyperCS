# HyperCS – Requirements

Derived from the HTML mockups in `mocks/`. Mockup content (names, counts, dates, text) is placeholder data; the requirements below describe the behaviour the mockups imply.

Pages covered: `home.html`, `login.html`, `register.html`, `settings.html`, `board-index.html`, `board.html`, `topic.html`.

## 1. Overview

HyperCS is a web application combining a small CMS (static-style content pages) with a discussion forum, sharing one account system, layout and navigation.

## 2. Global layout and navigation

- **R-G1** Every page uses the same layout: header, main navigation bar, content area, footer.
- **R-G2** The header shows the HyperCS logo (full logo, 48px high) at the top left on every page, including forum pages. The logo links to the homepage and has "HyperCS" as its alt text. The forum no longer shows a separate "Demo Forum" title in the header.
- **R-G3** The header contains a user box on the right:
  - Logged in: "Hello, **{display name}** | Logout", where the display name links to the settings page.
  - Logged out: "Login | Register".
- **R-G4** The main navigation contains:
  - Logged in: Home, Forum, Logout.
  - Logged out: Home, Forum, Login.
- **R-G5** The footer shows "Powered by HyperCS | © {year} {owner}" and links the software name.
- **R-G6** Forum pages show a breadcrumb trail (Board index › Category › Board › Topic) with every level except the current page linked.
- **R-G7** List pages (topics in a board, posts in a topic) are paginated, with previous/next controls, numbered pages, a highlighted current page, and a disabled previous control on the first page.
- **R-G8** The layout is responsive: multi-column blocks collapse on narrow screens (statistics grid to 2 columns, feature blocks and post layout to a single column below 700px).
- **R-G10** The software supports two languages: English (US) and German (Deutsch). English (US) is the default.
- **R-G11** The footer contains a language switcher on the right: a dropdown menu (opening upwards) whose button shows the flag and name of the current language. The menu lists English (US) 🇺🇸 and Deutsch 🇩🇪 with flag icons and marks the active one with a check mark. It closes on outside click or Escape. Selecting a language keeps the user on the equivalent page.
- **R-G12** All user-facing text (labels, buttons, menus, relative dates, singular/plural counts) is translated. User-generated content (posts, topic titles) is not translated.
- **R-G9** Pages use a consistent visual style (Roboto font, blue gradient navigation bar, grey section headers, blue links and primary buttons).

## 3. CMS

### 3.1 Homepage
- **R-C1** A homepage is available at the site root and is the "Home" menu target.
- **R-C2** The homepage contains a hero banner with a heading, introductory text and a call-to-action button.
- **R-C3** The homepage contains a row of three feature blocks (title and short text each).
- **R-C4** The homepage contains a "Latest news" list; each item has a linked title, a relative date and a short excerpt.
- **R-C5** Apart from the main menu link, the homepage makes no reference to the forum.
- **R-C6** Homepage content (hero, features, news) must be manageable by an administrator rather than hard-coded. *(Implied by "CMS functionality"; the mockup only shows dummy content.)*

## 4. Accounts

### 4.1 Registration
- **R-A1** Visitors can register with: username, email address, password, password confirmation.
- **R-A2** Registration requires accepting the terms of service (checkbox with a link to the terms).
- **R-A3** The registration page links to the login page for existing users.

### 4.2 Login
- **R-A4** Users can log in with username and password.
- **R-A5** A "Remember me" option keeps the user logged in across sessions.
- **R-A6** The login page links to the registration page.
- **R-A7** There is **no** password reset / "forgot password" function (explicitly removed).

### 4.3 Logout
- **R-A8** Logged-in users can log out from the header user box and the main menu.

### 4.4 Roles and display
- **R-A9** Users have a role. Administrators are shown with an "Admin" badge on their posts.
- **R-A10** The login and registration forms are displayed in a centred box, with the page heading left-aligned within the box.

### 4.5 User settings
- **R-A11** Logged-in users have a settings page, reached via their name in the header user box. It has a sidebar (Profile, Account, Password, Preferences) that jumps to the matching section; on narrow screens the sidebar stacks above the content.
- **R-A12** Each section is a separate form with its own save button.
- **R-A13** **Profile** (visible to other members): avatar (upload, remove; JPG/PNG, max. 1 MB), display name, location, website, "About me".
- **R-A14** **Account** (never shown publicly): first name, last name, email address, and the username shown read-only (usernames cannot be changed).
- **R-A15** **Change password**: requires the current password plus a new password and its confirmation.
- **R-A16** **Preferences**: language (English (US) or Deutsch, kept in sync with the footer switcher), time zone, email notifications (replies to own topics, quotes of own posts) and a privacy option to show or hide the user in the list of users online.

## 5. Forum

### 5.1 Structure
- **R-F1** Content is organised as: Categories → Boards → Topics → Posts.
- **R-F2** Categories and boards each have a name and a description.

### 5.2 Board index
- **R-F3** The board index lists all categories, each with its name, description and boards.
- **R-F4** Each board row shows: linked name, description, topic count, post count.
- **R-F5** Each board row shows its last post: the avatar of the user who made the last reply, the topic title (linked), the author (linked) and the relative time. The avatar is round, vertically centered next to the text, and falls back to the author's initial on a colored background when the user has no uploaded picture. Boards without topics show no last-post info.
- **R-F6** Counts use correct singular/plural wording ("1 topic", "0 topics", "1 post").
- **R-F7** The board index ends with a Statistics section, visually distinct from category headers (blue gradient header) and spaced apart from the last category.
- **R-F8** Statistics show totals for Topics, Posts, Members and Boards.
- **R-F9** Statistics also show: newest member, latest post (topic, author, time) and the users currently online.

### 5.3 Board page (topic list)
- **R-F10** Shows the board title, description, breadcrumbs and a "New topic" button.
- **R-F11** Lists topics in columns: Topic, Replies, Views, Last post.
- **R-F12** Each topic row shows a linked title, the starter and relative start time, reply count, view count, and last poster with relative time.
- **R-F13** Topics can be flagged **Pinned** or **Locked**, shown as a badge before the title.
- **R-F14** The topic list is paginated.

### 5.4 Topic page (post list)
- **R-F15** Shows the topic title, "Started by {author} · {time} · {n} posts", breadcrumbs and a "Post reply" button that jumps to the reply form.
- **R-F16** Lists posts in order. Each post has an author panel (avatar, name linked to profile, role badge, post count, join date) and the post body.
- **R-F17** Each post shows a title (original or "Re: …"), relative time and post number (permalink).
- **R-F18** Post actions: Quote and Report for other users' posts; Edit and Delete for the author and administrators.
- **R-F19** The post list is paginated.
- **R-F20** A reply form (text area and Submit button) is shown at the bottom of the topic.

## 6. Resolved questions (MVP decisions)

These were open questions; they are now decided (see `planning/mvp-prompt.md` section 3/4 and `planning/mvp-plan.md` for the full rationale) and override anything above where they differ.

- The forum name shown in `<title>`/header is not a separate site setting for the MVP; the header never shows a forum title, only the HyperCS logo (R-G2). CMS pages never mention the forum except the "Forum" menu link.
- Guests may read the forum (boards, topics, posts) but not reply, create topics, quote, or see admin-only categories/boards. Writing requires an account (R-F21).
- Moderation actions (pin/unpin, lock/unlock, move, delete) are exposed as post/topic-level controls, visible only to moderators/admins; "Move topic" opens a small modal with a destination-board dropdown (`mocks/move-topic.html`).
- Public profile pages exist at `/forum/users/{username}` (`mocks/profile.html`); there is no member list or search in the MVP (out of scope, see section 7).
- Time zones: the full IANA tz database, defaulting to `UTC` for new accounts (the browser's detected zone is pre-selected at registration client-side, not enforced). Language is a single source of truth stored per-account once logged in, `localStorage` for guests (R-A16a). Email changes are not re-verified in the MVP (no email sending at all).
- Post formatting is Markdown (CommonMark via `react-markdown`), sanitized on render, never raw HTML (R-F22).
- The CMS homepage is edited by admins in the React app (`mocks/admin-homepage.html`): hero, three feature blocks, and news items (create/edit/delete).
- Terms of service page (`mocks/terms.html`) is static CMS content, editable by admins, linked from registration. No email verification on registration (no email sending in the MVP).

## 7. New requirements (added while building the MVP)

- **R-F21** Guests can read all non-admin-only boards, topics and posts, but all write actions (new topic, reply, quote, edit, delete, moderate) require login; the UI replaces the reply form with a login/register prompt for guests (`mocks/board-index-guest.html`).
- **R-F22** Post and topic bodies are Markdown; they render sanitized (no raw HTML) with a live preview in the post/topic forms (`mocks/new-topic.html`, `mocks/edit-post.html`).
- **R-F23** Every post edit is kept as a revision; authors, moderators and admins can view a post's edit history and restore an older revision, which creates a new revision rather than deleting history (`mocks/post-history.html`).
- **R-F24** Moderators and admins can move a topic to a different board (`mocks/move-topic.html`) and pin/unpin or lock/unlock a topic; locked topics only accept replies from moderators/admins.
- **R-A17** A public profile page shows a member's display name, avatar initial, join date, post count and "About me" (`mocks/profile.html`).
- **R-A18** The Settings page has an "Active sessions" section listing signed-in devices with created/last-active times, a "this device" marker and a revoke action (`mocks/settings-sessions.html`).
- **R-ADM1** Admins have a structure-management screen to create/edit/delete/reorder categories and boards, including the admin-only flag (`mocks/admin-structure.html`).
- **R-ADM2** Admins have a users screen listing all members with a role dropdown per user (`mocks/admin-users.html`).
- **R-ADM3** Admins have a homepage/news screen to edit the hero, the three feature blocks, and create/edit/delete news items (`mocks/admin-homepage.html`).
- **R-G13** Unknown routes and generic errors show a dedicated page with the standard header/nav/footer (`mocks/404.html`).
- **R-G14** A static Terms of Service page is linked from registration and reachable at all times (`mocks/terms.html`).

## 8. Known gaps (deliberately out of MVP scope)

- **Account deletion** is not implemented. This is a GDPR-relevant gap to schedule for a future release (export/delete personal data on request).
- Rate limiting, email sending (including email verification and password reset), avatar upload, search, and report-post are all out of scope for the MVP (section 7 of `planning/mvp-prompt.md`).
