//! A student's own final exam times, imported from their SIO calendar (.ics). The registrar's schedule for everyone
//! is the `finals/<term>` catalog document. A calendar is parsed completely before it replaces a term's import.

use axum::{
    Extension, Json,
    extract::{Multipart, Query},
    http::StatusCode,
};
use chrono::{Datelike, NaiveDateTime, TimeZone, Utc};
use ical::property::Property;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    auth::{CurrentUser, Writer},
    db::course_id_from_code,
    error::{ApiError, ApiResult, ErrorBody},
    records::is_term,
    state::AppState,
};

const CALENDAR_MAX: usize = 120 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FinalExam {
    /// Five-digit course id, such as `15122`.
    pub course_id: String,
    /// Such as `15-122`.
    pub course_code: String,
    pub title: String,
    /// RFC 3339, UTC.
    pub starts_at: String,
    pub ends_at: String,
    pub location: String,
}

/// The first backend stored imports with snake_case fields.
#[derive(Deserialize)]
struct LegacyExam {
    course_code: String,
    #[serde(default)]
    course_name: String,
    starts_at: String,
    ends_at: String,
    #[serde(default)]
    location: String,
}

fn parse_stored(payload: &str) -> Vec<FinalExam> {
    let items: Vec<serde_json::Value> = serde_json::from_str(payload).unwrap_or_default();
    items
        .into_iter()
        .filter_map(|item| {
            if let Ok(exam) = serde_json::from_value::<FinalExam>(item.clone()) {
                return Some(exam);
            }
            let old: LegacyExam = serde_json::from_value(item).ok()?;
            Some(FinalExam {
                course_id: course_id_from_code(&old.course_code)?,
                course_code: old.course_code,
                title: old.course_name,
                starts_at: old.starts_at,
                ends_at: old.ends_at,
                location: old.location,
            })
        })
        .collect()
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct FinalsQuery {
    /// A term, such as `S27`.
    pub term: String,
    /// Only these courses: comma-separated ids or codes.
    pub courses: Option<String>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ImportQuery {
    /// A term, such as `S27`.
    pub term: String,
}

fn check_term(term: &str) -> ApiResult<()> {
    if is_term(term) {
        Ok(())
    } else {
        Err(ApiError::bad_request("term must be a term like S27."))
    }
}

/// The caller's imported final exams for a term, earliest first.
#[utoipa::path(
    get,
    path = "/api/finals",
    tag = "finals",
    params(FinalsQuery),
    responses((status = OK, body = Vec<FinalExam>), (status = UNAUTHORIZED, body = ErrorBody))
)]
pub async fn list(
    user: CurrentUser,
    Extension(state): Extension<AppState>,
    Query(q): Query<FinalsQuery>,
) -> ApiResult<Json<Vec<FinalExam>>> {
    check_term(&q.term)?;
    let term = q.term.clone();
    let stored: Option<String> = state
        .db
        .run(move |db| {
            Ok(db
                .query_row(
                    "SELECT payload FROM final_imports WHERE owner = ? AND term_id = ?",
                    params![user.subject, term],
                    |r| r.get(0),
                )
                .optional()?)
        })
        .await?;
    let mut exams = stored.as_deref().map(parse_stored).unwrap_or_default();
    if let Some(filter) = q.courses {
        let wanted: Vec<String> = filter.split(',').filter_map(course_id_from_code).collect();
        exams.retain(|e| wanted.contains(&e.course_id));
    }
    Ok(Json(exams))
}

