//! SQLite catalog reads and durable, owner-scoped personal records.
use crate::auth::CurrentUser;
use axum::{
    Extension, Json,
    extract::{Path, Query, Request},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use rusqlite::{Connection, params, types::Value as SqlValue};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

type ApiResult = Result<Json<Value>, ApiError>;
#[derive(Debug)]
pub struct ApiError(StatusCode, String);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"message":self.1}))).into_response()
    }
}
impl From<rusqlite::Error> for ApiError {
    fn from(e: rusqlite::Error) -> Self {
        tracing::error!(error=%e,"database operation failed");
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database operation failed".into(),
        )
    }
}
fn bad(msg: &str) -> ApiError {
    ApiError(StatusCode::BAD_REQUEST, msg.into())
}
fn missing() -> ApiError {
    ApiError(StatusCode::NOT_FOUND, "Record not found".into())
}
fn require(user: Result<CurrentUser, StatusCode>) -> Result<CurrentUser, ApiError> {
    user.map_err(|s| ApiError(s, "CMU sign-in required".into()))
}
#[derive(Clone)]
pub struct Backend {
    pub catalog: PathBuf,
    users: Arc<Mutex<Connection>>,
    pub origin: String,
}
impl Backend {
    pub fn open(catalog: PathBuf, users: PathBuf, origin: String) -> anyhow::Result<Self> {
        if let Some(parent) = users.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Connection::open(users)?;
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
CREATE TABLE IF NOT EXISTS schedules(id TEXT PRIMARY KEY,owner TEXT NOT NULL,payload TEXT NOT NULL,revision INTEGER NOT NULL DEFAULT 1,share_id TEXT UNIQUE);
CREATE INDEX IF NOT EXISTS schedule_owner ON schedules(owner);
CREATE TABLE IF NOT EXISTS saved_courses(owner TEXT NOT NULL,code TEXT NOT NULL,status TEXT NOT NULL,term_id TEXT NOT NULL DEFAULT '',PRIMARY KEY(owner,code,status,term_id));
PRAGMA user_version=1;")?;
        Ok(Self {
            catalog,
            users: Arc::new(Mutex::new(db)),
            origin: origin.trim_end_matches('/').to_owned(),
        })
    }
    fn db(&self, member: bool) -> Result<Connection, ApiError> {
        Connection::open_with_flags(
            self.catalog.join(if member {
                "member.sqlite"
            } else {
                "public.sqlite"
            }),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|_| {
            ApiError(
                StatusCode::SERVICE_UNAVAILABLE,
                "Catalog unavailable; run scripts/import_catalog.py".into(),
            )
        })
    }
}
fn rows(db: &Connection, sql: &str, p: Vec<SqlValue>) -> Result<Vec<Value>, ApiError> {
    let mut q = db.prepare(sql)?;
    let records = q.query_map(rusqlite::params_from_iter(p), |r| r.get::<_, String>(0))?;
    records
        .map(|r| {
            let s = r?;
            serde_json::from_str(&s).map_err(|_| {
                ApiError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Invalid stored record".into(),
                )
            })
        })
        .collect()
}
fn val(s: &str) -> SqlValue {
    SqlValue::Text(s.to_owned())
}
fn audience(user: &Result<CurrentUser, StatusCode>) -> Result<bool, ApiError> {
    match user {
        Ok(_) => Ok(true),
        Err(StatusCode::UNAUTHORIZED) => Ok(false),
        Err(s) => Err(ApiError(*s, "CMU sign-in required".into())),
    }
}
pub async fn guard(Extension(b): Extension<Backend>, req: Request, next: Next) -> Response {
    if !matches!(
        *req.method(),
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    ) {
        if req
            .headers()
            .get(header::ORIGIN)
            .and_then(|v| v.to_str().ok())
            != Some(b.origin.as_str())
        {
            return ApiError(
                StatusCode::FORBIDDEN,
                "Request origin does not match APP_URL".into(),
            )
            .into_response();
        }
    }
    let mut response = next.run(req).await;
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "private, no-store".parse().unwrap());
    response
        .headers_mut()
        .insert(header::VARY, "Cookie".parse().unwrap());
    response
}
pub async fn terms(Extension(b): Extension<Backend>) -> ApiResult {
    Ok(Json(json!(rows(
        &b.db(false)?,
        "SELECT payload_json FROM catalog_terms ORDER BY sort_order",
        vec![]
    )?)))
}
pub async fn geneds(Extension(b): Extension<Backend>) -> ApiResult {
    Ok(Json(json!(rows(
        &b.db(false)?,
        "SELECT payload_json FROM catalog_geneds ORDER BY id",
        vec![]
    )?)))
}
pub async fn courses(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Query(q): Query<HashMap<String, String>>,
) -> ApiResult {
    let member = audience(&user)?;
    let db = b.db(member)?;
    let (sql, p) = course_filter(&q, member)?;
    let page = q
        .get("page")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1)
        .clamp(1, 1_000_000);
    let size = q
        .get("page_size")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(12)
        .clamp(1, 100);
    let total: i64 = db.query_row(
        &format!("SELECT count(*) FROM catalog_courses WHERE {sql}"),
        rusqlite::params_from_iter(p.clone()),
        |r| r.get(0),
    )?;
    let mut p = p;
    p.push(SqlValue::Integer(size));
    p.push(SqlValue::Integer((page - 1) * size));
    let items = rows(
        &db,
        &format!(
            "SELECT payload_json FROM catalog_courses WHERE {sql} ORDER BY code LIMIT ? OFFSET ?"
        ),
        p,
    )?;
    Ok(Json(
        json!({"items":items,"total":total,"page":page,"page_size":size}),
    ))
}
fn course_filter(
    q: &HashMap<String, String>,
    member: bool,
) -> Result<(String, Vec<SqlValue>), ApiError> {
    let mut clauses = vec!["term_id = ?".to_owned()];
    let mut p = vec![val(q.get("term").map(String::as_str).unwrap_or(""))];
    if let Some(s) = q.get("search").filter(|s| !s.trim().is_empty()) {
        clauses.push("(instr(lower(code || ' ' || name || ' ' || description), ?) > 0 OR instr(replace(code, '-', ''), ?) > 0)".into());
        p.push(val(&s.trim().to_lowercase()));
        p.push(val(&s.trim().replace('-', "")));
    }
    if let Some(s) = q.get("department").filter(|s| !s.is_empty()) {
        clauses.push("(lower(department) = lower(?) OR substr(code,1,length(?)) = ?)".into());
        p.extend([val(s), val(s), val(s)]);
    }
    for (key, col) in [
        ("level", "level"),
        ("units", "units"),
        ("min_fce", "overall_course"),
    ] {
        if let Some(s) = q.get(key).filter(|s| !s.is_empty()) {
            if key == "min_fce" && !member {
                return Err(ApiError(
                    StatusCode::UNAUTHORIZED,
                    "FCE filters require CMU sign-in".into(),
                ));
            }
            let n = s
                .parse::<f64>()
                .map_err(|_| bad("Invalid numeric filter"))?;
            if !n.is_finite() {
                return Err(bad("Invalid numeric filter"));
            }
            let op = if key == "min_fce" || (key == "level" && n == 500.) {
                ">="
            } else {
                "="
            };
            clauses.push(format!("{col} {op} ?"));
            p.push(SqlValue::Real(n));
        }
    }
    if let Some(s) = q.get("offered_in").filter(|s| !s.is_empty()) {
        clauses.push("instr('|' || offered_in || '|', '|' || ? || '|') > 0".into());
        p.push(val(&s.to_lowercase()));
    }
    Ok((clauses.join(" AND "), p))
}
pub async fn course(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Path(code): Path<String>,
    Query(q): Query<HashMap<String, String>>,
) -> ApiResult {
    let db = b.db(audience(&user)?)?;
    let r = rows(
        &db,
        "SELECT payload_json FROM catalog_courses WHERE code=? AND term_id=?",
        vec![
            val(&code),
            val(q.get("term").map(String::as_str).unwrap_or("")),
        ],
    )?;
    Ok(Json(r.into_iter().next().ok_or_else(missing)?))
}
pub async fn instructors(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Query(q): Query<HashMap<String, String>>,
) -> ApiResult {
    require(user)?;
    let items = rows(
        &b.db(true)?,
        "SELECT payload_json FROM catalog_instructors WHERE instr(lower(name || ' ' || department),?)>0 ORDER BY name,id",
        vec![val(&q
            .get("search")
            .cloned()
            .unwrap_or_default()
            .trim()
            .to_lowercase())],
    )?;
    Ok(Json(json!({"total":items.len(),"items":items})))
}
pub async fn instructor(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Path(id): Path<String>,
) -> ApiResult {
    require(user)?;
    Ok(Json(
        rows(
            &b.db(true)?,
            "SELECT payload_json FROM catalog_instructors WHERE id=?",
            vec![val(&id)],
        )?
        .into_iter()
        .next()
        .ok_or_else(missing)?,
    ))
}
pub async fn programs(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Query(q): Query<HashMap<String, String>>,
) -> ApiResult {
    let items = rows(
        &b.db(audience(&user)?)?,
        "SELECT json_object('id',id,'name',name,'audit_name',json_extract(payload_json,'$.audit_name')) FROM programs WHERE instr(lower(name),?)>0 ORDER BY name,id",
        vec![val(&q
            .get("search")
            .cloned()
            .unwrap_or_default()
            .to_lowercase())],
    )?;
    Ok(Json(json!({"items":items,"total":items.len()})))
}
pub async fn program(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Path(id): Path<String>,
) -> ApiResult {
    Ok(Json(
        rows(
            &b.db(audience(&user)?)?,
            "SELECT payload_json FROM programs WHERE id=?",
            vec![val(&id)],
        )?
        .into_iter()
        .next()
        .ok_or_else(missing)?,
    ))
}
pub async fn version(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
) -> ApiResult {
    let member = audience(&user)?;
    let bytes = std::fs::read(b.catalog.join(if member {
        "member-browser.sqlite"
    } else {
        "public-browser.sqlite"
    }))
    .map_err(|_| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Catalog unavailable".into(),
        )
    })?;
    Ok(Json(
        json!({"hash":format!("{:x}",Sha256::digest(&bytes)),"bytes":bytes.len(),"format":"sqlite","schema_version":1,"audience":if member{"member"}else{"public"}}),
    ))
}
pub async fn binary(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
) -> Result<Response, ApiError> {
    let member = audience(&user)?;
    let bytes = tokio::fs::read(b.catalog.join(if member {
        "member-browser.sqlite.gz"
    } else {
        "public-browser.sqlite.gz"
    }))
    .await
    .map_err(|_| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Catalog unavailable".into(),
        )
    })?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/vnd.sqlite3"),
            (header::CONTENT_ENCODING, "gzip"),
        ],
        bytes,
    )
        .into_response())
}

