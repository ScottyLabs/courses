//! Read-only catalog queries; public/member separation is enforced here.
use super::*;

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
pub(super) fn course_filter(
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
