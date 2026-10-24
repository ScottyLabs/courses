//! Personal SIO final-exam imports. Parse completely before replacing a term.
use super::*;
use axum::extract::Multipart;
use chrono::{Datelike, NaiveDateTime, TimeZone, Utc};
use ical::property::Property;

pub async fn finals(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Query(q): Query<HashMap<String, String>>,
) -> ApiResult {
    let user = require(user)?;
    let term = q.get("term").ok_or_else(|| bad("Term required"))?;
    let stored = rows(
        &*user_db(&b)?,
        "SELECT payload FROM final_imports WHERE owner=? AND term_id=?",
        vec![val(&user.subject), val(term)],
    )?;
    let mut exams = stored.into_iter().next().unwrap_or(json!([]));
    if let Some(codes) = q.get("courses") {
        let codes: Vec<_> = codes.split(',').collect();
        exams
            .as_array_mut()
            .unwrap()
            .retain(|exam| codes.contains(&exam["course_code"].as_str().unwrap_or("")));
    }
    Ok(Json(exams))
}

pub async fn import_finals(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Query(q): Query<HashMap<String, String>>,
    mut multipart: Multipart,
) -> ApiResult {
    let user = require(user)?;
    let term = q.get("term").ok_or_else(|| bad("Term required"))?;
    let mut calendar = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| bad("Invalid calendar upload"))?
    {
        if field.name() != Some("calendar") || calendar.is_some() {
            return Err(bad("Upload one calendar file"));
        }
        if !field
            .file_name()
            .is_some_and(|name| name.to_ascii_lowercase().ends_with(".ics"))
        {
            return Err(bad("Upload a .ics file"));
        }
        calendar = Some(
            field
                .text()
                .await
                .map_err(|_| bad("Calendar must be UTF-8 text"))?,
        );
    }
    let exams = parse_calendar(
        calendar
            .as_deref()
            .ok_or_else(|| bad("Calendar file required"))?,
        term,
        &b.db(true)?,
    )?;
    let payload = json!(exams);
    user_db(&b)?.execute("INSERT INTO final_imports(owner,term_id,payload) VALUES(?,?,?) ON CONFLICT(owner,term_id) DO UPDATE SET payload=excluded.payload", params![user.subject,term,payload.to_string()])?;
    Ok(Json(payload))
}

pub async fn clear_finals(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<StatusCode, ApiError> {
    let user = require(user)?;
    let term = q.get("term").ok_or_else(|| bad("Term required"))?;
    user_db(&b)?.execute(
        "DELETE FROM final_imports WHERE owner=? AND term_id=?",
        params![user.subject, term],
    )?;
    Ok(StatusCode::NO_CONTENT)
}

fn timestamp(property: &Property) -> Result<chrono::DateTime<Utc>, ApiError> {
    let raw = property
        .value
        .as_deref()
        .ok_or_else(|| bad("Missing exam time"))?;
    let naive = NaiveDateTime::parse_from_str(raw.trim_end_matches('Z'), "%Y%m%dT%H%M%S")
        .map_err(|_| bad("Exam must have a date and time"))?;
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
        .map_err(|_| bad("Unsupported calendar timezone"))?;
    zone.from_local_datetime(&naive)
        .single()
        .map(|date| date.with_timezone(&Utc))
        .ok_or_else(|| bad("Ambiguous or invalid local exam time"))
}

fn parse_calendar(text: &str, term: &str, db: &Connection) -> Result<Vec<Value>, ApiError> {
    if text.len() > 120 * 1024 {
        return Err(bad("Calendar exceeds 120 KiB"));
    }
    let mut exams = Vec::new();
    for calendar in ical::IcalParser::new(std::io::Cursor::new(text)) {
        let calendar = calendar.map_err(|_| bad("Malformed iCalendar file"))?;
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
                return Err(bad("Recurring final exams are not supported"));
            }
            let code = summary
                .split(|c: char| !c.is_ascii_digit() && c != '-')
                .find_map(|word| {
                    let digits = word.replace('-', "");
                    (digits.len() == 5).then(|| format!("{}-{}", &digits[..2], &digits[2..]))
                });
            let Some(code) = code else {
                continue;
            };
            let course = rows(
                db,
                "SELECT payload_json FROM catalog_courses WHERE term_id=? AND code=?",
                vec![val(term), val(&code)],
            )?
            .into_iter()
            .next();
            let Some(course) = course else {
                continue;
            };
            let start = timestamp(find("DTSTART").ok_or_else(|| bad("Missing exam start"))?)?;
            let end = timestamp(find("DTEND").ok_or_else(|| bad("Missing exam end"))?)?;
            let local = start.with_timezone(&chrono_tz::America::New_York);
            let season = if local.month() <= 5 {
                'S'
            } else if local.month() <= 8 {
                'M'
            } else {
                'F'
            };
            if format!("{season}{:02}", local.year() % 100) != term {
                return Err(bad("Exam date does not match selected term"));
            }
            if end <= start || end - start > chrono::Duration::hours(12) {
                return Err(bad("Invalid exam duration"));
            }
            exams.push(json!({"course_code":code,"course_name":course["name"],"starts_at":start.to_rfc3339(),"ends_at":end.to_rfc3339(),"location":find("LOCATION").and_then(|p| p.value.as_deref()).unwrap_or("").replace("\\,",",").replace("\\n"," ")}));
        }
    }
    if exams.is_empty() {
        return Err(bad(
            "No final exams matching this term's courses were found; the previous import was preserved",
        ));
    }
    exams.sort_by_key(|exam| exam["starts_at"].as_str().unwrap().to_owned());
    exams.dedup();
    Ok(exams)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn imports_only_matching_finals_and_converts_timezone() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE catalog_courses(term_id TEXT,code TEXT,payload_json TEXT); INSERT INTO catalog_courses VALUES('S27','15-122','{\"name\":\"Imperative Computation\"}');").unwrap();
        let text = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nSUMMARY:15-122 Final Exam\r\nDTSTART;TZID=America/New_York:20270505T090000\r\nDTEND;TZID=America/New_York:20270505T120000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let exams = parse_calendar(text, "S27", &db).unwrap();
        assert_eq!(exams[0]["starts_at"], "2027-05-05T13:00:00+00:00");
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
}
