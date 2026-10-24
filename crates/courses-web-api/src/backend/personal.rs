//! Durable schedules, saved courses and owner-scoped mutations.
use super::*;

pub(super) fn user_db(b: &Backend) -> Result<std::sync::MutexGuard<'_, Connection>, ApiError> {
    b.users.lock().map_err(|_| {
        ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database unavailable".into(),
        )
    })
}
fn schedule(db: &Connection, id: &str, owner: &str) -> Result<Value, ApiError> {
    rows(db,"SELECT json_set(payload,'$.revision',revision,'$.share_id',share_id) FROM schedules WHERE id=? AND owner=?",vec![val(id),val(owner)])?.into_iter().next().ok_or_else(missing)
}
fn validate_schedule(b: &Backend, v: &Value) -> Result<(), ApiError> {
    let name = v["name"]
        .as_str()
        .ok_or_else(|| bad("Schedule name required"))?;
    if name.trim().is_empty() || name.len() > 120 {
        return Err(bad("Name must be 1–120 characters"));
    }
    let t = v["term_id"].as_str().ok_or_else(|| bad("Term required"))?;
    let db = b.db(true)?;
    if !db.query_row(
        "SELECT EXISTS(SELECT 1 FROM catalog_terms WHERE id=?)",
        [t],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(bad("Unknown term"));
    }
    if let Some(id) = v["program_id"].as_str() {
        if !db.query_row(
            "SELECT EXISTS(SELECT 1 FROM programs WHERE id=?)",
            [id],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(bad("Unknown program"));
        }
    }
    let items = v["items"]
        .as_array()
        .ok_or_else(|| bad("Items must be an array"))?;
    if items.len() > 50 {
        return Err(bad("Too many courses"));
    }
    let mut seen = std::collections::HashSet::new();
    for item in items {
        let code = item["course_code"]
            .as_str()
            .ok_or_else(|| bad("Invalid course"))?;
        if !seen.insert(code) {
            return Err(bad("Duplicate course"));
        }
        let c = rows(
            &db,
            "SELECT payload_json FROM catalog_courses WHERE code=? AND term_id=?",
            vec![val(code), val(t)],
        )?
        .into_iter()
        .next()
        .ok_or_else(|| bad("Course is not offered in this term"))?;
        let ids = item["section_ids"]
            .as_array()
            .ok_or_else(|| bad("Section IDs required"))?;
        if ids.len() > 20 {
            return Err(bad("Too many sections"));
        }
        for id in ids {
            if !id.is_string()
                || !c["sections"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|s| s["id"] == *id)
            {
                return Err(bad("Section does not belong to course and term"));
            }
        }
    }
    Ok(())
}
pub async fn schedules(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
) -> ApiResult {
    let u = require(user)?;
    Ok(Json(json!(rows(
        &*user_db(&b)?,
        "SELECT json_set(payload,'$.revision',revision,'$.share_id',share_id) FROM schedules WHERE owner=? ORDER BY id",
        vec![val(&u.subject)]
    )?)))
}
pub async fn create_schedule(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Json(v): Json<Value>,
) -> ApiResult {
    let u = require(user)?;
    let id = uuid::Uuid::new_v4().to_string();
    let payload = json!({"id":id,"name":v["name"],"term_id":v["term_id"],"items":[],"revision":1});
    validate_schedule(&b, &payload)?;
    let db = user_db(&b)?;
    db.execute(
        "INSERT INTO schedules(id,owner,payload) VALUES(?,?,?)",
        params![id, u.subject, payload.to_string()],
    )?;
    Ok(Json(payload))
}
pub async fn save_schedule(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> ApiResult {
    let u = require(user)?;
    validate_schedule(&b, &v)?;
    let revision = v["revision"]
        .as_i64()
        .ok_or_else(|| bad("Schedule revision required; reload before saving"))?;
    let payload = json!({"id":id,"name":v["name"],"term_id":v["term_id"],"items":v["items"].as_array().unwrap().iter().map(|item| json!({"course_code":item["course_code"],"section_ids":item["section_ids"]})).collect::<Vec<_>>(),"program_id":v["program_id"]});
    let db = user_db(&b)?;
    schedule(&db, &id, &u.subject)?;
    if db.execute(
        "UPDATE schedules SET payload=?,revision=revision+1 WHERE id=? AND owner=? AND revision=?",
        params![payload.to_string(), id, u.subject, revision],
    )? == 0
    {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "Schedule changed; reload before saving".into(),
        ));
    }
    Ok(Json(schedule(&db, &id, &u.subject)?))
}
pub async fn delete_schedule(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let u = require(user)?;
    if user_db(&b)?.execute(
        "DELETE FROM schedules WHERE id=? AND owner=?",
        params![id, u.subject],
    )? == 0
    {
        return Err(missing());
    }
    Ok(StatusCode::NO_CONTENT)
}
pub async fn share(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Path(id): Path<String>,
) -> ApiResult {
    let u = require(user)?;
    let db = user_db(&b)?;
    schedule(&db, &id, &u.subject)?;
    db.execute(
        "UPDATE schedules SET share_id=coalesce(share_id,?) WHERE id=? AND owner=?",
        params![uuid::Uuid::new_v4().to_string(), id, u.subject],
    )?;
    Ok(Json(
        json!({"share_id":schedule(&db,&id,&u.subject)?["share_id"]}),
    ))
}
pub async fn revoke_share(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let u = require(user)?;
    if user_db(&b)?.execute(
        "UPDATE schedules SET share_id=NULL WHERE id=? AND owner=?",
        params![id, u.subject],
    )? == 0
    {
        return Err(missing());
    }
    Ok(StatusCode::NO_CONTENT)
}
pub async fn shared(Extension(b): Extension<Backend>, Path(id): Path<String>) -> ApiResult {
    let db = user_db(&b)?;
    let v = rows(
        &db,
        "SELECT payload FROM schedules WHERE share_id=?",
        vec![val(&id)],
    )?
    .into_iter()
    .next()
    .ok_or_else(missing)?;
    Ok(Json(
        json!({"id":"shared","name":v["name"],"term_id":v["term_id"],"items":v["items"],"share_id":id}),
    ))
}
fn status(q: &Value) -> Result<(&str, &str), ApiError> {
    let s = q["status"].as_str().unwrap_or("saved");
    if !["saved", "taking", "taken"].contains(&s) {
        return Err(bad("Invalid status"));
    }
    let t = if s == "saved" {
        ""
    } else {
        q["term_id"]
            .as_str()
            .ok_or_else(|| bad("Term required for history"))?
    };
    Ok((s, t))
}
pub async fn saved(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Query(q): Query<HashMap<String, String>>,
) -> ApiResult {
    let u = require(user)?;
    let s = q.get("status").map(String::as_str).unwrap_or("saved");
    if !["saved", "taking", "taken"].contains(&s) {
        return Err(bad("Invalid status"));
    }
    let db = user_db(&b)?;
    if s != "saved" {
        let history = rows(
            &db,
            "SELECT json_object('code',code,'term_id',term_id) FROM saved_courses WHERE owner=? AND status=? ORDER BY term_id DESC,code",
            vec![val(&u.subject), val(s)],
        )?;
        let catalog = b.db(true)?;
        let mut items = vec![];
        for entry in history {
            let code = entry["code"].as_str().unwrap();
            let term = entry["term_id"].as_str().unwrap();
            for mut course in rows(
                &catalog,
                "SELECT payload_json FROM catalog_courses WHERE code=? AND term_id=?",
                vec![val(code), val(term)],
            )? {
                course["term_id"] = json!(term);
                items.push(course);
            }
        }
        return Ok(Json(json!(items)));
    }
    let mut stmt =
        db.prepare("SELECT DISTINCT code FROM saved_courses WHERE owner=? AND status=?")?;
    let codes = stmt
        .query_map(params![u.subject, s], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    let catalog = b.db(true)?;
    let mut items = vec![];
    for code in codes {
        items.extend(rows(
            &catalog,
            "SELECT payload_json FROM catalog_courses WHERE code=? AND term_id=''",
            vec![val(&code)],
        )?)
    }
    Ok(Json(json!(items)))
}
pub async fn save_course(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Path(code): Path<String>,
    Json(v): Json<Value>,
) -> ApiResult {
    let u = require(user)?;
    let (s, t) = status(&v)?;
    if rows(
        &b.db(true)?,
        "SELECT payload_json FROM catalog_courses WHERE code=? AND term_id=?",
        vec![val(&code), val(t)],
    )?
    .is_empty()
    {
        return Err(bad("Unknown course or term"));
    }
    user_db(&b)?.execute(
        "INSERT OR IGNORE INTO saved_courses(owner,code,status,term_id) VALUES(?,?,?,?)",
        params![u.subject, code, s, t],
    )?;
    Ok(Json(json!({"code":code,"status":s,"term_id":t})))
}
pub async fn unsave_course(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Path(code): Path<String>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<StatusCode, ApiError> {
    let u = require(user)?;
    let v = json!(q);
    let (s, t) = status(&v)?;
    user_db(&b)?.execute(
        "DELETE FROM saved_courses WHERE owner=? AND code=? AND status=? AND term_id=?",
        params![u.subject, code, s, t],
    )?;
    Ok(StatusCode::NO_CONTENT)
}
