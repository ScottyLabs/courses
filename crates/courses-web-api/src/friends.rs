//! Friends: requests by Andrew ID or invite link, accepted by the other person. Friendship alone shares nothing; each
//! student separately chooses whether accepted friends see their courses and their friends-visible schedules, and
//! can change that or remove a friend at any time.

use std::collections::{BTreeMap, BTreeSet};

use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::StatusCode,
};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    auth::{CurrentUser, Writer},
    error::{ApiError, ApiResult, ErrorBody},
    records::{
        documents::{RecordData, RecordSource, read},
        is_term,
        schedules::{self, ScheduleEntry},
    },
    state::AppState,
};

const INVITE_DAYS: i64 = 7;

/// Records the caller's Andrew ID and name so friends can find them. A stale row holding the same Andrew ID (the
/// issuer changed the subject) gives the ID up.
pub async fn register_profile(state: &AppState, user: &CurrentUser) -> ApiResult<()> {
    let user = user.clone();
    state
        .db
        .run(move |db| {
            let tx = db.transaction()?;
            tx.execute(
                "UPDATE profiles SET andrew_id = andrew_id || '~' || subject WHERE andrew_id = ? AND subject <> ?",
                params![user.andrew_id, user.subject],
            )?;
            tx.execute(
                "INSERT INTO profiles(subject, andrew_id, name) VALUES (?, ?, ?)
                 ON CONFLICT(subject) DO UPDATE SET andrew_id = excluded.andrew_id, name = excluded.name",
                params![user.subject, user.andrew_id, user.name],
            )?;
            tx.commit()?;
            Ok(())
        })
        .await
}

#[derive(Clone, Copy, Serialize, ToSchema, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum FriendStatus {
    Accepted,
    /// They asked you.
    Incoming,
    /// You asked them.
    Outgoing,
}

