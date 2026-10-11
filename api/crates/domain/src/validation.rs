//! Text normalization and validation shared by request DTOs and services.
//! Pure functions (no I/O) so they are cheap to unit test.

use unicode_normalization::UnicodeNormalization;

use crate::constants::RESERVED_USERNAMES;

/// Error code reported for control or invisible characters.
pub const INVALID_CHARACTERS: &str = "invalid_characters";

/// Characters that render invisibly or reorder text and have no business in a
/// single-line field such as a name or title.
fn is_invisible_format(c: char) -> bool {
    matches!(
        c,
        '\u{200B}'..='\u{200F}' // zero-width space/joiners, LRM/RLM
            | '\u{2028}'..='\u{202E}' // line/paragraph separators, bidi embeddings and overrides
            | '\u{2060}'..='\u{2064}' // word joiner and invisible operators
            | '\u{2066}'..='\u{2069}' // bidi isolates
            | '\u{FEFF}'
    )
}

/// Normalizes a single-line field: NFC, trimmed. Rejects (rather than
/// silently strips) control characters, line breaks and invisible/bidi
/// formatting characters.
pub fn normalize_line(input: &str) -> Result<String, &'static str> {
    let nfc: String = input.nfc().collect();
    let trimmed = nfc.trim();
    if trimmed
        .chars()
        .any(|c| c.is_control() || is_invisible_format(c))
    {
        return Err(INVALID_CHARACTERS);
    }
    Ok(trimmed.to_string())
}

/// Normalizes a multi-line text: line endings become `\n`, NFC. Control
/// characters are rejected except newline and tab. With `trim` the result is
/// also trimmed (used for short descriptions); Markdown bodies are otherwise
/// stored exactly as submitted.
pub fn normalize_multiline(input: &str, trim: bool) -> Result<String, &'static str> {
    let unified = input.replace("\r\n", "\n").replace('\r', "\n");
    let nfc: String = unified.nfc().collect();
    if nfc
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err(INVALID_CHARACTERS);
    }
    Ok(if trim { nfc.trim().to_string() } else { nfc })
}

/// True for a username made only of `[A-Za-z0-9_-]`.
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

/// Only `http://` and `https://` URLs with a host are accepted for websites.
pub fn is_http_url(value: &str) -> bool {
    let Some(rest) = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
    else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    !host.is_empty() && !value.chars().any(|c| c.is_whitespace() || c.is_control())
}

/// True if `value` is a known IANA time zone name (or `UTC`).
pub fn is_valid_timezone(value: &str) -> bool {
    value.parse::<chrono_tz::Tz>().is_ok()
}

/// German transliteration first (so `Müller` becomes `mueller`, not
/// `muller`), then a generic ASCII fold for everything else.
fn fold_char(c: char, out: &mut String) {
    match c {
        'ä' => out.push_str("ae"),
        'ö' => out.push_str("oe"),
        'ü' => out.push_str("ue"),
        'ß' => out.push_str("ss"),
        'æ' => out.push_str("ae"),
        'œ' => out.push_str("oe"),
        'ø' => out.push('o'),
        'đ' => out.push('d'),
        'ł' => out.push('l'),
        'þ' => out.push_str("th"),
        c if c.is_ascii() => out.push(c),
        c => {
            // Decompose (é -> e + U+0301) and keep the ASCII base letters;
            // combining marks and everything else non-ASCII are dropped.
            for d in std::iter::once(c).nfd() {
                if d.is_ascii() {
                    out.push(d);
                }
            }
        }
    }
}

