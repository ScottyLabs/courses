//! The personal SQLite database: one connection behind a mutex, used from blocking threads, and its migrations.
//!
//! Every query that touches personal data is scoped to its owner (the OIDC subject) in its SQL.

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::{Context, bail};
use rusqlite::{Connection, Transaction, params};

use crate::error::{ApiError, ApiResult};

/// `PRAGMA user_version` after [`migrate`].
pub const SCHEMA_VERSION: i64 = 4;

#[derive(Clone)]
pub struct UserDb {
    conn: Arc<Mutex<Connection>>,
}

impl std::fmt::Debug for UserDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UserDb")
    }
}

impl UserDb {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        let mut conn =
            Connection::open(path).with_context(|| format!("opening {}", path.display()))?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        migrate(&mut conn).with_context(|| format!("migrating {}", path.display()))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Runs `f` with the connection on a blocking thread.
    pub async fn run<T, F>(&self, f: F) -> ApiResult<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> ApiResult<T> + Send + 'static,
    {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = conn
                .lock()
                .map_err(|_| ApiError::internal("user database mutex poisoned"))?;
            f(&mut conn)
        })
        .await
        .map_err(ApiError::internal)?
    }

    /// Health: a committed write proves the file is writable.
    pub async fn check_writable(&self) -> ApiResult<()> {
        self.run(|db| {
            db.execute(
                "INSERT INTO health(id, checked_at) VALUES (1, ?) ON CONFLICT(id) DO UPDATE SET checked_at = excluded.checked_at",
                [now_ms()],
            )?;
            Ok(())
        })
        .await
    }
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// UTC day number, for daily write caps.
pub fn utc_day(ms: i64) -> i64 {
    ms.div_euclid(86_400_000)
}

