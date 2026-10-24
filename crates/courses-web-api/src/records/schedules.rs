//! Schedules: Courseweave's `Schedule` (`state/store.ts`), plus visibility to friends, a revision and a share link.
//! Each is checked against the catalog: the term exists, every course is offered in it, and the chosen sections are
//! that course's sections there.

use std::collections::HashSet;

use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::StatusCode,
};
use rusqlite::{Connection, OptionalExtension, named_params, params};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
    COUNT_WRITE, DAILY_WRITES, TermQuery, UNDER_CAP, WriteState, check, check_name, created,
    is_course_id, new_id, to_stored, write_failed,
};
use crate::{
    auth::{CurrentUser, Writer},
    catalog::Catalog,
    db::{now_ms, utc_day},
    error::{ApiError, ApiResult, ErrorBody},
    state::AppState,
};

pub const ENTRIES_MAX: usize = 100;
/// Most schedules one student keeps.
pub const SCHEDULES_MAX: i64 = 300;
/// Course colors index Courseweave's palette, which has 10; leave room for more.
const COLOR_MAX: u32 = 63;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleEntry {
    /// Five-digit course id, such as `15122`.
    pub course_id: String,
    /// The chosen top-level section (a lecture or a standalone section), or null before one is picked.
    #[serde(default)]
    #[schema(required = true)]
    pub section: Option<String>,
    /// The chosen child section (recitation or lab) of `section`, or null.
    #[serde(default)]
    #[schema(required = true)]
    pub child: Option<String>,
    /// Pass/fail elected.
    #[serde(default)]
    #[schema(required = true)]
    pub pass_fail: bool,
    /// Index into the course color palette.
    #[serde(default)]
    #[schema(required = true)]
    pub color: u32,
    /// Left off the calendar and the unit total.
    #[serde(default)]
    #[schema(required = true)]
    pub hidden: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Private,
    /// Accepted friends who are shown your schedules may see it.
    Friends,
}

impl Visibility {
    fn as_str(self) -> &'static str {
        match self {
            Visibility::Private => "private",
            Visibility::Friends => "friends",
        }
    }

    fn parse(s: &str) -> Self {
        if s == "friends" {
            Visibility::Friends
        } else {
            Visibility::Private
        }
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub id: String,
    pub term_id: String,
    pub name: String,
    pub entries: Vec<ScheduleEntry>,
    pub visibility: Visibility,
    /// Increments on every write. Send it back with the next write.
    pub revision: i64,
    /// The read-only share token, when a share link exists.
    #[schema(required = true)]
    pub share_id: Option<String>,
    /// Milliseconds since the epoch, set by the server.
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewSchedule {
    /// An id of your own (1 to 64 letters, digits, `-` or `_`), so a retried create can't make two. Defaults to a
    /// new UUID.
    pub id: Option<String>,
    pub term_id: String,
    pub name: String,
    #[serde(default)]
    pub entries: Vec<ScheduleEntry>,
    #[serde(default)]
    pub visibility: Visibility,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleUpdate {
    pub term_id: String,
    pub name: String,
    pub entries: Vec<ScheduleEntry>,
    pub visibility: Visibility,
    /// The revision this change is based on.
    pub revision: i64,
}

/// What a share link shows: no owner, id or visibility.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SharedSchedule {
    pub term_id: String,
    pub name: String,
    pub entries: Vec<ScheduleEntry>,
    pub updated_at: i64,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShareLink {
    pub share_id: String,
    /// The website's page for the link.
    pub url: String,
}

/// 409 body for schedules.
#[derive(Serialize, ToSchema)]
pub struct ScheduleConflict {
    pub code: crate::error::ErrorCode,
    pub message: String,
    pub current: Schedule,
}

/// The JSON kept in `user_schedules.body`.
#[derive(Serialize, Deserialize)]
pub struct Body {
    pub name: String,
    pub entries: Vec<ScheduleEntry>,
}

const COLUMNS: &str =
    "id, term_id, body, visibility, revision, share_id, created_at, updated_at, day, day_writes";

fn from_row(r: &rusqlite::Row) -> rusqlite::Result<(Schedule, WriteState)> {
    let body: String = r.get(2)?;
    let body: Body = serde_json::from_str(&body).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e))
    })?;
    Ok((
        Schedule {
            id: r.get(0)?,
            term_id: r.get(1)?,
            name: body.name,
            entries: body.entries,
            visibility: Visibility::parse(&r.get::<_, String>(3)?),
            revision: r.get(4)?,
            share_id: r.get(5)?,
            created_at: r.get(6)?,
            updated_at: r.get(7)?,
        },
        WriteState {
            revision: r.get(4)?,
            day: r.get(8)?,
            day_writes: r.get(9)?,
        },
    ))
}

