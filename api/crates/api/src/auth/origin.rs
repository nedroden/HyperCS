//! Origin/CSRF validation as pure functions so they can be unit tested
//! without spinning up the whole extractor stack.

/// Headers relevant to the same-origin decision.
#[derive(Debug, Default, Clone, Copy)]
pub struct OriginHeaders<'a> {
    /// `Origin`, e.g. `https://example.com`.
    pub origin: Option<&'a str>,
    /// `Host`, e.g. `example.com` or `localhost:8080`.
    pub host: Option<&'a str>,
    /// `X-Forwarded-Host`: set by reverse proxies that rewrite `Host`. A
    /// cross-site attacker cannot set it from a browser without a CORS
    /// preflight, which this API never grants for unlisted origins.
    pub forwarded_host: Option<&'a str>,
    /// `Sec-Fetch-Site` (`same-origin`, `same-site`, `cross-site`, `none`).
    pub sec_fetch_site: Option<&'a str>,
}

fn origin_host(origin: &str) -> &str {
    origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"))
        .unwrap_or(origin)
}

/// Decides whether an unsafe (state-changing) request comes from a trusted
/// origin:
///
/// - the `Origin` is in the explicit allow-list, or
/// - it names the same host the request was addressed to (`Host` or
///   `X-Forwarded-Host`) and the browser did not flag the request as
///   cross-site, or
/// - there is no `Origin` but the browser says `Sec-Fetch-Site: same-origin`.
///
/// Anything else (including a missing `Origin` without that hint, and the
/// literal `null` origin) is rejected.
pub fn origin_is_allowed(headers: OriginHeaders<'_>, allowed_origins: &[String]) -> bool {
    match headers.origin {
        Some(origin) => {
            if allowed_origins.iter().any(|o| o == origin) {
                return true;
            }
            if headers.sec_fetch_site == Some("cross-site") {
                return false;
            }
            let host = origin_host(origin);
            [headers.host, headers.forwarded_host]
                .into_iter()
                .flatten()
                .any(|h| h == host)
        }
        None => headers.sec_fetch_site == Some("same-origin"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h<'a>(origin: Option<&'a str>, host: Option<&'a str>) -> OriginHeaders<'a> {
        OriginHeaders {
            origin,
            host,
            ..Default::default()
        }
    }

    #[test]
    fn same_origin_allowed_when_allow_list_empty() {
        assert!(origin_is_allowed(
            h(Some("https://hypercs.example"), Some("hypercs.example")),
            &[]
        ));
        assert!(origin_is_allowed(
            h(Some("http://localhost:5173"), Some("localhost:5173")),
            &[]
        ));
    }

    #[test]
    fn cross_origin_rejected() {
        assert!(!origin_is_allowed(
            h(Some("https://evil.example"), Some("hypercs.example")),
            &[]
        ));
        assert!(!origin_is_allowed(
            h(Some("null"), Some("hypercs.example")),
            &[]
        ));
    }

    #[test]
    fn missing_origin_rejected_unless_sec_fetch_says_same_origin() {
        assert!(!origin_is_allowed(h(None, Some("hypercs.example")), &[]));
        let mut headers = h(None, Some("hypercs.example"));
        headers.sec_fetch_site = Some("same-origin");
        assert!(origin_is_allowed(headers, &[]));
        headers.sec_fetch_site = Some("cross-site");
        assert!(!origin_is_allowed(headers, &[]));
    }

    #[test]
    fn allow_list_is_trusted_in_addition_to_same_origin() {
        let allowed = vec!["http://localhost:5173".to_string()];
        // dev server proxying with a rewritten Host
        assert!(origin_is_allowed(
            h(Some("http://localhost:5173"), Some("api:8080")),
            &allowed
        ));
        assert!(!origin_is_allowed(
            h(Some("https://not-allowed.example"), Some("hypercs.example")),
            &allowed
        ));
        // same-origin still works with a non-empty allow-list
        assert!(origin_is_allowed(
            h(Some("https://hypercs.example"), Some("hypercs.example")),
            &allowed
        ));
    }

    #[test]
    fn forwarded_host_is_honored_but_not_for_cross_site_requests() {
        let mut headers = h(Some("https://hypercs.example"), Some("api:8080"));
        headers.forwarded_host = Some("hypercs.example");
        assert!(origin_is_allowed(headers, &[]));
        headers.sec_fetch_site = Some("cross-site");
        assert!(!origin_is_allowed(headers, &[]));
    }
}