fn user_db(b: &Backend) -> Result<std::sync::MutexGuard<'_, Connection>, ApiError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Backend {
        let root = std::env::temp_dir().join(format!("courses-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        for audience in ["public", "member"] {
            let db = Connection::open(root.join(format!("{audience}.sqlite"))).unwrap();
            db.execute_batch("CREATE TABLE catalog_terms(id TEXT,sort_order INTEGER,payload_json TEXT); INSERT INTO catalog_terms VALUES('F26',0,'{}'); CREATE TABLE catalog_courses(term_id TEXT,code TEXT,payload_json TEXT); CREATE TABLE catalog_instructors(id TEXT,name TEXT,department TEXT,payload_json TEXT);").unwrap();
            db.execute(
                "INSERT INTO catalog_courses VALUES('F26','15-122',?)",
                [json!({"code":"15-122","sections":[{"id":"s1"}]}).to_string()],
            )
            .unwrap();
        }
        Backend::open(
            root.clone(),
            root.join("users.sqlite"),
            "https://courses.test".into(),
        )
        .unwrap()
    }
    fn user(id: &str) -> Result<CurrentUser, StatusCode> {
        Ok(CurrentUser {
            subject: id.into(),
            name: id.into(),
            groups: vec![],
        })
    }
    fn body() -> Value {
        json!({"name":"Fall plan","term_id":"F26"})
    }
    #[tokio::test]
    async fn personal_records_survive_restart_and_enforce_ownership() {
        let b = fixture();
        let created = create_schedule(user("alice"), Extension(b.clone()), Json(body()))
            .await
            .unwrap()
            .0;
        let id = created["id"].as_str().unwrap().to_owned();
        assert_eq!(
            delete_schedule(user("bob"), Extension(b.clone()), Path(id.clone()))
                .await
                .unwrap_err()
                .0,
            StatusCode::NOT_FOUND
        );
        let reopened = Backend::open(
            b.catalog.clone(),
            b.catalog.join("users.sqlite"),
            b.origin.clone(),
        )
        .unwrap();
        assert_eq!(
            schedules(user("alice"), Extension(reopened))
                .await
                .unwrap()
                .0
                .as_array()
                .unwrap()
                .len(),
            1
        );
        std::fs::remove_dir_all(&b.catalog).unwrap();
    }
    #[tokio::test]
    async fn saved_courses_survive_restart_and_are_owner_scoped() {
        let b = fixture();
        let catalog = Connection::open(b.catalog.join("member.sqlite")).unwrap();
        catalog
            .execute(
                "INSERT INTO catalog_courses VALUES('', '15-122', ?)",
                [json!({"code":"15-122","sections":[]}).to_string()],
            )
            .unwrap();
        save_course(
            user("alice"),
            Extension(b.clone()),
            Path("15-122".into()),
            Json(json!({"status":"saved"})),
        )
        .await
        .unwrap();
        save_course(
            user("alice"),
            Extension(b.clone()),
            Path("15-122".into()),
            Json(json!({"status":"taken","term_id":"F26"})),
        )
        .await
        .unwrap();
        let reopened = Backend::open(
            b.catalog.clone(),
            b.catalog.join("users.sqlite"),
            b.origin.clone(),
        )
        .unwrap();
        assert_eq!(
            saved(
                user("alice"),
                Extension(reopened.clone()),
                Query(HashMap::new())
            )
            .await
            .unwrap()
            .0
            .as_array()
            .unwrap()
            .len(),
            1
        );
        assert!(
            saved(
                user("bob"),
                Extension(reopened.clone()),
                Query(HashMap::new())
            )
            .await
            .unwrap()
            .0
            .as_array()
            .unwrap()
            .is_empty()
        );
        unsave_course(
            user("alice"),
            Extension(reopened.clone()),
            Path("15-122".into()),
            Query(HashMap::new()),
        )
        .await
        .unwrap();
        assert!(
            saved(
                user("alice"),
                Extension(reopened.clone()),
                Query(HashMap::new())
            )
            .await
            .unwrap()
            .0
            .as_array()
            .unwrap()
            .is_empty()
        );
        assert_eq!(
            saved(
                user("alice"),
                Extension(reopened),
                Query(HashMap::from([("status".into(), "taken".into())]))
            )
            .await
            .unwrap()
            .0
            .as_array()
            .unwrap()
            .len(),
            1
        );
        std::fs::remove_dir_all(&b.catalog).unwrap();
    }
    #[tokio::test]
    async fn stale_revisions_and_wrong_sections_are_rejected() {
        let b = fixture();
        let mut v = create_schedule(user("alice"), Extension(b.clone()), Json(body()))
            .await
            .unwrap()
            .0;
        let id = v["id"].as_str().unwrap().to_owned();
        v["items"] = json!([{"course_code":"15-122","section_ids":["s1"]}]);
        let _ = save_schedule(
            user("alice"),
            Extension(b.clone()),
            Path(id.clone()),
            Json(v.clone()),
        )
        .await
        .unwrap();
        assert_eq!(
            save_schedule(
                user("alice"),
                Extension(b.clone()),
                Path(id.clone()),
                Json(v.clone())
            )
            .await
            .unwrap_err()
            .0,
            StatusCode::CONFLICT
        );
        v["items"][0]["section_ids"] = json!(["other-course"]);
        assert_eq!(
            save_schedule(user("alice"), Extension(b.clone()), Path(id), Json(v))
                .await
                .unwrap_err()
                .0,
            StatusCode::BAD_REQUEST
        );
        std::fs::remove_dir_all(&b.catalog).unwrap();
    }
    #[tokio::test]
    async fn shares_can_be_revoked_and_exclude_owner() {
        let b = fixture();
        let v = create_schedule(user("alice"), Extension(b.clone()), Json(body()))
            .await
            .unwrap()
            .0;
        let id = v["id"].as_str().unwrap().to_owned();
        let grant = share(user("alice"), Extension(b.clone()), Path(id.clone()))
            .await
            .unwrap()
            .0;
        let token = grant["share_id"].as_str().unwrap().to_owned();
        let view = shared(Extension(b.clone()), Path(token.clone()))
            .await
            .unwrap()
            .0;
        assert!(view.get("owner").is_none());
        assert_eq!(view["id"], "shared");
        revoke_share(user("alice"), Extension(b.clone()), Path(id))
            .await
            .unwrap();
        assert_eq!(
            shared(Extension(b.clone()), Path(token))
                .await
                .unwrap_err()
                .0,
            StatusCode::NOT_FOUND
        );
        std::fs::remove_dir_all(&b.catalog).unwrap();
    }
    #[tokio::test]
    async fn anonymous_cannot_read_instructors_or_personal_data() {
        let b = fixture();
        assert_eq!(
            instructors(
                Err(StatusCode::UNAUTHORIZED),
                Extension(b.clone()),
                Query(HashMap::new())
            )
            .await
            .unwrap_err()
            .0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            schedules(Err(StatusCode::FORBIDDEN), Extension(b.clone()))
                .await
                .unwrap_err()
                .0,
            StatusCode::FORBIDDEN
        );
        assert!(course_filter(&HashMap::from([("min_fce".into(), "4".into())]), false).is_err());
        std::fs::remove_dir_all(&b.catalog).unwrap();
    }
    #[tokio::test]
    async fn origin_is_required_for_cookie_mutations() {
        use tower::ServiceExt;
        let b = fixture();
        let app = axum::Router::new()
            .route("/write", axum::routing::post(|| async { "ok" }))
            .layer(axum::middleware::from_fn(guard))
            .layer(Extension(b.clone()));
        for origin in [
            None,
            Some("https://evil.test"),
            Some("https://courses.test"),
        ] {
            let mut req = Request::builder().method("POST").uri("/write");
            if let Some(origin) = origin {
                req = req.header(header::ORIGIN, origin)
            }
            let result = app
                .clone()
                .oneshot(req.body(axum::body::Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(
                result.status(),
                if origin == Some("https://courses.test") {
                    StatusCode::OK
                } else {
                    StatusCode::FORBIDDEN
                }
            );
        }
        std::fs::remove_dir_all(&b.catalog).unwrap();
    }
}