fn load(db: &Connection, owner: &str, id: &str) -> ApiResult<Option<(Schedule, WriteState)>> {
    Ok(db
        .query_row(
            &format!("SELECT {COLUMNS} FROM user_schedules WHERE owner = ? AND id = ?"),
            [owner, id],
            from_row,
        )
        .optional()?)
}

/// An owner's schedules, oldest first; only those visible to friends when `only_friends`.
pub fn list(
    db: &Connection,
    owner: &str,
    term: Option<&str>,
    only_friends: bool,
) -> ApiResult<Vec<Schedule>> {
    let mut stmt = db.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM user_schedules
         WHERE owner = ?1 AND (?2 IS NULL OR term_id = ?2) AND (?3 = 0 OR visibility = 'friends')
         ORDER BY created_at, id"
    ))?;
    let rows = stmt.query_map(params![owner, term, only_friends], |r| {
        from_row(r).map(|(s, _)| s)
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Shape checks that need no catalog.
fn check_shape(entries: &[ScheduleEntry]) -> ApiResult<()> {
    check(
        entries.len() <= ENTRIES_MAX,
        format!("Schedules hold at most {ENTRIES_MAX} courses."),
    )?;
    let mut seen = HashSet::new();
    for e in entries {
        check(
            is_course_id(&e.course_id),
            format!("{:?} isn't a course id like 15122.", e.course_id),
        )?;
        check(
            seen.insert(e.course_id.as_str()),
            format!("{} is in the schedule twice.", e.course_id),
        )?;
        check(e.color <= COLOR_MAX, "Colors are palette indexes.")?;
        for name in [&e.section, &e.child].into_iter().flatten() {
            check(
                !name.is_empty() && name.len() <= 16,
                "Section names are 1 to 16 characters.",
            )?;
        }
        check(
            e.section.is_some() || e.child.is_none(),
            format!(
                "{}: pick a section before its recitation or lab.",
                e.course_id
            ),
        )?;
    }
    Ok(())
}

/// The term exists, each course is offered in it, `section` is one of its top-level sections there and `child` belongs
/// to `section`.
pub async fn check_catalog(
    catalog: &Catalog,
    term: String,
    entries: Vec<ScheduleEntry>,
) -> ApiResult<()> {
    catalog
        .with(move |r| {
            let db = r.connection();
            let term_known: bool = db.query_row(
                "SELECT EXISTS(SELECT 1 FROM terms WHERE id = ?)",
                [&term],
                |r| r.get(0),
            )?;
            check(term_known, format!("There's no term {term:?}."))?;
            let mut offered = db.prepare_cached(
                "SELECT EXISTS(SELECT 1 FROM offerings WHERE course_id = ? AND term_id = ?)",
            )?;
            let mut parent_of = db.prepare_cached(
                "SELECT parent FROM sections WHERE term_id = ? AND course_id = ? AND name = ?",
            )?;
            for e in &entries {
                let code = format!("{}-{}", &e.course_id[..2], &e.course_id[2..]);
                let is_offered: bool =
                    offered.query_row([&e.course_id, &term], |r| r.get(0))?;
                check(is_offered, format!("{code} isn't offered in {term}."))?;
                let Some(section) = &e.section else { continue };
                let parent: Option<Option<String>> = parent_of
                    .query_row([&term, &e.course_id, section], |r| r.get(0))
                    .optional()?;
                match parent {
                    None => {
                        return Err(ApiError::bad_request(format!(
                            "{code} has no section {section} in {term}."
                        )));
                    }
                    Some(Some(lecture)) => {
                        return Err(ApiError::bad_request(format!(
                            "{code} {section} belongs to {lecture}; choose {lecture} as the section and {section} as the child."
                        )));
                    }
                    Some(None) => {}
                }
                if let Some(child) = &e.child {
                    let parent: Option<Option<String>> = parent_of
                        .query_row([&term, &e.course_id, child], |r| r.get(0))
                        .optional()?;
                    check(
                        parent == Some(Some(section.clone())),
                        format!("{code} {child} isn't a section of {section} in {term}."),
                    )?;
                }
            }
            Ok(())
        })
        .await
}

/// The caller's schedules, oldest first.
#[utoipa::path(
    get,
    path = "/api/schedules",
    tag = "schedules",
    params(TermQuery),
    responses((status = OK, body = Vec<Schedule>), (status = UNAUTHORIZED, body = ErrorBody))
)]
pub async fn list_mine(
    user: CurrentUser,
    Extension(state): Extension<AppState>,
    Query(q): Query<TermQuery>,
) -> ApiResult<Json<Vec<Schedule>>> {
    state
        .db
        .run(move |db| list(db, &user.subject, q.term.as_deref(), false))
        .await
        .map(Json)
}