#[derive(Serialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Friend {
    /// The friendship's id, used by the other friend routes.
    pub id: String,
    pub name: String,
    pub andrew_id: String,
    pub status: FriendStatus,
    /// An accepted friend shows you the courses they're taking.
    pub shares_courses: bool,
    /// An accepted friend shows you their schedules marked visible to friends.
    pub shares_schedules: bool,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FriendRequest {
    pub andrew_id: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Invite {
    pub token: String,
    /// The website's page that redeems it.
    pub url: String,
    /// Milliseconds since the epoch.
    pub expires_at: i64,
}

/// Who made an invite, for the invite page before the invite is accepted.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InviteInfo {
    /// The name of the student who made the invite.
    pub name: String,
    pub andrew_id: String,
    /// Milliseconds since the epoch.
    pub expires_at: i64,
    /// The caller made this invite.
    pub own: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Sharing {
    /// Accepted friends see the courses you're taking, from your record and friends-visible schedules.
    pub share_courses: bool,
    /// Accepted friends see your schedules marked visible to friends.
    pub share_schedules: bool,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FriendSchedule {
    pub id: String,
    pub term_id: String,
    pub name: String,
    pub entries: Vec<ScheduleEntry>,
    pub updated_at: i64,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct FriendTermQuery {
    /// A term, such as `S27`.
    pub term: Option<String>,
}

fn friend_list(db: &Connection, me: &str) -> ApiResult<Vec<Friend>> {
    let mut stmt = db.prepare_cached(
        "SELECT f.id, f.requester, f.accepted, p.name, p.andrew_id, p.share_courses, p.share_schedules
         FROM friendships f
         JOIN profiles p ON p.subject = CASE WHEN f.requester = ?1 THEN f.recipient ELSE f.requester END
         WHERE f.requester = ?1 OR f.recipient = ?1
         ORDER BY f.accepted DESC, p.name, f.id",
    )?;
    let rows = stmt.query_map([me], |r| {
        let requester: String = r.get(1)?;
        let accepted: bool = r.get(2)?;
        let status = if accepted {
            FriendStatus::Accepted
        } else if requester == me {
            FriendStatus::Outgoing
        } else {
            FriendStatus::Incoming
        };
        Ok(Friend {
            id: r.get(0)?,
            name: r.get(3)?,
            andrew_id: r.get(4)?,
            status,
            shares_courses: accepted && r.get::<_, bool>(5)?,
            shares_schedules: accepted && r.get::<_, bool>(6)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Accepted friends and pending requests either way.
#[utoipa::path(
    get,
    path = "/api/friends",
    tag = "friends",
    responses((status = OK, body = Vec<Friend>), (status = UNAUTHORIZED, body = ErrorBody))
)]
pub async fn list(
    user: CurrentUser,
    Extension(state): Extension<AppState>,
) -> ApiResult<Json<Vec<Friend>>> {
    register_profile(&state, &user).await?;
    state
        .db
        .run(move |db| friend_list(db, &user.subject))
        .await
        .map(Json)
}

/// Creates a pending request; `requester` and `recipient` are subjects.
fn request(db: &Connection, requester: &str, recipient: &str) -> ApiResult<String> {
    if requester == recipient {
        return Err(ApiError::bad_request("You can't add yourself."));
    }
    let pair = if requester < recipient {
        format!("{requester}\n{recipient}")
    } else {
        format!("{recipient}\n{requester}")
    };
    let exists: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM friendships WHERE pair = ?)",
        [&pair],
        |r| r.get(0),
    )?;
    if exists {
        return Err(ApiError::bad_request(
            "You're already friends, or a request is waiting.",
        ));
    }
    let id = uuid::Uuid::new_v4().to_string();
    db.execute(
        "INSERT INTO friendships(id, pair, requester, recipient, accepted) VALUES (?, ?, ?, ?, 0)",
        params![id, pair, requester, recipient],
    )?;
    Ok(id)
}

/// Asks someone, by Andrew ID, to be friends. They must have signed in once.
#[utoipa::path(
    post,
    path = "/api/friends",
    tag = "friends",
    request_body = FriendRequest,
    responses((status = OK, body = Friend), (status = BAD_REQUEST, body = ErrorBody))
)]
pub async fn send_request(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Json(body): Json<FriendRequest>,
) -> ApiResult<Json<Friend>> {
    register_profile(&state, &user).await?;
    let andrew = body.andrew_id.trim().to_ascii_lowercase();
    let andrew = andrew.split('@').next().unwrap_or_default().to_owned();
    if andrew.is_empty()
        || andrew.len() > 64
        || !andrew
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(ApiError::bad_request("That isn't an Andrew ID."));
    }
    state
        .db
        .run(move |db| {
            let target: Option<String> = db
                .query_row(
                    "SELECT subject FROM profiles WHERE andrew_id = ?",
                    [&andrew],
                    |r| r.get(0),
                )
                .optional()?;
            let target = target.ok_or_else(|| {
                ApiError::bad_request(
                    "No one with that Andrew ID has signed in yet; send them an invite link instead.",
                )
            })?;
            let id = request(db, &user.subject, &target)?;
            friend_list(db, &user.subject)?
                .into_iter()
                .find(|f| f.id == id)
                .map(Json)
                .ok_or_else(|| ApiError::internal("friend request vanished"))
        })
        .await
}

/// Accepts an incoming request. Only its recipient can.
#[utoipa::path(
    post,
    path = "/api/friends/{id}/accept",
    tag = "friends",
    params(("id" = String, Path, description = "Friendship id")),
    responses((status = NO_CONTENT), (status = NOT_FOUND, body = ErrorBody))
)]
pub async fn accept(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let changed = state
        .db
        .run(move |db| {
            Ok(db.execute(
                "UPDATE friendships SET accepted = 1 WHERE id = ? AND recipient = ? AND accepted = 0",
                params![id, user.subject],
            )?)
        })
        .await?;
    if changed == 0 {
        return Err(ApiError::not_found("No such request."));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Declines, cancels or ends a friendship. Either person can, and access ends at once.
#[utoipa::path(
    delete,
    path = "/api/friends/{id}",
    tag = "friends",
    params(("id" = String, Path, description = "Friendship id")),
    responses((status = NO_CONTENT), (status = NOT_FOUND, body = ErrorBody))
)]
pub async fn remove(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let changed = state
        .db
        .run(move |db| {
            Ok(db.execute(
                "DELETE FROM friendships WHERE id = ?1 AND (requester = ?2 OR recipient = ?2)",
                params![id, user.subject],
            )?)
        })
        .await?;
    if changed == 0 {
        return Err(ApiError::not_found("No such friend."));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Creates a single-use invite link valid for seven days, replacing the caller's previous one.
#[utoipa::path(
    post,
    path = "/api/friends/invites",
    tag = "friends",
    responses((status = OK, body = Invite))
)]
pub async fn invite(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
) -> ApiResult<Json<Invite>> {
    register_profile(&state, &user).await?;
    let token = uuid::Uuid::new_v4().simple().to_string();
    let expires = Utc::now() + chrono::Duration::days(INVITE_DAYS);
    let origin = state.origin.clone();
    let stored = token.clone();
    state
        .db
        .run(move |db| {
            db.execute(
                "DELETE FROM friend_invites WHERE owner = ? OR expires_at < ?",
                params![user.subject, Utc::now().timestamp()],
            )?;
            db.execute(
                "INSERT INTO friend_invites(token, owner, expires_at) VALUES (?, ?, ?)",
                params![stored, user.subject, expires.timestamp()],
            )?;
            Ok(())
        })
        .await?;
    Ok(Json(Invite {
        url: format!("{origin}/invite/{token}"),
        token,
        expires_at: expires.timestamp_millis(),
    }))
}

