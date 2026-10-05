//! Origin/CSRF validation as pure functions so they can be unit tested
//! without spinning up the whole extractor stack.

/// `origin` is the request's `Origin` header value (e.g. `https://example.com`),
/// `host` is the request's `Host` header (`example.com` or `example.com:8080`).
/// When `allowed_origins` is empty, same-origin deployment is assumed: the
/// origin's host (scheme stripped) must equal `host`.
pub fn origin_is_allowed(
    origin: Option<&str>,
    host: Option<&str>,
    allowed_origins: &[String],
) -> bool {
    let Some(origin) = origin else { return false };
    if allowed_origins.iter().any(|o| o == origin) {
        return true;
    }
    if !allowed_origins.is_empty() {
        return false;
    }
    let origin_host = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"))
        .unwrap_or(origin);
    match host {
        Some(host) => origin_host == host,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_origin_allowed_when_allow_list_empty() {
        assert!(origin_is_allowed(
            Some("https://hypercs.example"),
            Some("hypercs.example"),
            &[]
        ));
        assert!(origin_is_allowed(
            Some("http://localhost:5173"),
            Some("localhost:5173"),
            &[]
        ));
    }

    #[test]
    fn cross_origin_rejected_when_allow_list_empty() {
        assert!(!origin_is_allowed(
            Some("https://evil.example"),
            Some("hypercs.example"),
            &[]
        ));
    }

    #[test]
    fn missing_origin_rejected() {
        assert!(!origin_is_allowed(None, Some("hypercs.example"), &[]));
    }

    #[test]
    fn explicit_allow_list_checked_verbatim() {
        let allowed = vec!["https://hypercs.example".to_string()];
        assert!(origin_is_allowed(
            Some("https://hypercs.example"),
            Some("other-host"),
            &allowed
        ));
        assert!(!origin_is_allowed(
            Some("https://not-allowed.example"),
            Some("hypercs.example"),
            &allowed
        ));
    }
}
