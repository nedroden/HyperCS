//! Text normalization and validation shared by request DTOs and services.
//! Keep these pure (no I/O) so they can run both client-independent checks
//! and be unit tested cheaply.

use unicode_normalization::UnicodeNormalization;

use crate::constants::RESERVED_USERNAMES;

/// Trim, normalize to NFC, normalize line endings to `\n`, and reject control
/// characters (newlines/tabs allowed only if `multiline`; otherwise they are
/// replaced with a single space rather than dropped).
pub fn normalize_text(input: &str, multiline: bool) -> String {
    let normalized_newlines = input.replace("\r\n", "\n").replace('\r', "\n");
    let nfc: String = normalized_newlines.nfc().collect();
    let mapped: String = nfc
        .chars()
        .filter_map(|c| match c {
            '\n' | '\t' if multiline => Some(c),
            '\n' | '\t' => Some(' '),
            c if c.is_control() => None,
            c => Some(c),
        })
        .collect();
    mapped.trim().to_string()
}

/// Rejects zero-width characters some fonts render invisibly.
pub fn contains_zero_width(input: &str) -> bool {
    input.chars().any(|c| {
        matches!(
            c,
            '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' | '\u{2060}'
        )
    })
}

pub fn is_valid_username_chars(username: &str) -> bool {
    !username.is_empty()
        && username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub fn is_reserved_username(username: &str) -> bool {
    let lower = username.to_lowercase();
    RESERVED_USERNAMES.contains(&lower.as_str())
}

/// Lowercase ASCII slug: non-alphanumeric runs collapsed to `-`, trimmed,
/// truncated. Cosmetic only (the UUID is authoritative); collisions are fine.
pub fn slugify(input: &str, max_len: usize) -> String {
    let mut slug = String::new();
    let mut last_was_dash = true; // suppress leading dash
    for c in input.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    slug.truncate(max_len);
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "topic".to_string()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_trims_and_collapses_newlines_when_single_line() {
        assert_eq!(normalize_text("  hello\r\nworld  ", false), "hello world");
    }

    #[test]
    fn normalize_keeps_newlines_when_multiline() {
        assert_eq!(normalize_text(" hi\r\nthere ", true), "hi\nthere");
    }

    #[test]
    fn normalize_strips_control_chars() {
        assert_eq!(normalize_text("a\u{0007}b", false), "ab");
    }

    #[test]
    fn username_chars_validated() {
        assert!(is_valid_username_chars("robert_monden-1"));
        assert!(!is_valid_username_chars("robert monden"));
        assert!(!is_valid_username_chars("röbert"));
        assert!(!is_valid_username_chars(""));
    }

    #[test]
    fn reserved_usernames_case_insensitive() {
        assert!(is_reserved_username("Admin"));
        assert!(is_reserved_username("ROOT"));
        assert!(!is_reserved_username("robert"));
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello, World!", 80), "hello-world");
        assert_eq!(slugify("  multiple   spaces  ", 80), "multiple-spaces");
        assert_eq!(slugify("日本語", 80), "topic");
    }

    #[test]
    fn slugify_truncates() {
        let long = "a".repeat(200);
        assert_eq!(slugify(&long, 10).len(), 10);
    }
}