/// Who made an invite, so the invite page can say who it's from. Seeing it doesn't use the invite up.
#[utoipa::path(
    get,
    path = "/api/friends/invites/{token}",
    tag = "friends",
    params(("token" = String, Path)),
    responses((status = OK, body = InviteInfo), (status = NOT_FOUND, body = ErrorBody), (status = UNAUTHORIZED, body = ErrorBody))
)]
pub async fn invite_info(
    user: CurrentUser,
    Extension(state): Extension<AppState>,
    Path(token): Path<String>,
) -> ApiResult<Json<InviteInfo>> {
    state
        .db
        .run(move |db| {
            let found: Option<(String, String, i64, String)> = db
                .query_row(
                    "SELECT p.name, p.andrew_id, i.expires_at, i.owner FROM friend_invites i
                     JOIN profiles p ON p.subject = i.owner
                     WHERE i.token = ? AND i.expires_at > ?",
                    params![token, Utc::now().timestamp()],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
                .optional()?;
            let (name, andrew_id, expires_at, owner) =
                found.ok_or_else(|| ApiError::not_found("This invite expired or was used."))?;
            Ok(Json(InviteInfo {
                name,
                andrew_id,
                expires_at: expires_at * 1000,
                own: owner == user.subject,
            }))
        })
        .await
}

/// Accepts an invite: the two become friends at once and the link is used up.
#[utoipa::path(
    post,
    path = "/api/friends/invites/{token}/redeem",
    tag = "friends",
    params(("token" = String, Path)),
    responses((status = OK, body = Friend), (status = NOT_FOUND, body = ErrorBody), (status = BAD_REQUEST, body = ErrorBody))
)]
pub async fn redeem(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Path(token): Path<String>,
) -> ApiResult<Json<Friend>> {
    register_profile(&state, &user).await?;
    state
        .db
        .run(move |db| {
            let tx = db.transaction()?;
            let owner: Option<String> = tx
                .query_row(
                    "SELECT owner FROM friend_invites WHERE token = ? AND expires_at > ?",
                    params![token, Utc::now().timestamp()],
                    |r| r.get(0),
                )
                .optional()?;
            let owner =
                owner.ok_or_else(|| ApiError::not_found("This invite expired or was used."))?;
            let id = request(&tx, &owner, &user.subject)?;
            tx.execute("UPDATE friendships SET accepted = 1 WHERE id = ?", [&id])?;
            tx.execute("DELETE FROM friend_invites WHERE token = ?", [&token])?;
            let friend = friend_list(&tx, &user.subject)?
                .into_iter()
                .find(|f| f.id == id);
            tx.commit()?;
            friend
                .map(Json)
                .ok_or_else(|| ApiError::internal("friendship vanished"))
        })
        .await
}

