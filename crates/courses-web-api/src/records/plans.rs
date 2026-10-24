//! Degree plans: Courseweave's `Plan` (`state/plan.ts`, `lib/plan.ts`) plus a revision. Checked for shape and bounds;
//! plans may name courses and terms the catalog doesn't have yet.

use std::collections::HashSet;

use axum::{Extension, Json, extract::Path, http::StatusCode};
use rusqlite::{Connection, OptionalExtension, named_params, params};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
    COUNT_WRITE, DAILY_WRITES, UNDER_CAP, WriteState, check, check_name, check_units, created,
    is_course_id, is_record_id, is_term, new_id, to_stored, write_failed,
};
use crate::{
    auth::{CurrentUser, Writer},
    db::{now_ms, utc_day},
    error::{ApiError, ApiResult, ErrorBody},
    state::AppState,
};

pub const PLANS_MAX: i64 = 50;
const TERMS_MAX: usize = 40;
const COURSES_PER_TERM_MAX: usize = 40;
const PROGRAMS_MAX: usize = 10;

fn is_false(b: &bool) -> bool {
    !*b
}

/// Which program version a plan audits against (`lib/plan-types.ts`).
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProgramSelection {
    /// Stellic catalog program id.
    pub catalog_id: i64,
    /// The audit version (entry year and campus).
    pub audit_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concentrations: Option<Vec<i64>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanCourse {
    /// Unique within the plan, so a repeatable course can appear twice.
    pub id: String,
    pub course_id: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub pass_fail: bool,
    /// Units to count for a variable-unit course.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanLink {
    pub schedule_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanTerm {
    /// Such as `F25`.
    pub term: String,
    #[serde(default)]
    #[schema(required = true)]
    pub courses: Vec<PlanCourse>,
    /// While set, the term's courses come from this schedule.
    #[serde(default)]
    pub link: Option<PlanLink>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub programs: Vec<ProgramSelection>,
    pub start_term: String,
    /// Oldest first.
    pub terms: Vec<PlanTerm>,
    pub revision: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewPlan {
    /// An id of your own, as for schedules. Defaults to a new UUID.
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub programs: Vec<ProgramSelection>,
    pub start_term: String,
    #[serde(default)]
    pub terms: Vec<PlanTerm>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanUpdate {
    pub name: String,
    pub programs: Vec<ProgramSelection>,
    pub start_term: String,
    pub terms: Vec<PlanTerm>,
    pub revision: i64,
}

/// 409 body for plans.
#[derive(Serialize, ToSchema)]
pub struct PlanConflict {
    pub code: crate::error::ErrorCode,
    pub message: String,
    pub current: Plan,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Body {
    name: String,
    programs: Vec<ProgramSelection>,
    start_term: String,
    terms: Vec<PlanTerm>,
}

fn check_plan(
    programs: &[ProgramSelection],
    start_term: &str,
    terms: &[PlanTerm],
) -> ApiResult<()> {
    check(is_term(start_term), "startTerm must be a term like F25.")?;
    check(
        programs.len() <= PROGRAMS_MAX,
        format!("Plans audit at most {PROGRAMS_MAX} programs."),
    )?;
    for p in programs {
        check(
            p.concentrations.as_ref().is_none_or(|c| c.len() <= 20),
            "Too many concentrations.",
        )?;
    }
    check(
        terms.len() <= TERMS_MAX,
        format!("Plans hold at most {TERMS_MAX} semesters."),
    )?;
    let mut seen_terms = HashSet::new();
    let mut seen_ids = HashSet::new();
    for t in terms {
        check(
            is_term(&t.term),
            format!("{:?} isn't a term like F25.", t.term),
        )?;
        check(
            seen_terms.insert(&t.term),
            format!("{} is in the plan twice.", t.term),
        )?;
        check(
            t.courses.len() <= COURSES_PER_TERM_MAX,
            format!("Semesters hold at most {COURSES_PER_TERM_MAX} courses."),
        )?;
        if let Some(link) = &t.link {
            check(
                is_record_id(&link.schedule_id),
                "Linked schedule ids are record ids.",
            )?;
        }
        for c in &t.courses {
            check(
                is_record_id(&c.id),
                "Plan course ids are 1 to 64 letters, digits, - or _.",
            )?;
            check(seen_ids.insert(&c.id), "Plan course ids must be unique.")?;
            check(
                is_course_id(&c.course_id),
                format!("{:?} isn't a course id like 15122.", c.course_id),
            )?;
            check_units(c.units)?;
        }
    }
    Ok(())
}

const COLUMNS: &str = "id, body, revision, created_at, updated_at, day, day_writes";

fn from_row(r: &rusqlite::Row) -> rusqlite::Result<(Plan, WriteState)> {
    let body: String = r.get(1)?;
    let body: Body = serde_json::from_str(&body).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e))
    })?;
    Ok((
        Plan {
            id: r.get(0)?,
            name: body.name,
            programs: body.programs,
            start_term: body.start_term,
            terms: body.terms,
            revision: r.get(2)?,
            created_at: r.get(3)?,
            updated_at: r.get(4)?,
        },
        WriteState {
            revision: r.get(2)?,
            day: r.get(5)?,
            day_writes: r.get(6)?,
        },
    ))
}

fn load(db: &Connection, owner: &str, id: &str) -> ApiResult<Option<(Plan, WriteState)>> {
    Ok(db
        .query_row(
            &format!("SELECT {COLUMNS} FROM user_plans WHERE owner = ? AND id = ?"),
            [owner, id],
            from_row,
        )
        .optional()?)
}

/// The caller's plans, oldest first.
#[utoipa::path(
    get,
    path = "/api/plans",
    tag = "plans",
    responses((status = OK, body = Vec<Plan>), (status = UNAUTHORIZED, body = ErrorBody))
)]
pub async fn list(
    user: CurrentUser,
    Extension(state): Extension<AppState>,
) -> ApiResult<Json<Vec<Plan>>> {
    state
        .db
        .run(move |db| {
            let mut stmt = db.prepare_cached(&format!(
                "SELECT {COLUMNS} FROM user_plans WHERE owner = ? ORDER BY created_at, id"
            ))?;
            let rows = stmt.query_map([&user.subject], |r| from_row(r).map(|(p, _)| p))?;
            Ok(Json(rows.collect::<Result<_, _>>()?))
        })
        .await
}

