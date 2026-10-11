//! Validation limits, pagination sizes and abuse-control defaults. Defined
//! once here: the request DTOs, the OpenAPI schema (via `utoipa` attributes)
//! and the services all read these constants, and a test in `hypercs-api`
//! asserts the generated OpenAPI document carries the same numbers.

pub const USERNAME_MIN: u64 = 3;
pub const USERNAME_MAX: u64 = 32;
pub const DISPLAY_NAME_MIN: u64 = 1;
pub const DISPLAY_NAME_MAX: u64 = 50;
pub const EMAIL_MAX: u64 = 254;
pub const PASSWORD_MIN: u64 = 10;
pub const PASSWORD_MAX: u64 = 128;
pub const PERSON_NAME_MAX: u64 = 60;
pub const LOCATION_MAX: u64 = 100;
pub const TIMEZONE_MAX: u64 = 64;
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
pub const HERO_HEADING_MAX: u64 = 120;
pub const SITE_TITLE_MAX: u64 = 60;
pub const SLOGAN_MAX: u64 = 300;
pub const HERO_BUTTON_MAX: u64 = 60;
pub const FEATURE_TITLE_MAX: u64 = 60;
pub const FEATURE_TEXT_MAX: u64 = 200;
pub const FEATURE_COUNT: u64 = 3;
pub const TERMS_BODY_MAX: u64 = 20_000;

/// Topics listed per board page.
pub const TOPICS_PER_PAGE: u32 = 20;
/// Posts listed per topic page.
pub const POSTS_PER_PAGE: u32 = 15;
pub const NEWS_PER_PAGE: u32 = 10;
pub const USERS_PER_PAGE: u32 = 20;
pub const REVISIONS_PER_PAGE: u32 = 20;
/// Hard cap on `perPage` regardless of what the client requests.
pub const MAX_PAGE_SIZE: u32 = 100;

/// Idle expiry for a non-"remember me" session.
pub const SESSION_IDLE_HOURS: i64 = 24;
/// Lifetime of a "remember me" session.
pub const SESSION_REMEMBER_DAYS: i64 = 30;
/// Longest user-agent string kept as a device hint.
pub const USER_AGENT_MAX: usize = 255;

/// Default minimum time between two posts by the same member.
pub const MIN_POST_INTERVAL_SECS: i64 = 5;
/// Default minimum time between posts for accounts younger than
/// [`NEW_ACCOUNT_AGE_MINS`].
pub const NEW_ACCOUNT_POST_INTERVAL_SECS: i64 = 30;
pub const NEW_ACCOUNT_AGE_MINS: i64 = 10;
/// Default window in which an identical consecutive post by the same user is
/// rejected.
pub const DUPLICATE_POST_WINDOW_SECS: i64 = 120;

/// A user counts as online if seen within this many minutes.
pub const ONLINE_WINDOW_MINUTES: i64 = 5;
/// At most this many users are listed by name as online.
pub const ONLINE_USERS_LIMIT: i64 = 50;
/// `last_seen_at` is updated at most this often per session.
pub const LAST_SEEN_THROTTLE_MINUTES: i64 = 1;

/// Default global request body size limit (bytes); some routes override it.
pub const DEFAULT_BODY_LIMIT_BYTES: usize = 64 * 1024;
/// Body limit for the small auth forms.
pub const AUTH_BODY_LIMIT_BYTES: usize = 4 * 1024;
/// Body limit for the terms-of-service PUT (up to 20,000 characters).
pub const TERMS_BODY_LIMIT_BYTES: usize = 128 * 1024;

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
