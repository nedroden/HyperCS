//! Input normalization with field-level errors. Normalization (NFC, trim,
//! line endings) is applied to every text field before it is stored; control
//! and invisible characters are rejected rather than silently stripped.

use hypercs_domain::validation::{normalize_line, normalize_multiline};

use crate::error::{ApiError, ApiResult};

fn length_error(field: &str, min: u64, max: u64) -> ApiError {
    let message = if min > 0 && max < u64::MAX {
        format!("Must be between {min} and {max} characters.")
    } else if min > 0 {
        format!("Must be at least {min} characters.")
    } else {
        format!("Must be at most {max} characters.")
    };
    ApiError::validation(field, "length", message)
}

fn invalid_chars(field: &str, code: &str) -> ApiError {
    ApiError::validation(
        field,
        code,
        "Contains control or invisible characters that are not allowed.",
    )
}

/// Checks the character count (after normalization, which can shorten the
/// text) against `min..=max`.
pub fn check_len(field: &str, value: &str, min: u64, max: u64) -> ApiResult<()> {
    let n = value.chars().count() as u64;
    if n < min || n > max {
        Err(length_error(field, min, max))
    } else {
        Ok(())
    }
}

/// Single-line field: NFC, trimmed, no control/invisible characters, length
/// within `min..=max`.
pub fn line(field: &str, input: &str, min: u64, max: u64) -> ApiResult<String> {
    let value = normalize_line(input).map_err(|code| invalid_chars(field, code))?;
    check_len(field, &value, min, max)?;
    Ok(value)
}

/// Short multi-line text (descriptions, About me): normalized and trimmed.
pub fn text_trimmed(field: &str, input: &str, max: u64) -> ApiResult<String> {
    let value = normalize_multiline(input, true).map_err(|code| invalid_chars(field, code))?;
    check_len(field, &value, 0, max)?;
    Ok(value)
}

/// Markdown body: stored as submitted apart from line endings and NFC; must
/// not be blank.
pub fn body(field: &str, input: &str, min: u64, max: u64) -> ApiResult<String> {
    let value = normalize_multiline(input, false).map_err(|code| invalid_chars(field, code))?;
    if min > 0 && value.trim().is_empty() {
        return Err(length_error(field, min, max));
    }
    check_len(field, &value, min, max)?;
    Ok(value)
}

/// Applies a PATCH value for a nullable optional text field: `None` leaves
/// `current`, `null` or empty clears, a value is normalized with `normalize`.
pub fn merge_nullable(
    current: Option<String>,
    patch: &Option<Option<String>>,
    normalize: impl FnOnce(&str) -> ApiResult<String>,
) -> ApiResult<Option<String>> {
    match patch {
        None => Ok(current),
        Some(None) => Ok(None),
        Some(Some(value)) => {
            let normalized = normalize(value)?;
            Ok((!normalized.is_empty()).then_some(normalized))
        }
    }
}
