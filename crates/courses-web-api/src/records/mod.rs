//! Personal records (decision 17): schedules, plans, the academic record, preferences and saved courses. Every record
//! has a revision; a write names the revision it was based on, and a stale write gets 409 with the current record.
//! Each record takes at most [`DAILY_WRITES`] writes a UTC day, like Courseweave's synced documents.

pub mod documents;
pub mod plans;
pub mod saved;
pub mod schedules;

use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    db::{now_ms, utc_day},
    error::{ApiError, ApiResult},
};

/// Longest schedule or plan name, in characters.
pub const NAME_MAX: usize = 120;
/// Largest stored record, in bytes of JSON.
pub const BODY_MAX: usize = 128 * 1024;
/// Most writes one record accepts per UTC day. Client writes are debounced, so normal use is far below this.
pub const DAILY_WRITES: i64 = 500;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct TermQuery {
    /// Only this term, such as `S27`.
    pub term: Option<String>,
}

pub fn is_course_id(s: &str) -> bool {
    s.len() == 5 && s.bytes().all(|b| b.is_ascii_digit())
}

/// `S27`, `M26` or `F26`.
pub fn is_term(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 3 && matches!(b[0], b'S' | b'M' | b'F') && b[1..].iter().all(u8::is_ascii_digit)
}

/// Client-made ids: 1 to 64 letters, digits, `-` or `_`.
pub fn is_record_id(s: &str) -> bool {
    (1..=64).contains(&s.len())
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

pub fn check(ok: bool, message: impl Into<String>) -> ApiResult<()> {
    if ok {
        Ok(())
    } else {
        Err(ApiError::bad_request(message))
    }
}

/// A trimmed name of 1 to [`NAME_MAX`] characters.
pub fn check_name(name: &str) -> ApiResult<String> {
    let name = name.trim();
    check(
        !name.is_empty() && name.chars().count() <= NAME_MAX,
        format!("Names must be 1 to {NAME_MAX} characters."),
    )?;
    Ok(name.to_owned())
}

/// An optional text field of at most `max` characters.
pub fn check_text(field: &str, value: Option<&str>, max: usize) -> ApiResult<()> {
    check(
        value.is_none_or(|v| v.chars().count() <= max),
        format!("{field} must be at most {max} characters."),
    )
}

pub fn check_units(units: Option<f64>) -> ApiResult<()> {
    check(
        units.is_none_or(|u| u.is_finite() && (0.0..=1000.0).contains(&u)),
        "Units must be a number from 0 to 1000.",
    )
}

/// The JSON to store, refused when it's over [`BODY_MAX`].
pub fn to_stored(value: &impl serde::Serialize) -> ApiResult<String> {
    let text = serde_json::to_string(value)?;
    if text.len() > BODY_MAX {
        return Err(ApiError::too_large(format!(
            "Records are limited to {BODY_MAX} bytes."
        )));
    }
    Ok(text)
}

/// The id for a new record: the client's, when it gave a valid one, or a new UUID.
pub fn new_id(given: Option<String>) -> ApiResult<String> {
    match given {
        Some(id) => {
            check(
                is_record_id(&id),
                "Ids are 1 to 64 letters, digits, - or _.",
            )?;
            Ok(id)
        }
        None => Ok(uuid::Uuid::new_v4().to_string()),
    }
}

/// Seconds until the next UTC midnight.
pub fn seconds_until_next_utc_day(now: i64) -> u64 {
    let next = (utc_day(now) + 1) * 86_400_000;
    ((next - now) as u64).div_ceil(1000)
}

/// The write-cap bookkeeping a guarded update sets, as SQL. Binds `:today` and expects `:cap` in the WHERE clause.
pub const COUNT_WRITE: &str =
    "day_writes = CASE WHEN day = :today THEN day_writes + 1 ELSE 1 END, day = :today";
/// The WHERE clause half that enforces the cap.
pub const UNDER_CAP: &str = "(day <> :today OR day_writes < :cap)";

/// Bookkeeping read back after a guarded update changed nothing.
pub struct WriteState {
    pub revision: i64,
    pub day: i64,
    pub day_writes: i64,
}

/// Why a guarded update changed nothing: the record is gone (404), it reached today's cap (429), or the client's
/// revision is stale (409 with `current`).
pub fn write_failed(
    state: Option<WriteState>,
    base: i64,
    current: impl FnOnce() -> ApiResult<serde_json::Value>,
) -> ApiError {
    let Some(state) = state else {
        return ApiError::not_found("No such record.");
    };
    let now = now_ms();
    if state.revision == base && state.day == utc_day(now) && state.day_writes >= DAILY_WRITES {
        return ApiError::too_many(
            seconds_until_next_utc_day(now),
            "This record has reached today's change limit; it will save again tomorrow (UTC).",
        );
    }
    match current() {
        Ok(value) => {
            let mut error = ApiError::conflict(());
            error.current = Some(value);
            error
        }
        Err(e) => e,
    }
}

pub fn created<T>(body: T) -> (StatusCode, axum::Json<T>) {
    (StatusCode::CREATED, axum::Json(body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validators() {
        assert!(is_course_id("15122") && !is_course_id("15-122") && !is_course_id("1512a"));
        assert!(is_term("S27") && is_term("M26") && !is_term("s27") && !is_term("N26"));
        assert!(is_record_id("abc-1_2") && !is_record_id("") && !is_record_id("a/b"));
        assert_eq!(check_name("  Fall  ").unwrap(), "Fall");
        assert!(check_name(" ").is_err());
        assert!(check_name(&"x".repeat(NAME_MAX + 1)).is_err());
        assert!(check_units(Some(f64::NAN)).is_err());
        assert!(check_units(Some(-1.0)).is_err());
        assert!(check_units(Some(12.0)).is_ok());
        assert_eq!(seconds_until_next_utc_day(86_400_000 - 1500), 2);
    }
}