/// Revokes one of the caller's invites.
#[utoipa::path(
    delete,
    path = "/api/friends/invites/{token}",
    tag = "friends",
    params(("token" = String, Path)),
    responses((status = NO_CONTENT))
)]
pub async fn revoke_invite(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Path(token): Path<String>,
) -> ApiResult<StatusCode> {
    state
        .db
        .run(move |db| {
            db.execute(
                "DELETE FROM friend_invites WHERE token = ? AND owner = ?",
                params![token, user.subject],
            )?;
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

/// What the caller shows accepted friends.
#[utoipa::path(get, path = "/api/friends/sharing", tag = "friends", responses((status = OK, body = Sharing)))]
pub async fn get_sharing(
    user: CurrentUser,
    Extension(state): Extension<AppState>,
) -> ApiResult<Json<Sharing>> {
    register_profile(&state, &user).await?;
    state
        .db
        .run(move |db| {
            Ok(Json(db.query_row(
                "SELECT share_courses, share_schedules FROM profiles WHERE subject = ?",
                [&user.subject],
                |r| {
                    Ok(Sharing {
                        share_courses: r.get(0)?,
                        share_schedules: r.get(1)?,
                    })
                },
            )?))
        })
        .await
}

/// Changes what the caller shows accepted friends. Accepting a friendship never changes this.
#[utoipa::path(
    put,
    path = "/api/friends/sharing",
    tag = "friends",
    request_body = Sharing,
    responses((status = OK, body = Sharing))
)]
pub async fn set_sharing(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    Json(sharing): Json<Sharing>,
) -> ApiResult<Json<Sharing>> {
    register_profile(&state, &user).await?;
    state
        .db
        .run(move |db| {
            db.execute(
                "UPDATE profiles SET share_courses = ?, share_schedules = ? WHERE subject = ?",
                params![sharing.share_courses, sharing.share_schedules, user.subject],
            )?;
            Ok(Json(sharing))
        })
        .await
}

/// An accepted friend's subject, sharing flags and name, by friendship id; `None` unless the caller is in it.
fn accepted_friend(
    db: &Connection,
    me: &str,
    friendship: &str,
) -> ApiResult<Option<(String, bool, bool, String)>> {
    Ok(db
        .query_row(
            "SELECT p.subject, p.share_courses, p.share_schedules, p.name FROM friendships f
             JOIN profiles p ON p.subject = CASE WHEN f.requester = ?2 THEN f.recipient ELSE f.requester END
             WHERE f.id = ?1 AND f.accepted = 1 AND (f.requester = ?2 OR f.recipient = ?2)",
            params![friendship, me],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?)
}

/// An accepted friend's schedules marked visible to friends; empty unless they share schedules.
#[utoipa::path(
    get,
    path = "/api/friends/{id}/schedules",
    tag = "friends",
    params(("id" = String, Path, description = "Friendship id"), FriendTermQuery),
    responses((status = OK, body = Vec<FriendSchedule>), (status = NOT_FOUND, body = ErrorBody))
)]
pub async fn friend_schedules(
    user: CurrentUser,
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
    Query(q): Query<FriendTermQuery>,
) -> ApiResult<Json<Vec<FriendSchedule>>> {
    state
        .db
        .run(move |db| {
            let (friend, _, shares_schedules, _) = accepted_friend(db, &user.subject, &id)?
                .ok_or_else(|| ApiError::not_found("No such friend."))?;
            if !shares_schedules {
                return Ok(Json(vec![]));
            }
            let list = schedules::list(db, &friend, q.term.as_deref(), true)?;
            Ok(Json(
                list.into_iter()
                    .map(|s| FriendSchedule {
                        id: s.id,
                        term_id: s.term_id,
                        name: s.name,
                        entries: s.entries,
                        updated_at: s.updated_at,
                    })
                    .collect(),
            ))
        })
        .await
}

