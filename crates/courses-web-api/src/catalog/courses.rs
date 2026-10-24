//! The public course API over the catalog's relational tables (decision 16). Only public fields live there.

use axum::{
    Extension, Json,
    extract::{Path, Query},
};
use rusqlite::{Connection, OptionalExtension, types::Value as Sql};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    db::course_id_from_code,
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Term {
    /// Such as `S27`.
    pub id: String,
    /// Such as `Spring 2027`.
    pub name: String,
    pub season: String,
    pub year: i64,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CourseSummary {
    /// Five digits, such as `15122`.
    pub id: String,
    /// Such as `15-122`.
    pub code: String,
    pub title: String,
    #[schema(required = true)]
    pub department: Option<String>,
    #[schema(required = true)]
    pub units: Option<f64>,
    /// Upper bound for variable-unit courses.
    #[schema(required = true)]
    pub units_max: Option<f64>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CoursePage {
    pub items: Vec<CourseSummary>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Offering {
    pub term_id: String,
    pub title: String,
    #[schema(required = true)]
    pub department: Option<String>,
    #[schema(required = true)]
    pub units: Option<f64>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Meeting {
    /// Weekdays, 0 for Monday through 6 for Sunday; empty when TBA.
    pub days: Vec<u8>,
    /// Minutes after midnight, Pittsburgh time; null when TBA.
    #[schema(required = true)]
    pub start: Option<i64>,
    #[schema(required = true)]
    pub end: Option<i64>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    /// Such as `A`, `1` or `A3`.
    pub name: String,
    /// `lecture` or `section`.
    pub kind: String,
    /// The lecture this section belongs to, if any.
    #[schema(required = true)]
    pub parent: Option<String>,
    #[schema(required = true)]
    pub mini: Option<i64>,
    pub meetings: Vec<Meeting>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CourseDetail {
    pub id: String,
    pub code: String,
    pub title: String,
    #[schema(required = true)]
    pub department: Option<String>,
    #[schema(required = true)]
    pub units: Option<f64>,
    #[schema(required = true)]
    pub units_max: Option<f64>,
    #[schema(required = true)]
    pub description: Option<String>,
    #[schema(required = true)]
    pub prerequisites: Option<String>,
    #[schema(required = true)]
    pub corequisites: Option<String>,
    /// Newest first.
    pub offerings: Vec<Offering>,
    /// The sections in `term`, when `term` was given.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<Section>>,
}

#[derive(Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct CourseQuery {
    /// Words to search for in codes, titles and descriptions, or a course number such as `15-122`, `15122` or `15-1`.
    pub search: Option<String>,
    /// Only courses offered in this term.
    pub term: Option<String>,
    /// A department name, such as `Computer Science`, or a two-digit prefix, such as `15`.
    pub department: Option<String>,
    /// Course level in hundreds, such as `100` for 15-122 or `400` for 15-445.
    pub level: Option<u32>,
    /// Courses that can be taken for this many units.
    pub units: Option<f64>,
    /// From 1. Defaults to 1.
    pub page: Option<i64>,
    /// 1 to 100. Defaults to 20.
    pub page_size: Option<i64>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DetailQuery {
    /// Include this term's sections and meetings.
    pub term: Option<String>,
}

/// Every term, newest first.
#[utoipa::path(get, path = "/api/terms", tag = "courses", responses((status = OK, body = Vec<Term>)))]
pub async fn terms(Extension(state): Extension<AppState>) -> ApiResult<Json<Vec<Term>>> {
    state
        .catalog
        .with(|r| {
            let mut stmt = r
                .connection()
                .prepare_cached("SELECT id, name, season, year FROM terms ORDER BY position, id")?;
            let rows = stmt.query_map([], |r| {
                Ok(Term {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    season: r.get(2)?,
                    year: r.get(3)?,
                })
            })?;
            Ok(Json(rows.collect::<Result<_, _>>()?))
        })
        .await
}

/// Search terms as an FTS5 query: each word as a quoted prefix, all required.
fn fts_query(search: &str) -> Option<String> {
    let words: Vec<String> = search
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .take(12)
        .map(|w| format!("\"{}\"*", w.chars().take(40).collect::<String>()))
        .collect();
    (!words.is_empty()).then(|| words.join(" "))
}

/// A course number or its start: `15-122`, `15122`, `15-1`, `15`.
fn number_prefix(search: &str) -> Option<String> {
    let s = search.trim();
    let (dept, rest) = match s.split_once('-') {
        Some((d, r)) => (d, r),
        None if s.len() > 2 => s.split_at(2),
        None => (s, ""),
    };
    let ok = dept.len() == 2
        && rest.len() <= 3
        && dept.bytes().chain(rest.bytes()).all(|b| b.is_ascii_digit());
    ok.then(|| format!("{dept}{rest}"))
}

/// Search courses, optionally limited to a term, department, level or unit count.
#[utoipa::path(
    get,
    path = "/api/courses",
    tag = "courses",
    params(CourseQuery),
    responses((status = OK, body = CoursePage), (status = BAD_REQUEST, body = crate::error::ErrorBody))
)]
pub async fn courses(
    Extension(state): Extension<AppState>,
    Query(q): Query<CourseQuery>,
) -> ApiResult<Json<CoursePage>> {
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(20);
    if !(1..=100_000).contains(&page) || !(1..=100).contains(&page_size) {
        return Err(ApiError::bad_request(
            "page must be at least 1 and pageSize from 1 to 100.",
        ));
    }
    if let Some(level) = q.level
        && (!(100..=900).contains(&level) || level % 100 != 0)
    {
        return Err(ApiError::bad_request(
            "level must be a multiple of 100 from 100 to 900.",
        ));
    }
    if q.units.is_some_and(|u| !u.is_finite() || u < 0.0) {
        return Err(ApiError::bad_request("units must be a number."));
    }
    state
        .catalog
        .with(move |r| search(r.connection(), &q, page, page_size))
        .await
        .map(Json)
}

fn search(db: &Connection, q: &CourseQuery, page: i64, page_size: i64) -> ApiResult<CoursePage> {
    let mut from = "courses c".to_owned();
    let mut clauses: Vec<String> = vec![];
    let mut params: Vec<Sql> = vec![];
    let mut order = "c.id".to_owned();
    if let Some(search) = q.search.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(prefix) = number_prefix(search) {
            clauses.push("c.id LIKE ? || '%'".into());
            params.push(Sql::Text(prefix));
        } else if let Some(fts) = fts_query(search) {
            from = "course_search s JOIN courses c ON c.id = s.id".into();
            clauses.push("course_search MATCH ?".into());
            params.push(Sql::Text(fts));
            order = "bm25(course_search, 0.0, 10.0, 5.0, 1.0), c.id".into();
        }
    }
    if let Some(term) = q.term.as_deref().filter(|t| !t.is_empty()) {
        clauses.push(
            "EXISTS (SELECT 1 FROM offerings o WHERE o.course_id = c.id AND o.term_id = ?)".into(),
        );
        params.push(Sql::Text(term.to_owned()));
    }
    if let Some(dept) = q
        .department
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty())
    {
        if dept.len() == 2 && dept.bytes().all(|b| b.is_ascii_digit()) {
            clauses.push("substr(c.id, 1, 2) = ?".into());
        } else {
            clauses.push("lower(c.department) = lower(?)".into());
        }
        params.push(Sql::Text(dept.to_owned()));
    }
    if let Some(level) = q.level {
        clauses.push("substr(c.id, 3, 1) = ?".into());
        params.push(Sql::Text((level / 100).to_string()));
    }
    if let Some(units) = q.units {
        clauses.push("c.units <= ? AND coalesce(c.units_max, c.units) >= ?".into());
        params.push(Sql::Real(units));
        params.push(Sql::Real(units));
    }
    let filter = if clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", clauses.join(" AND "))
    };
    let total: i64 = db.query_row(
        &format!("SELECT count(*) FROM {from} {filter}"),
        rusqlite::params_from_iter(params.iter()),
        |r| r.get(0),
    )?;
    params.push(Sql::Integer(page_size));
    params.push(Sql::Integer((page - 1) * page_size));
    let mut stmt = db.prepare(&format!(
        "SELECT c.id, c.code, c.title, c.department, c.units, c.units_max FROM {from} {filter}
         ORDER BY {order} LIMIT ? OFFSET ?"
    ))?;
    let items = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |r| {
            Ok(CourseSummary {
                id: r.get(0)?,
                code: r.get(1)?,
                title: r.get(2)?,
                department: r.get(3)?,
                units: r.get(4)?,
                units_max: r.get(5)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(CoursePage {
        items,
        total,
        page,
        page_size,
    })
}

/// One course, by code (`15-122`) or id (`15122`), with its offerings, and its sections when `term` is given.
#[utoipa::path(
    get,
    path = "/api/courses/{code}",
    tag = "courses",
    params(("code" = String, Path, description = "`15-122` or `15122`"), DetailQuery),
    responses((status = OK, body = CourseDetail), (status = NOT_FOUND, body = crate::error::ErrorBody))
)]
pub async fn course(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
    Query(q): Query<DetailQuery>,
) -> ApiResult<Json<CourseDetail>> {
    let id = course_id_from_code(&code).ok_or_else(|| ApiError::not_found("No such course."))?;
    state
        .catalog
        .with(move |r| detail(r.connection(), &id, q.term.as_deref()))
        .await?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("No such course."))
}

