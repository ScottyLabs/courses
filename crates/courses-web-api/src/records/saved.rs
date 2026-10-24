//! Saved courses (bookmarks), not tied to a semester.

use axum::{Extension, Json, extract::Path, http::StatusCode};
use rusqlite::params;
use serde::Serialize;
use utoipa::ToSchema;

use super::{check, is_course_id};
use crate::{
    auth::{CurrentUser, Writer},
    db::now_ms,
    error::{ApiError, ApiResult, ErrorBody},
    state::AppState,
};

const SAVED_MAX: i64 = 1000;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SavedCourse {
    pub course_id: String,
    /// Milliseconds since the epoch.
    pub saved_at: i64,
}

/// The caller's saved courses, oldest first.
#[utoipa::path(
    get,
    path = "/api/saved",
    tag = "saved",
    responses((status = OK, body = Vec<SavedCourse>), (status = UNAUTHORIZED, body = ErrorBody))
)]
pub async fn list(
    user: CurrentUser,
    Extension(state): Extension<AppState>,
) -> ApiResult<Json<Vec<SavedCourse>>> {
    state
        .db
        .run(move |db| {
            let mut stmt = db.prepare_cached(
                "SELECT course_id, saved_at FROM user_saved WHERE owner = ? ORDER BY saved_at, course_id",
            )?;
            let rows = stmt.query_map([&user.subject], |r| {
                Ok(SavedCourse {
                    course_id: r.get(0)?,
                    saved_at: r.get(1)?,
                })
            })?;
            Ok(Json(rows.collect::<Result<_, _>>()?))
        })
        .await
}

/// Saves a course. Saving it again changes nothing.
#[utoipa::path(
    put,
    path = "/api/saved/{courseId}",
    tag = "saved",
    params(("courseId" = String, Path, description = "Five-digit course id, such as `15122`")),
    responses((status = NO_CONTENT), (status = BAD_REQUEST, body = ErrorBody))
)]
pub async fn save(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Path(course_id): Path<String>,
) -> ApiResult<StatusCode> {
    check(is_course_id(&course_id), "Course ids look like 15122.")?;
    let id = course_id.clone();
    let known = state
        .catalog
        .with(move |r| {
            Ok(r.connection().query_row(
                "SELECT EXISTS(SELECT 1 FROM courses WHERE id = ?)",
                [&id],
                |r| r.get::<_, bool>(0),
            )?)
        })
        .await?;
    check(known, "There's no such course in the catalog.")?;
    state
        .db
        .run(move |db| {
            let (count, saved): (i64, bool) = db.query_row(
                "SELECT count(*), coalesce(sum(course_id = ?2), 0) > 0 FROM user_saved WHERE owner = ?1",
                params![user.subject, course_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            if count >= SAVED_MAX && !saved {
                return Err(ApiError::bad_request(format!(
                    "You can save at most {SAVED_MAX} courses."
                )));
            }
            db.execute(
                "INSERT INTO user_saved(owner, course_id, saved_at) VALUES (?, ?, ?) ON CONFLICT DO NOTHING",
                params![user.subject, course_id, now_ms()],
            )?;
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

/// Un-saves a course. Removing one that isn't saved changes nothing.
#[utoipa::path(
    delete,
    path = "/api/saved/{courseId}",
    tag = "saved",
    params(("courseId" = String, Path)),
    responses((status = NO_CONTENT))
)]
pub async fn remove(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Path(course_id): Path<String>,
) -> ApiResult<StatusCode> {
    state
        .db
        .run(move |db| {
            db.execute(
                "DELETE FROM user_saved WHERE owner = ? AND course_id = ?",
                [&user.subject, &course_id],
            )?;
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}