/// Replaces the term's import with the final exams in an SIO calendar: one multipart field `calendar`, a UTF-8
/// `.ics` file of at most 120 KiB. Events that aren't finals for this term's courses are dropped, and a calendar with
/// none keeps the previous import.
#[utoipa::path(
    post,
    path = "/api/finals/import",
    tag = "finals",
    params(ImportQuery),
    request_body(content_type = "multipart/form-data", description = "A field `calendar` holding the .ics file"),
    responses((status = OK, body = Vec<FinalExam>), (status = BAD_REQUEST, body = ErrorBody))
)]
pub async fn import(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Query(q): Query<ImportQuery>,
    mut multipart: Multipart,
) -> ApiResult<Json<Vec<FinalExam>>> {
    check_term(&q.term)?;
    let mut calendar = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::bad_request("That upload couldn't be read."))?
    {
        if field.name() != Some("calendar") || calendar.is_some() {
            return Err(ApiError::bad_request("Upload one calendar file."));
        }
        if !field
            .file_name()
            .is_some_and(|name| name.to_ascii_lowercase().ends_with(".ics"))
        {
            return Err(ApiError::bad_request("Upload a .ics file."));
        }
        calendar = Some(
            field
                .text()
                .await
                .map_err(|_| ApiError::bad_request("Calendars must be UTF-8 text."))?,
        );
    }
    let calendar = calendar.ok_or_else(|| ApiError::bad_request("Calendar file required."))?;
    let term = q.term.clone();
    let exams = state
        .catalog
        .with(move |r| parse_calendar(&calendar, &term, r.connection()))
        .await?;
    let payload = serde_json::to_string(&exams)?;
    state
        .db
        .run(move |db| {
            db.execute(
                "INSERT INTO final_imports(owner, term_id, payload) VALUES (?, ?, ?)
                 ON CONFLICT(owner, term_id) DO UPDATE SET payload = excluded.payload",
                params![user.subject, q.term, payload],
            )?;
            Ok(())
        })
        .await?;
    Ok(Json(exams))
}