pub fn detail(db: &Connection, id: &str, term: Option<&str>) -> ApiResult<Option<CourseDetail>> {
    let course = db
        .query_row(
            "SELECT id, code, title, department, units, units_max, description, prerequisites, corequisites
             FROM courses WHERE id = ?",
            [id],
            |r| {
                Ok(CourseDetail {
                    id: r.get(0)?,
                    code: r.get(1)?,
                    title: r.get(2)?,
                    department: r.get(3)?,
                    units: r.get(4)?,
                    units_max: r.get(5)?,
                    description: r.get(6)?,
                    prerequisites: r.get(7)?,
                    corequisites: r.get(8)?,
                    offerings: vec![],
                    sections: None,
                })
            },
        )
        .optional()?;
    let Some(mut course) = course else {
        return Ok(None);
    };
    let mut stmt = db.prepare_cached(
        "SELECT o.term_id, o.title, o.department, o.units FROM offerings o
         LEFT JOIN terms t ON t.id = o.term_id WHERE o.course_id = ? ORDER BY t.position, o.term_id",
    )?;
    course.offerings = stmt
        .query_map([id], |r| {
            Ok(Offering {
                term_id: r.get(0)?,
                title: r.get(1)?,
                department: r.get(2)?,
                units: r.get(3)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    if let Some(term) = term {
        course.sections = Some(sections(db, term, id)?);
    }
    Ok(Some(course))
}

fn sections(db: &Connection, term: &str, id: &str) -> ApiResult<Vec<Section>> {
    let mut stmt = db.prepare_cached(
        "SELECT name, kind, parent, mini FROM sections WHERE term_id = ? AND course_id = ? ORDER BY name",
    )?;
    let mut sections: Vec<Section> = stmt
        .query_map([term, id], |r| {
            Ok(Section {
                name: r.get(0)?,
                kind: r.get(1)?,
                parent: r.get(2)?,
                mini: r.get(3)?,
                meetings: vec![],
            })
        })?
        .collect::<Result<_, _>>()?;
    let mut stmt = db.prepare_cached(
        "SELECT days, start, end FROM meetings WHERE term_id = ? AND course_id = ? AND section = ? ORDER BY position",
    )?;
    for section in &mut sections {
        section.meetings = stmt
            .query_map([term, id, section.name.as_str()], |r| {
                let days: String = r.get(0)?;
                Ok(Meeting {
                    days: days
                        .bytes()
                        .filter(u8::is_ascii_digit)
                        .map(|b| b - b'0')
                        .collect(),
                    start: r.get(1)?,
                    end: r.get(2)?,
                })
            })?
            .collect::<Result<_, _>>()?;
    }
    Ok(sections)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_parsing() {
        assert_eq!(number_prefix("15-122").as_deref(), Some("15122"));
        assert_eq!(number_prefix("15122").as_deref(), Some("15122"));
        assert_eq!(number_prefix("15-1").as_deref(), Some("151"));
        assert_eq!(number_prefix("15").as_deref(), Some("15"));
        assert_eq!(number_prefix("151223"), None);
        assert_eq!(number_prefix("intro"), None);
        assert_eq!(
            fts_query("Intro to \"ML\"*").as_deref(),
            Some("\"Intro\"* \"to\"* \"ML\"*")
        );
        assert_eq!(fts_query("- * ()"), None);
    }
}