/// Cosmetic URL slug: lowercase ASCII, German umlauts transliterated, other
/// accents folded, non-alphanumeric runs collapsed to `-`, truncated to
/// `max_len`. `fallback` is used when nothing usable remains. The UUID is
/// authoritative in URLs, so collisions are fine.
pub fn slugify(input: &str, max_len: usize, fallback: &str) -> String {
    let folded: String = {
        let mut out = String::with_capacity(input.len());
        for c in input.nfc().flat_map(char::to_lowercase) {
            fold_char(c, &mut out);
        }
        out
    };
    let mut slug = String::new();
    let mut last_was_dash = true; // suppress a leading dash
    for c in folded.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    slug.truncate(max_len);
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        fallback.to_string()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_trims_and_normalizes_to_nfc() {
        assert_eq!(normalize_line("  hello  ").as_deref(), Ok("hello"));
        // "e" + combining acute becomes the single precomposed character.
        assert_eq!(normalize_line("e\u{0301}").as_deref(), Ok("\u{00e9}"));
    }

    #[test]
    fn line_rejects_control_newline_and_invisible_characters() {
        assert_eq!(normalize_line("a\u{0007}b"), Err(INVALID_CHARACTERS));
        assert_eq!(normalize_line("a\nb"), Err(INVALID_CHARACTERS));
        assert_eq!(normalize_line("a\tb"), Err(INVALID_CHARACTERS));
        assert_eq!(normalize_line("a\u{200B}b"), Err(INVALID_CHARACTERS));
        assert_eq!(normalize_line("a\u{202E}b"), Err(INVALID_CHARACTERS));
        assert_eq!(normalize_line("a\0b"), Err(INVALID_CHARACTERS));
    }

    #[test]
    fn line_trailing_newline_is_just_trimmed() {
        assert_eq!(normalize_line("title\r\n").as_deref(), Ok("title"));
    }

    #[test]
    fn multiline_normalizes_line_endings_and_keeps_text_as_submitted() {
        assert_eq!(
            normalize_multiline(" hi\r\nthere\rfriend\t! ", false).as_deref(),
            Ok(" hi\nthere\nfriend\t! ")
        );
        assert_eq!(normalize_multiline(" hi ", true).as_deref(), Ok("hi"));
    }

    #[test]
    fn multiline_rejects_control_characters_but_allows_newline_and_tab() {
        assert_eq!(
            normalize_multiline("a\u{0007}b", false),
            Err(INVALID_CHARACTERS)
        );
        assert_eq!(normalize_multiline("a\0b", false), Err(INVALID_CHARACTERS));
        assert!(normalize_multiline("a\n\tb", false).is_ok());
        // ZWJ is legitimate inside emoji sequences in post bodies.
        assert!(normalize_multiline("👨\u{200D}👩", false).is_ok());
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
        assert!(is_reserved_username("Moderator"));
        assert!(!is_reserved_username("robert"));
    }

    #[test]
    fn http_urls() {
        assert!(is_http_url("https://example.com"));
        assert!(is_http_url("http://example.com/path?q=1"));
        assert!(!is_http_url("javascript:alert(1)"));
        assert!(!is_http_url("ftp://example.com"));
        assert!(!is_http_url("https://"));
        assert!(!is_http_url("https://exa mple.com"));
    }

    #[test]
    fn timezones() {
        assert!(is_valid_timezone("UTC"));
        assert!(is_valid_timezone("Europe/Berlin"));
        assert!(!is_valid_timezone("Mars/Olympus"));
        assert!(!is_valid_timezone(""));
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello, World!", 80, "topic"), "hello-world");
        assert_eq!(
            slugify("  multiple   spaces  ", 80, "topic"),
            "multiple-spaces"
        );
        assert_eq!(slugify("日本語", 80, "topic"), "topic");
        assert_eq!(slugify("日本語", 80, "news"), "news");
    }

    #[test]
    fn slugify_transliterates_german_before_folding() {
        assert_eq!(slugify("Müller", 80, "x"), "mueller");
        assert_eq!(slugify("Größe Äpfel Öl", 80, "x"), "groesse-aepfel-oel");
        assert_eq!(slugify("ÜBER STRAßE", 80, "x"), "ueber-strasse");
        // Decomposed input (u + combining diaeresis) must behave the same.
        assert_eq!(slugify("Mu\u{0308}ller", 80, "x"), "mueller");
    }

    #[test]
    fn slugify_folds_other_accents_to_ascii() {
        assert_eq!(slugify("Café déjà vu", 80, "x"), "cafe-deja-vu");
        assert_eq!(slugify("Łódź & Øre", 80, "x"), "lodz-ore");
    }

    #[test]
    fn slugify_truncates_without_trailing_dash() {
        let long = "a".repeat(200);
        assert_eq!(slugify(&long, 10, "x").len(), 10);
        assert_eq!(slugify("abcd efgh", 5, "x"), "abcd");
    }
}
