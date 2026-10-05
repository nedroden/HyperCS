use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{DateTime, Utc};
use hypercs_domain::constants::{SESSION_IDLE_HOURS, SESSION_REMEMBER_DAYS};
use time::Duration as TimeDuration;

use crate::auth::extractor::SESSION_COOKIE;

/// Builds the `Set-Cookie` header value for a newly created session.
/// `remember` extends the cookie's lifetime to 30 days; otherwise it is a
/// browser-session cookie backed by a 24h idle expiry enforced server-side.
pub fn build_session_cookie(token: String, remember: bool, secure: bool) -> Cookie<'static> {
    let mut cookie = Cookie::new(SESSION_COOKIE, token);
    cookie.set_http_only(true);
    cookie.set_secure(secure);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    if remember {
        cookie.set_max_age(TimeDuration::days(SESSION_REMEMBER_DAYS));
    }
    cookie
}

pub fn build_logout_cookie(secure: bool) -> Cookie<'static> {
    let mut cookie = Cookie::new(SESSION_COOKIE, "");
    cookie.set_http_only(true);
    cookie.set_secure(secure);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    cookie.set_max_age(TimeDuration::seconds(0));
    cookie
}

pub fn session_expiry(remember: bool, now: DateTime<Utc>) -> DateTime<Utc> {
    if remember {
        now + chrono::Duration::days(SESSION_REMEMBER_DAYS)
    } else {
        now + chrono::Duration::hours(SESSION_IDLE_HOURS)
    }
}
