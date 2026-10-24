//! Deleting an account: every record the caller owns, their friendships, invites, profile and sessions.

use axum::{Extension, http::StatusCode};
use tower_sessions::Session;

use crate::{
    auth::Writer,
    error::{ApiResult, ErrorBody},
    state::AppState,
};

/// Each table holding personal data, with its owner column.
const OWNED: [(&str, &str); 10] = [
    ("user_schedules", "owner"),
    ("user_plans", "owner"),
    ("user_documents", "owner"),
    ("user_saved", "owner"),
    ("final_imports", "owner"),
    ("friend_invites", "owner"),
    ("schedules", "owner"),
    ("saved_courses", "owner"),
    ("sessions", "owner"),
    ("profiles", "subject"),
];

/// Deletes all of the caller's data and signs out all of their sessions.
#[utoipa::path(
    delete,
    path = "/api/account",
    tag = "auth",
    responses((status = NO_CONTENT), (status = UNAUTHORIZED, body = ErrorBody))
)]
pub async fn delete(
    Writer(user): Writer,
    Extension(state): Extension<AppState>,
    session: Option<Extension<Session>>,
) -> ApiResult<StatusCode> {
    let subject = user.subject.clone();
    state
        .db
        .run(move |db| {
            let tx = db.transaction()?;
            tx.execute(
                "DELETE FROM friendships WHERE requester = ?1 OR recipient = ?1",
                [&subject],
            )?;
            for (table, column) in OWNED {
                tx.execute(
                    &format!("DELETE FROM {table} WHERE {column} = ?"),
                    [&subject],
                )?;
            }
            tx.commit()?;
            Ok(())
        })
        .await?;
    if let Some(Extension(session)) = session
        && let Err(e) = session.flush().await
    {
        tracing::warn!(error = %e, "flushing the session after account deletion failed");
    }
    Ok(StatusCode::NO_CONTENT)
}
