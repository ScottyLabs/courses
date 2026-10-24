//! Sessions in the personal database (ADR 23), so a restart doesn't sign everyone out.
//!
//! Rows are keyed by the SHA-256 of the session id, so the database alone can't be replayed as cookies. The data holds
//! `axum-oidc`'s tokens; Keycloak's token lifetimes still bound how long a session stays signed in, since an expired
//! ID token is refreshed or the session is treated as signed out.

use async_trait::async_trait;
use rusqlite::{OptionalExtension, params};
use sha2::{Digest, Sha256};
use tower_sessions::{
    SessionStore,
    cookie::time::OffsetDateTime,
    session::{Id, Record},
    session_store,
};

use crate::{auth::SESSION_SUBJECT, db::UserDb, error::ApiError};

#[derive(Clone, Debug)]
pub struct SqliteStore {
    db: UserDb,
}

impl SqliteStore {
    pub fn new(db: UserDb) -> Self {
        Self { db }
    }

    /// Deletes expired sessions now, then hourly.
    pub fn spawn_cleanup(&self) -> tokio::task::JoinHandle<()> {
        let db = self.db.clone();
        tokio::spawn(async move {
            let mut every = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                every.tick().await;
                let removed = db
                    .run(|db| {
                        Ok(db.execute(
                            "DELETE FROM sessions WHERE expires_at <= ?",
                            [OffsetDateTime::now_utc().unix_timestamp()],
                        )?)
                    })
                    .await;
                match removed {
                    Ok(n) if n > 0 => tracing::info!(removed = n, "deleted expired sessions"),
                    Ok(_) => {}
                    Err(e) => tracing::warn!(error = %e.message, "session cleanup failed"),
                }
            }
        })
    }
}

fn key(id: &Id) -> String {
    format!("{:x}", Sha256::digest(id.to_string().as_bytes()))
}

fn backend(e: ApiError) -> session_store::Error {
    session_store::Error::Backend(e.message)
}

fn row(record: &Record) -> session_store::Result<(String, Option<String>, String, i64)> {
    let owner = record
        .data
        .get(SESSION_SUBJECT)
        .and_then(|v| v.as_str())
        .map(str::to_owned);
    let data = serde_json::to_string(&record.data)
        .map_err(|e| session_store::Error::Encode(e.to_string()))?;
    Ok((
        key(&record.id),
        owner,
        data,
        record.expiry_date.unix_timestamp(),
    ))
}

#[async_trait]
impl SessionStore for SqliteStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        loop {
            let (id, owner, data, expires) = row(record)?;
            let inserted = self
                .db
                .run(move |db| {
                    Ok(db.execute(
                        "INSERT INTO sessions(id_hash, owner, data, expires_at) VALUES (?, ?, ?, ?)
                         ON CONFLICT(id_hash) DO NOTHING",
                        params![id, owner, data, expires],
                    )?)
                })
                .await
                .map_err(backend)?;
            if inserted == 1 {
                return Ok(());
            }
            record.id = Id::default();
        }
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let (id, owner, data, expires) = row(record)?;
        self.db
            .run(move |db| {
                db.execute(
                    "INSERT INTO sessions(id_hash, owner, data, expires_at) VALUES (?, ?, ?, ?)
                     ON CONFLICT(id_hash) DO UPDATE SET owner = excluded.owner, data = excluded.data,
                       expires_at = excluded.expires_at",
                    params![id, owner, data, expires],
                )?;
                Ok(())
            })
            .await
            .map_err(backend)
    }

    async fn load(&self, id: &Id) -> session_store::Result<Option<Record>> {
        let hash = key(id);
        let row = self
            .db
            .run(move |db| {
                Ok(db
                    .query_row(
                        "SELECT data, expires_at FROM sessions WHERE id_hash = ? AND expires_at > ?",
                        params![hash, OffsetDateTime::now_utc().unix_timestamp()],
                        |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)),
                    )
                    .optional()?)
            })
            .await
            .map_err(backend)?;
        let Some((data, expires)) = row else {
            return Ok(None);
        };
        Ok(Some(Record {
            id: *id,
            data: serde_json::from_str(&data)
                .map_err(|e| session_store::Error::Decode(e.to_string()))?,
            expiry_date: OffsetDateTime::from_unix_timestamp(expires)
                .map_err(|e| session_store::Error::Decode(e.to_string()))?,
        }))
    }

    async fn delete(&self, id: &Id) -> session_store::Result<()> {
        let hash = key(id);
        self.db
            .run(move |db| {
                db.execute("DELETE FROM sessions WHERE id_hash = ?", [hash])?;
                Ok(())
            })
            .await
            .map_err(backend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_sessions::cookie::time::Duration;

    fn record(subject: Option<&str>, expires_in: Duration) -> Record {
        let mut data = std::collections::HashMap::new();
        if let Some(s) = subject {
            data.insert(SESSION_SUBJECT.to_owned(), serde_json::json!(s));
        }
        data.insert("k".to_owned(), serde_json::json!({"v": 1}));
        Record {
            id: Id::default(),
            data,
            expiry_date: OffsetDateTime::now_utc() + expires_in,
        }
    }

    #[tokio::test]
    async fn sessions_survive_reopening_and_expire() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("users.sqlite");
        let store = SqliteStore::new(UserDb::open(&path).unwrap());
        let mut live = record(Some("alice"), Duration::hours(1));
        store.create(&mut live).await.unwrap();
        let mut old = record(None, Duration::seconds(-5));
        store.create(&mut old).await.unwrap();

        let reopened = SqliteStore::new(UserDb::open(&path).unwrap());
        let loaded = reopened.load(&live.id).await.unwrap().unwrap();
        assert_eq!(loaded.data["k"], serde_json::json!({"v": 1}));
        assert!(reopened.load(&old.id).await.unwrap().is_none());

        // Ids are stored hashed, with the owner alongside.
        let (owner, hashed): (Option<String>, String) = reopened
            .db
            .run(|db| {
                Ok(db.query_row(
                    "SELECT owner, id_hash FROM sessions WHERE owner IS NOT NULL",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )?)
            })
            .await
            .unwrap();
        assert_eq!(owner.as_deref(), Some("alice"));
        assert_ne!(hashed, live.id.to_string());

        reopened.delete(&live.id).await.unwrap();
        assert!(reopened.load(&live.id).await.unwrap().is_none());
    }
}