/// Creates a schedule at revision 1.
#[utoipa::path(
    post,
    path = "/api/schedules",
    tag = "schedules",
    request_body = NewSchedule,
    responses(
        (status = CREATED, body = Schedule),
        (status = BAD_REQUEST, description = "Invalid, or not in the catalog", body = ErrorBody),
        (status = CONFLICT, description = "A schedule with this id exists", body = ScheduleConflict),
    )
)]
pub async fn create(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Json(new): Json<NewSchedule>,
) -> ApiResult<(StatusCode, Json<Schedule>)> {
    let id = new_id(new.id)?;
    let name = check_name(&new.name)?;
    check_shape(&new.entries)?;
    check_catalog(&state.catalog, new.term_id.clone(), new.entries.clone()).await?;
    let body = to_stored(&Body {
        name,
        entries: new.entries,
    })?;
    let schedule = state
        .db
        .run(move |db| {
            let tx = db.transaction()?;
            if let Some((current, _)) = load(&tx, &user.subject, &id)? {
                return Err(ApiError::conflict(current));
            }
            let count: i64 = tx.query_row(
                "SELECT count(*) FROM user_schedules WHERE owner = ?",
                [&user.subject],
                |r| r.get(0),
            )?;
            check(
                count < SCHEDULES_MAX,
                format!("You can keep at most {SCHEDULES_MAX} schedules."),
            )?;
            let now = now_ms();
            tx.execute(
                "INSERT INTO user_schedules(owner, id, term_id, body, visibility, revision, created_at, updated_at, day, day_writes)
                 VALUES (?, ?, ?, ?, ?, 1, ?, ?, ?, 1)",
                params![user.subject, id, new.term_id, body, new.visibility.as_str(), now, now, utc_day(now)],
            )?;
            let created = load(&tx, &user.subject, &id)?.map(|(s, _)| s);
            tx.commit()?;
            created.ok_or_else(|| ApiError::internal("schedule vanished"))
        })
        .await?;
    Ok(created(schedule))
}