/// The schema the first backend shipped, unchanged. New databases start here too, so every database takes the same
/// path to the current version.
const V3: &str = "
CREATE TABLE IF NOT EXISTS schedules(id TEXT PRIMARY KEY,owner TEXT NOT NULL,payload TEXT NOT NULL,revision INTEGER NOT NULL DEFAULT 1,share_id TEXT UNIQUE);
CREATE INDEX IF NOT EXISTS schedule_owner ON schedules(owner);
CREATE TABLE IF NOT EXISTS saved_courses(owner TEXT NOT NULL,code TEXT NOT NULL,status TEXT NOT NULL,term_id TEXT NOT NULL DEFAULT '',PRIMARY KEY(owner,code,status,term_id));
CREATE TABLE IF NOT EXISTS final_imports(owner TEXT NOT NULL,term_id TEXT NOT NULL,payload TEXT NOT NULL,PRIMARY KEY(owner,term_id));
CREATE TABLE IF NOT EXISTS profiles(subject TEXT PRIMARY KEY,andrew_id TEXT NOT NULL UNIQUE,name TEXT NOT NULL,share_courses INTEGER NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS friendships(id TEXT PRIMARY KEY,pair TEXT NOT NULL UNIQUE,requester TEXT NOT NULL,recipient TEXT NOT NULL,accepted INTEGER NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS friend_invites(token TEXT PRIMARY KEY,owner TEXT NOT NULL,expires_at INTEGER NOT NULL);
PRAGMA user_version=3;
";

/// Courseweave's records (decision 17), persisted sessions and schedule sharing with friends. Additive: the v3
/// `schedules` and `saved_courses` tables stay as they were.
const V4: &str = "
CREATE TABLE user_schedules(
  owner TEXT NOT NULL,
  id TEXT NOT NULL,
  term_id TEXT NOT NULL,
  body TEXT NOT NULL,
  visibility TEXT NOT NULL DEFAULT 'private' CHECK(visibility IN ('private', 'friends')),
  share_id TEXT UNIQUE,
  revision INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  day INTEGER NOT NULL DEFAULT 0,
  day_writes INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY(owner, id)
);
CREATE INDEX user_schedules_term ON user_schedules(owner, term_id);
CREATE TABLE user_plans(
  owner TEXT NOT NULL,
  id TEXT NOT NULL,
  body TEXT NOT NULL,
  revision INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  day INTEGER NOT NULL DEFAULT 0,
  day_writes INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY(owner, id)
);
CREATE TABLE user_documents(
  owner TEXT NOT NULL,
  kind TEXT NOT NULL CHECK(kind IN ('record', 'preferences')),
  body TEXT NOT NULL,
  revision INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  day INTEGER NOT NULL DEFAULT 0,
  day_writes INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY(owner, kind)
);
CREATE TABLE user_saved(
  owner TEXT NOT NULL,
  course_id TEXT NOT NULL,
  saved_at INTEGER NOT NULL,
  PRIMARY KEY(owner, course_id)
);
CREATE TABLE sessions(
  id_hash TEXT PRIMARY KEY,
  owner TEXT,
  data TEXT NOT NULL,
  expires_at INTEGER NOT NULL
);
CREATE INDEX sessions_owner ON sessions(owner);
CREATE INDEX sessions_expiry ON sessions(expires_at);
CREATE TABLE health(id INTEGER PRIMARY KEY CHECK(id = 1), checked_at INTEGER NOT NULL);
ALTER TABLE profiles ADD COLUMN share_schedules INTEGER NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS friendships_requester ON friendships(requester);
CREATE INDEX IF NOT EXISTS friendships_recipient ON friendships(recipient);
";

pub fn migrate(conn: &mut Connection) -> anyhow::Result<()> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version > SCHEMA_VERSION {
        bail!(
            "the user database has schema {version}, newer than this build's {SCHEMA_VERSION}; deploy a newer build"
        );
    }
    if version < 3 {
        conn.execute_batch(V3)?;
    }
    if version < 4 {
        let tx = conn.transaction()?;
        tx.execute_batch(V4)?;
        convert_v3(&tx)?;
        tx.execute_batch("PRAGMA user_version=4")?;
        tx.commit()?;
    }
    Ok(())
}

/// `15-122` or `15122` to `15122`.
pub fn course_id_from_code(code: &str) -> Option<String> {
    let digits: String = code.chars().filter(|c| *c != '-').collect();
    (digits.len() == 5 && digits.bytes().all(|b| b.is_ascii_digit())).then_some(digits)
}

/// Copies v3 schedules and saved courses into the v4 tables, best effort. Old section ids can't be mapped to section
/// names, so entries start without a section. The old rows are left as they were.
fn convert_v3(tx: &Transaction) -> anyhow::Result<()> {
    let now = now_ms();
    let mut old = tx.prepare("SELECT id, owner, payload, share_id FROM schedules")?;
    let rows = old
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (id, owner, payload, share_id) in rows {
        let Ok(payload) = serde_json::from_str::<serde_json::Value>(&payload) else {
            continue;
        };
        let Some(term) = payload["term_id"].as_str() else {
            continue;
        };
        let name = payload["name"]
            .as_str()
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .unwrap_or("Schedule");
        let name: String = name.chars().take(crate::records::NAME_MAX).collect();
        let mut seen = std::collections::HashSet::new();
        let entries: Vec<serde_json::Value> = payload["items"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|item| course_id_from_code(item["course_code"].as_str()?))
            .filter(|id| seen.insert(id.clone()))
            .take(crate::records::schedules::ENTRIES_MAX)
            .enumerate()
            .map(|(i, course_id)| {
                serde_json::json!({
                    "courseId": course_id,
                    "section": null,
                    "child": null,
                    "passFail": false,
                    "color": i % 10,
                    "hidden": false,
                })
            })
            .collect();
        let body = serde_json::json!({ "name": name, "entries": entries });
        tx.execute(
            "INSERT OR IGNORE INTO user_schedules(owner, id, term_id, body, visibility, share_id, revision, created_at, updated_at)
             VALUES (?, ?, ?, ?, 'private', ?, 1, ?, ?)",
            params![owner, id, term, body.to_string(), share_id, now, now],
        )?;
    }
    let mut saved = tx.prepare(
        "SELECT owner, code FROM saved_courses WHERE status = 'saved' ORDER BY owner, code",
    )?;
    let rows = saved
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    for (owner, code) in rows {
        if let Some(course_id) = course_id_from_code(&code) {
            tx.execute(
                "INSERT OR IGNORE INTO user_saved(owner, course_id, saved_at) VALUES (?, ?, ?)",
                params![owner, course_id, now],
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const V3_ROWS: &str = r#"
INSERT INTO schedules VALUES('old-1','alice','{"id":"old-1","name":"Fall plan","term_id":"F26","items":[{"course_code":"15-122","section_ids":["s1"]},{"course_code":"bogus","section_ids":[]},{"course_code":"21-127","section_ids":[]}]}',3,'share-old');
INSERT INTO schedules VALUES('old-2','bob','not json',1,NULL);
INSERT INTO saved_courses VALUES('alice','15-122','saved','');
INSERT INTO saved_courses VALUES('alice','15-213','taken','F25');
INSERT INTO profiles VALUES('alice','alice','Alice',1);
"#;

    #[test]
    fn opens_a_v3_database_and_converts_it_additively() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("users.sqlite");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(V3).unwrap();
            conn.execute_batch(V3_ROWS).unwrap();
        }
        let db = UserDb::open(&path).unwrap();
        let conn = db.conn.lock().unwrap();
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);

        let (term, body, share): (String, String, Option<String>) = conn
            .query_row(
                "SELECT term_id, body, share_id FROM user_schedules WHERE owner='alice' AND id='old-1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(term, "F26");
        assert_eq!(share.as_deref(), Some("share-old"));
        let body: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(body["name"], "Fall plan");
        let entries = body["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["courseId"], "15122");
        assert_eq!(entries[0]["section"], serde_json::Value::Null);
        assert_eq!(entries[1]["courseId"], "21127");

        // Unparseable rows are skipped, not fatal.
        let bobs: i64 = conn
            .query_row(
                "SELECT count(*) FROM user_schedules WHERE owner='bob'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(bobs, 0);

        let saved: Vec<String> = conn
            .prepare("SELECT course_id FROM user_saved WHERE owner='alice'")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(saved, vec!["15122"]);

        // The old rows are untouched, and existing profiles gain the new column.
        let old: i64 = conn
            .query_row("SELECT count(*) FROM schedules", [], |r| r.get(0))
            .unwrap();
        assert_eq!(old, 2);
        let old_saved: i64 = conn
            .query_row("SELECT count(*) FROM saved_courses", [], |r| r.get(0))
            .unwrap();
        assert_eq!(old_saved, 2);
        let (courses, schedules): (bool, bool) = conn
            .query_row(
                "SELECT share_courses, share_schedules FROM profiles WHERE subject='alice'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(courses && !schedules);
        drop(conn);

        // Opening again is a no-op.
        drop(db);
        UserDb::open(&path).unwrap();
    }

    #[test]
    fn new_databases_reach_the_current_version_and_refuse_newer_ones() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("users.sqlite");
        drop(UserDb::open(&path).unwrap());
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA user_version=99").unwrap();
        drop(conn);
        let err = UserDb::open(&path).err().unwrap();
        assert!(format!("{err:#}").contains("newer"));
    }
}