/// Creates a plan at revision 1.
#[utoipa::path(
    post,
    path = "/api/plans",
    tag = "plans",
    request_body = NewPlan,
    responses(
        (status = CREATED, body = Plan),
        (status = BAD_REQUEST, body = ErrorBody),
        (status = CONFLICT, description = "A plan with this id exists", body = PlanConflict),
    )
)]
pub async fn create(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Json(new): Json<NewPlan>,
) -> ApiResult<(StatusCode, Json<Plan>)> {
    let id = new_id(new.id)?;
    let name = check_name(&new.name)?;
    check_plan(&new.programs, &new.start_term, &new.terms)?;
    let body = to_stored(&Body {
        name,
        programs: new.programs,
        start_term: new.start_term,
        terms: new.terms,
    })?;
    let plan = state
        .db
        .run(move |db| {
            let tx = db.transaction()?;
            if let Some((current, _)) = load(&tx, &user.subject, &id)? {
                return Err(ApiError::conflict(current));
            }
            let count: i64 = tx.query_row(
                "SELECT count(*) FROM user_plans WHERE owner = ?",
                [&user.subject],
                |r| r.get(0),
            )?;
            check(count < PLANS_MAX, format!("You can keep at most {PLANS_MAX} plans."))?;
            let now = now_ms();
            tx.execute(
                "INSERT INTO user_plans(owner, id, body, revision, created_at, updated_at, day, day_writes)
                 VALUES (?, ?, ?, 1, ?, ?, ?, 1)",
                params![user.subject, id, body, now, now, utc_day(now)],
            )?;
            let created = load(&tx, &user.subject, &id)?.map(|(p, _)| p);
            tx.commit()?;
            created.ok_or_else(|| ApiError::internal("plan vanished"))
        })
        .await?;
    Ok(created(plan))
}

/// Replaces a plan. A stale `revision` gets 409 with the current plan.
#[utoipa::path(
    put,
    path = "/api/plans/{id}",
    tag = "plans",
    params(("id" = String, Path)),
    request_body = PlanUpdate,
    responses(
        (status = OK, body = Plan),
        (status = BAD_REQUEST, body = ErrorBody),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, body = PlanConflict),
        (status = TOO_MANY_REQUESTS, body = ErrorBody),
    )
)]
pub async fn update(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
    Json(update): Json<PlanUpdate>,
) -> ApiResult<Json<Plan>> {
    let name = check_name(&update.name)?;
    check_plan(&update.programs, &update.start_term, &update.terms)?;
    let base = update.revision;
    let body = to_stored(&Body {
        name,
        programs: update.programs,
        start_term: update.start_term,
        terms: update.terms,
    })?;
    let plan = state
        .db
        .run(move |db| {
            let now = now_ms();
            let changed = db.execute(
                &format!(
                    "UPDATE user_plans SET body = :body, revision = revision + 1, updated_at = :now, {COUNT_WRITE}
                     WHERE owner = :owner AND id = :id AND revision = :base AND {UNDER_CAP}"
                ),
                named_params! {
                    ":body": body,
                    ":now": now,
                    ":today": utc_day(now),
                    ":owner": user.subject,
                    ":id": id,
                    ":base": base,
                    ":cap": DAILY_WRITES,
                },
            )?;
            let stored = load(db, &user.subject, &id)?;
            if changed == 1 {
                return stored
                    .map(|(p, _)| p)
                    .ok_or_else(|| ApiError::internal("plan vanished"));
            }
            let (current, write) = match stored {
                Some((p, w)) => (Some(p), Some(w)),
                None => (None, None),
            };
            Err(write_failed(write, base, || Ok(serde_json::to_value(current)?)))
        })
        .await?;
    Ok(Json(plan))
}

/// Deletes a plan.
#[utoipa::path(
    delete,
    path = "/api/plans/{id}",
    tag = "plans",
    params(("id" = String, Path)),
    responses((status = NO_CONTENT), (status = NOT_FOUND, body = ErrorBody))
)]
pub async fn delete(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let removed = state
        .db
        .run(move |db| {
            Ok(db.execute(
                "DELETE FROM user_plans WHERE owner = ? AND id = ?",
                [&user.subject, &id],
            )?)
        })
        .await?;
    if removed == 0 {
        return Err(ApiError::not_found("No such plan."));
    }
    Ok(StatusCode::NO_CONTENT)
}
