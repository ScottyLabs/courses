//! Database foundation, request guard and shared endpoint skeleton error.
use crate::auth::CurrentUser;
use axum::{
    Extension, Json,
    extract::{Path, Query, Request},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use rusqlite::Connection;
use serde_json::{Value, json};
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
pub(crate) fn not_implemented() -> ApiError {
    ApiError(
        StatusCode::NOT_IMPLEMENTED,
        "Backend endpoint not implemented".into(),
    )
}
#[derive(Clone)]
pub struct Backend {
    pub catalog: PathBuf,
    // Reserved for endpoint implementations; initialization remains active.
    #[allow(dead_code)]
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
CREATE TABLE IF NOT EXISTS final_imports(owner TEXT NOT NULL,term_id TEXT NOT NULL,payload TEXT NOT NULL,PRIMARY KEY(owner,term_id));
CREATE TABLE IF NOT EXISTS profiles(subject TEXT PRIMARY KEY,andrew_id TEXT NOT NULL UNIQUE,name TEXT NOT NULL,share_courses INTEGER NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS friendships(id TEXT PRIMARY KEY,pair TEXT NOT NULL UNIQUE,requester TEXT NOT NULL,recipient TEXT NOT NULL,accepted INTEGER NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS friend_invites(token TEXT PRIMARY KEY,owner TEXT NOT NULL,expires_at INTEGER NOT NULL);
PRAGMA user_version=3;")?;
        Ok(Self {
            catalog,
            users: Arc::new(Mutex::new(db)),
            origin: origin.trim_end_matches('/').to_owned(),
        })
    }
    pub fn ready(&self) -> Result<(), ApiError> {
        for member in [false, true] {
            let count: i64 =
                self.db(member)?
                    .query_row("SELECT count(*) FROM catalog_terms", [], |r| r.get(0))?;
            if count == 0 {
                return Err(ApiError(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Catalog has no terms".into(),
                ));
            }
        }
        for name in [
            "public-browser.sqlite",
            "member-browser.sqlite",
            "public-browser.sqlite.gz",
            "member-browser.sqlite.gz",
        ] {
            if !self.catalog.join(name).is_file() {
                return Err(ApiError(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Browser catalog is missing".into(),
                ));
            }
        }
        Ok(())
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

mod catalog;
mod personal;
pub use catalog::*;
pub use personal::*;

#[cfg(test)]
mod tests;

mod finals;
pub use finals::*;

mod friends;
pub use friends::*;