/// Course ids a friend is taking in `term`, from what they share: friends-visible schedules (courses not hidden) when
/// they share schedules, and their record's in-progress courses (taken, no grade yet) when they share courses.
fn taking(
    db: &Connection,
    friend: &str,
    term: &str,
    courses: bool,
    schedules_too: bool,
) -> ApiResult<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    if schedules_too {
        for s in schedules::list(db, friend, Some(term), true)? {
            ids.extend(
                s.entries
                    .into_iter()
                    .filter(|e| !e.hidden)
                    .map(|e| e.course_id),
            );
        }
    }
    if courses {
        let (record, _, _) = read::<RecordData>(db, friend)?;
        ids.extend(record.entries.into_iter().filter_map(|e| {
            let in_progress = e.source == RecordSource::Taken
                && e.term.as_deref() == Some(term)
                && e.grade.as_deref().is_none_or(|g| g.trim().is_empty());
            in_progress.then_some(e.course_id).flatten()
        }));
    }
    Ok(ids)
}

/// Accepted friends taking each course in `term`: course id to friends' names.
#[utoipa::path(
    get,
    path = "/api/friends/courses",
    tag = "friends",
    params(FriendTermQuery),
    responses((status = OK, body = BTreeMap<String, Vec<String>>), (status = BAD_REQUEST, body = ErrorBody))
)]
pub async fn friends_courses(
    user: CurrentUser,
    Extension(state): Extension<AppState>,
    Query(q): Query<FriendTermQuery>,
) -> ApiResult<Json<BTreeMap<String, Vec<String>>>> {
    let term = q
        .term
        .filter(|t| is_term(t))
        .ok_or_else(|| ApiError::bad_request("term must be a term like S27."))?;
    state
        .db
        .run(move |db| {
            let mut stmt = db.prepare_cached(
                "SELECT p.subject, p.name, p.share_courses, p.share_schedules FROM friendships f
                 JOIN profiles p ON p.subject = CASE WHEN f.requester = ?1 THEN f.recipient ELSE f.requester END
                 WHERE (f.requester = ?1 OR f.recipient = ?1) AND f.accepted = 1
                   AND (p.share_courses = 1 OR p.share_schedules = 1)",
            )?;
            let friends = stmt
                .query_map([&user.subject], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, bool>(2)?,
                        r.get::<_, bool>(3)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            let mut by_course: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for (subject, name, courses, schedules_too) in friends {
                for course in taking(db, &subject, &term, courses, schedules_too)? {
                    by_course.entry(course).or_default().push(name.clone());
                }
            }
            for names in by_course.values_mut() {
                names.sort();
            }
            Ok(Json(by_course))
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_and_self_requests_are_rejected() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE friendships(id TEXT,pair TEXT UNIQUE,requester TEXT,recipient TEXT,accepted INTEGER);").unwrap();
        assert!(request(&db, "alice", "alice").is_err());
        request(&db, "alice", "bob").unwrap();
        assert!(request(&db, "bob", "alice").is_err());
        assert!(
            !db.query_row("SELECT accepted FROM friendships", [], |r| r
                .get::<_, bool>(0))
                .unwrap()
        );
    }
}
