# HyperCS – Requirements

Derived from the HTML mockups in `mocks/`. Mockup content (names, counts, dates, text) is placeholder data; the requirements below describe the behaviour the mockups imply.

Pages covered: `home.html`, `login.html`, `register.html`, `settings.html`, `board-index.html`, `board.html`, `topic.html`.

## 1. Overview

HyperCS is a web application combining a small CMS (static-style content pages) with a discussion forum, sharing one account system, layout and navigation.

## 2. Global layout and navigation

- **R-G1** Every page uses the same layout: header, main navigation bar, content area, footer.
- **R-G2** The header shows the site name. On forum pages it reads "HyperCS Demo Forum" (the name is configurable); on CMS and account pages it reads "HyperCS".
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
- **R-F5** Each board row shows its last post: topic title (linked), author (linked) and relative time. Boards without topics show no last-post info.
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

## 6. Open questions

- Is the forum name ("Demo Forum") a site setting, and should it also appear on CMS pages?
- Which actions require login (viewing, replying, creating topics)? The mockups only show the logged-in view of the forum.
- How are pinned/locked statuses, and moderation actions (delete, move, lock), exposed in the UI? Not mocked yet.
- Public profile pages, member list and search are implied by links but not mocked.
- Settings: which time zones are supported, whether the chosen language is remembered per user account (versus per browser), how the default language is picked for new visitors, and whether an email change requires re-verification.
- Post formatting: plain text only, or markup/BBCode/Markdown? Not specified.
- How is the CMS homepage edited (admin UI, not mocked)?
- Terms of service page content and email verification on registration are not specified.