/// Replaces a schedule. A stale `revision` gets 409 with the current schedule.
#[utoipa::path(
    put,
    path = "/api/schedules/{id}",
    tag = "schedules",
    params(("id" = String, Path)),
    request_body = ScheduleUpdate,
    responses(
        (status = OK, body = Schedule),
        (status = BAD_REQUEST, body = ErrorBody),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, body = ScheduleConflict),
        (status = TOO_MANY_REQUESTS, description = "Over the daily write cap or the per-minute limit", body = ErrorBody),
    )
)]
pub async fn update(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
    Json(update): Json<ScheduleUpdate>,
) -> ApiResult<Json<Schedule>> {
    let name = check_name(&update.name)?;
    check_shape(&update.entries)?;
    check_catalog(
        &state.catalog,
        update.term_id.clone(),
        update.entries.clone(),
    )
    .await?;
    let body = to_stored(&Body {
        name,
        entries: update.entries,
    })?;
    let schedule = state
        .db
        .run(move |db| {
            let now = now_ms();
            let changed = db.execute(
                &format!(
                    "UPDATE user_schedules SET term_id = :term, body = :body, visibility = :visibility,
                       revision = revision + 1, updated_at = :now, {COUNT_WRITE}
                     WHERE owner = :owner AND id = :id AND revision = :base AND {UNDER_CAP}"
                ),
                named_params! {
                    ":term": update.term_id,
                    ":body": body,
                    ":visibility": update.visibility.as_str(),
                    ":now": now,
                    ":today": utc_day(now),
                    ":owner": user.subject,
                    ":id": id,
                    ":base": update.revision,
                    ":cap": DAILY_WRITES,
                },
            )?;
            let stored = load(db, &user.subject, &id)?;
            if changed == 1 {
                return stored
                    .map(|(s, _)| s)
                    .ok_or_else(|| ApiError::internal("schedule vanished"));
            }
            let (current, write) = match stored {
                Some((s, w)) => (Some(s), Some(w)),
                None => (None, None),
            };
            Err(write_failed(write, update.revision, || {
                Ok(serde_json::to_value(current)?)
            }))
        })
        .await?;
    Ok(Json(schedule))
}

/// Deletes a schedule and its share link.
#[utoipa::path(
    delete,
    path = "/api/schedules/{id}",
    tag = "schedules",
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
                "DELETE FROM user_schedules WHERE owner = ? AND id = ?",
                [&user.subject, &id],
            )?)
        })
        .await?;
    if removed == 0 {
        return Err(ApiError::not_found("No such schedule."));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Creates a read-only share link, or returns the existing one.
#[utoipa::path(
    post,
    path = "/api/schedules/{id}/share",
    tag = "schedules",
    params(("id" = String, Path)),
    responses((status = OK, body = ShareLink), (status = NOT_FOUND, body = ErrorBody))
)]
pub async fn share(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ShareLink>> {
    let origin = state.origin.clone();
    let share_id = state
        .db
        .run(move |db| {
            db.execute(
                "UPDATE user_schedules SET share_id = coalesce(share_id, ?) WHERE owner = ? AND id = ?",
                params![uuid::Uuid::new_v4().to_string(), user.subject, id],
            )?;
            load(db, &user.subject, &id)?
                .and_then(|(s, _)| s.share_id)
                .ok_or_else(|| ApiError::not_found("No such schedule."))
        })
        .await?;
    Ok(Json(ShareLink {
        url: format!("{origin}/schedules/shared/{share_id}"),
        share_id,
    }))
}

/// Revokes the share link.
#[utoipa::path(
    delete,
    path = "/api/schedules/{id}/share",
    tag = "schedules",
    params(("id" = String, Path)),
    responses((status = NO_CONTENT), (status = NOT_FOUND, body = ErrorBody))
)]
pub async fn unshare(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let changed = state
        .db
        .run(move |db| {
            Ok(db.execute(
                "UPDATE user_schedules SET share_id = NULL WHERE owner = ? AND id = ?",
                [&user.subject, &id],
            )?)
        })
        .await?;
    if changed == 0 {
        return Err(ApiError::not_found("No such schedule."));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// A schedule shared by link. Anyone with the link may read it; it never names its owner.
#[utoipa::path(
    get,
    path = "/api/shared-schedules/{token}",
    tag = "schedules",
    params(("token" = String, Path)),
    responses((status = OK, body = SharedSchedule), (status = NOT_FOUND, body = ErrorBody))
)]
pub async fn shared(
    Extension(state): Extension<AppState>,
    Path(token): Path<String>,
) -> ApiResult<Json<SharedSchedule>> {
    state
        .db
        .run(move |db| {
            let row = db
                .query_row(
                    "SELECT term_id, body, updated_at FROM user_schedules WHERE share_id = ?",
                    [&token],
                    |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, i64>(2)?,
                        ))
                    },
                )
                .optional()?;
            let (term_id, body, updated_at) =
                row.ok_or_else(|| ApiError::not_found("This link was revoked or never existed."))?;
            let body: Body = serde_json::from_str(&body)?;
            Ok(Json(SharedSchedule {
                term_id,
                name: body.name,
                entries: body.entries,
                updated_at,
            }))
        })
        .await
}
