//! Validation limits used by requests, the OpenAPI schema and the generated
//! zod schemas on the front-end. Defined once here; mirror changes in
//! `web/src/shared/api` validation if the generator does not pick them up
//! automatically from OpenAPI `minLength`/`maxLength`.

pub const USERNAME_MIN: u64 = 3;
pub const USERNAME_MAX: u64 = 32;
pub const DISPLAY_NAME_MIN: u64 = 1;
pub const DISPLAY_NAME_MAX: u64 = 50;
pub const EMAIL_MAX: u64 = 254;
pub const PASSWORD_MIN: u64 = 10;
pub const PASSWORD_MAX: u64 = 128;
pub const TOPIC_TITLE_MIN: u64 = 3;
pub const TOPIC_TITLE_MAX: u64 = 120;
pub const POST_BODY_MIN: u64 = 1;
pub const POST_BODY_MAX: u64 = 10_000;
pub const CATEGORY_BOARD_NAME_MIN: u64 = 1;
pub const CATEGORY_BOARD_NAME_MAX: u64 = 60;
pub const DESCRIPTION_MAX: u64 = 200;
pub const ABOUT_ME_MAX: u64 = 500;
pub const WEBSITE_MAX: u64 = 200;
pub const NEWS_TITLE_MIN: u64 = 1;
pub const NEWS_TITLE_MAX: u64 = 120;
pub const NEWS_BODY_MAX: u64 = 5_000;

/// Topics listed per board page.
pub const TOPICS_PER_PAGE: u32 = 20;
/// Posts listed per topic page.
pub const POSTS_PER_PAGE: u32 = 15;
/// Hard cap on `perPage` regardless of what the client requests.
pub const MAX_PAGE_SIZE: u32 = 100;

/// Idle expiry for a non-"remember me" session.
pub const SESSION_IDLE_HOURS: i64 = 24;
/// Lifetime of a "remember me" session.
pub const SESSION_REMEMBER_DAYS: i64 = 30;

/// Minimum time between two posts by the same member.
pub const MIN_POST_INTERVAL_SECS: i64 = 5;
/// Minimum time between posts for accounts younger than `NEW_ACCOUNT_AGE_MINS`.
pub const NEW_ACCOUNT_POST_INTERVAL_SECS: i64 = 30;
pub const NEW_ACCOUNT_AGE_MINS: i64 = 10;
/// Window in which an identical consecutive post by the same user is rejected.
pub const DUPLICATE_POST_WINDOW_SECS: i64 = 120;

/// A user counts as online if seen within this many minutes.
pub const ONLINE_WINDOW_MINUTES: i64 = 5;
/// `last_seen_at` is updated at most this often per session.
pub const LAST_SEEN_THROTTLE_MINUTES: i64 = 1;

/// Default global request body size limit (bytes); some routes override it.
pub const DEFAULT_BODY_LIMIT_BYTES: usize = 64 * 1024;

/// Usernames reserved regardless of case.
pub const RESERVED_USERNAMES: &[&str] = &[
    "admin",
    "administrator",
    "root",
    "system",
    "api",
    "moderator",
    "mod",
    "support",
    "staff",
    "help",
    "null",
    "undefined",
    "hypercs",
];