/// Removes the caller's import for a term.
#[utoipa::path(
    delete,
    path = "/api/finals/import",
    tag = "finals",
    params(ImportQuery),
    responses((status = NO_CONTENT))
)]
pub async fn clear(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Query(q): Query<ImportQuery>,
) -> ApiResult<StatusCode> {
    check_term(&q.term)?;
    state
        .db
        .run(move |db| {
            db.execute(
                "DELETE FROM final_imports WHERE owner = ? AND term_id = ?",
                params![user.subject, q.term],
            )?;
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

fn timestamp(property: &Property) -> ApiResult<chrono::DateTime<Utc>> {
    let raw = property
        .value
        .as_deref()
        .ok_or_else(|| ApiError::bad_request("An exam has no time."))?;
    let naive = NaiveDateTime::parse_from_str(raw.trim_end_matches('Z'), "%Y%m%dT%H%M%S")
        .map_err(|_| ApiError::bad_request("Exams must have a date and time."))?;
    if raw.ends_with('Z') {
        return Ok(naive.and_utc());
    }
    let zone = property
        .params
        .as_ref()
        .and_then(|p| p.iter().find(|(k, _)| k == "TZID"))
        .and_then(|(_, v)| v.first())
        .map(String::as_str)
        .unwrap_or("America/New_York");
    let zone: chrono_tz::Tz = zone
        .parse()
        .map_err(|_| ApiError::bad_request("Unsupported calendar time zone."))?;
    zone.from_local_datetime(&naive)
        .single()
        .map(|date| date.with_timezone(&Utc))
        .ok_or_else(|| ApiError::bad_request("An exam time is ambiguous or invalid."))
}

pub fn parse_calendar(text: &str, term: &str, catalog: &Connection) -> ApiResult<Vec<FinalExam>> {
    if text.len() > CALENDAR_MAX {
        return Err(ApiError::bad_request("Calendars are limited to 120 KiB."));
    }
    let mut title_of = catalog
        .prepare_cached("SELECT title FROM offerings WHERE term_id = ? AND course_id = ?")?;
    let mut exams = Vec::new();
    for calendar in ical::IcalParser::new(std::io::Cursor::new(text)) {
        let calendar =
            calendar.map_err(|_| ApiError::bad_request("That isn't a valid calendar file."))?;
        for event in calendar.events {
            let find = |name: &str| event.properties.iter().find(|p| p.name == name);
            let summary = find("SUMMARY")
                .and_then(|p| p.value.as_deref())
                .unwrap_or("");
            let description = find("DESCRIPTION")
                .and_then(|p| p.value.as_deref())
                .unwrap_or("");
            if !format!("{summary} {description}")
                .to_ascii_lowercase()
                .contains("final")
            {
                continue;
            }
            if find("RRULE").is_some() {
                return Err(ApiError::bad_request(
                    "Recurring final exams aren't supported.",
                ));
            }
            let Some(course_id) = summary
                .split(|c: char| !c.is_ascii_digit() && c != '-')
                .find_map(course_id_from_code)
            else {
                continue;
            };
            let Some(title) = title_of
                .query_row([term, course_id.as_str()], |r| r.get::<_, String>(0))
                .optional()?
            else {
                continue;
            };
            let start = timestamp(
                find("DTSTART").ok_or_else(|| ApiError::bad_request("An exam has no start."))?,
            )?;
            let end = timestamp(
                find("DTEND").ok_or_else(|| ApiError::bad_request("An exam has no end."))?,
            )?;
            let local = start.with_timezone(&chrono_tz::America::New_York);
            let season = match local.month() {
                1..=5 => 'S',
                6..=8 => 'M',
                _ => 'F',
            };
            if format!("{season}{:02}", local.year() % 100) != term {
                return Err(ApiError::bad_request(
                    "An exam's date isn't in the selected term.",
                ));
            }
            if end <= start || end - start > chrono::Duration::hours(12) {
                return Err(ApiError::bad_request("An exam's length is invalid."));
            }
            exams.push(FinalExam {
                course_code: format!("{}-{}", &course_id[..2], &course_id[2..]),
                course_id,
                title,
                starts_at: start.to_rfc3339(),
                ends_at: end.to_rfc3339(),
                location: find("LOCATION")
                    .and_then(|p| p.value.as_deref())
                    .unwrap_or("")
                    .replace("\\,", ",")
                    .replace("\\n", " "),
            });
        }
    }
    if exams.is_empty() {
        return Err(ApiError::bad_request(
            "No final exams for this term's courses were found; the previous import was kept.",
        ));
    }
    exams.sort_by(|a, b| a.starts_at.cmp(&b.starts_at));
    exams.dedup();
    Ok(exams)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_only_matching_finals_and_converts_time_zones() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(
            "CREATE TABLE offerings(course_id TEXT, term_id TEXT, title TEXT);
             INSERT INTO offerings VALUES('15122', 'S27', 'Imperative Computation');",
        )
        .unwrap();
        let text = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nSUMMARY:15-122 Final Exam\r\nDTSTART;TZID=America/New_York:20270505T090000\r\nDTEND;TZID=America/New_York:20270505T120000\r\nLOCATION:Wean 7500\\, Room A\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let exams = parse_calendar(text, "S27", &db).unwrap();
        assert_eq!(exams[0].starts_at, "2027-05-05T13:00:00+00:00");
        assert_eq!(exams[0].course_id, "15122");
        assert_eq!(exams[0].title, "Imperative Computation");
        assert_eq!(exams[0].location, "Wean 7500, Room A");
        assert!(parse_calendar(&text.replace("20270505", "20260505"), "S27", &db).is_err());
        assert!(parse_calendar(&text.replace("Final Exam", "Lecture"), "S27", &db).is_err());
        assert!(
            parse_calendar(
                &text.replace("20270505T120000", "20270505T080000"),
                "S27",
                &db
            )
            .is_err()
        );
    }

    #[test]
    fn reads_imports_stored_by_the_first_backend() {
        let old = r#"[{"course_code":"15-122","course_name":"Imperative","starts_at":"2027-05-05T13:00:00+00:00","ends_at":"2027-05-05T16:00:00+00:00","location":"Wean"}]"#;
        let exams = parse_stored(old);
        assert_eq!(exams.len(), 1);
        assert_eq!(exams[0].course_id, "15122");
        assert_eq!(exams[0].title, "Imperative");
        let new = serde_json::to_string(&exams).unwrap();
        assert_eq!(parse_stored(&new), exams);
    }
}
