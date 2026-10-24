//! The `catalog.sqlite` file format, shared by the builder (`courses-index`) and its readers (`courses-web-api`).
//!
//! A catalog holds:
//! - `documents`: the JSON documents the website loads (decision 16), gzipped, one row per name and audience
//! - relational tables of public fields (`terms`, `courses`, `offerings`, `sections`, `meetings`, and the
//!   `course_search` full-text index) for server-side queries (decision 15)
//! - `meta`: key/value pairs, including `schema_version`
//!
//! A document's public version is what signed-out visitors get. A member version, when there is one, is what signed-in
//! CMU users get instead; a document with only a member version is unavailable to signed-out visitors.

use std::io::{Read, Write as _};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use flate2::{Compression, GzBuilder, read::GzDecoder};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::Serialize;
use sha2::{Digest, Sha256};

/// Raised whenever the tables or the documents' formats change incompatibly.
pub const SCHEMA_VERSION: u32 = 2;

/// The catalog's file name inside a catalog directory.
pub const FILE_NAME: &str = "catalog.sqlite";

pub const SCHEMA: &str = r#"
CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL) WITHOUT ROWID;
CREATE TABLE documents(
  name TEXT NOT NULL,
  audience TEXT NOT NULL CHECK(audience IN ('public', 'member')),
  etag TEXT NOT NULL,
  bytes INTEGER NOT NULL,
  gzip BLOB NOT NULL,
  PRIMARY KEY(name, audience)
) WITHOUT ROWID;
CREATE TABLE terms(
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  season TEXT NOT NULL,
  year INTEGER NOT NULL,
  position INTEGER NOT NULL
) WITHOUT ROWID;
CREATE TABLE courses(
  id TEXT PRIMARY KEY,
  code TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  department TEXT,
  units REAL,
  units_max REAL,
  description TEXT,
  prerequisites TEXT,
  corequisites TEXT
) WITHOUT ROWID;
CREATE TABLE offerings(
  course_id TEXT NOT NULL,
  term_id TEXT NOT NULL,
  title TEXT NOT NULL,
  department TEXT,
  units REAL,
  PRIMARY KEY(course_id, term_id)
) WITHOUT ROWID;
CREATE INDEX offerings_term ON offerings(term_id, course_id);
CREATE TABLE sections(
  term_id TEXT NOT NULL,
  course_id TEXT NOT NULL,
  name TEXT NOT NULL,
  kind TEXT NOT NULL,
  parent TEXT,
  mini INTEGER,
  PRIMARY KEY(term_id, course_id, name)
) WITHOUT ROWID;
CREATE TABLE meetings(
  term_id TEXT NOT NULL,
  course_id TEXT NOT NULL,
  section TEXT NOT NULL,
  position INTEGER NOT NULL,
  days TEXT NOT NULL,
  start INTEGER,
  end INTEGER,
  PRIMARY KEY(term_id, course_id, section, position)
) WITHOUT ROWID;
CREATE VIRTUAL TABLE course_search USING fts5(id UNINDEXED, code, title, description);
"#;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Audience {
    Public,
    Member,
}

impl Audience {
    pub fn as_str(self) -> &'static str {
        match self {
            Audience::Public => "public",
            Audience::Member => "member",
        }
    }

    fn parse(s: &str) -> Result<Self> {
        match s {
            "public" => Ok(Audience::Public),
            "member" => Ok(Audience::Member),
            _ => bail!("unknown audience {s:?}"),
        }
    }
}

/// Document names are slash-separated lowercase path segments, such as `term/s27` or `fce/courses/15`.
/// Uppercase is allowed for term ids (`term/S27`).
pub fn valid_document_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 200
        && name.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && segment
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        })
}

/// Gzip without a timestamp or file name, so the same input always gives the same bytes.
pub fn gzip(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), Compression::best());
    encoder.write_all(bytes)?;
    Ok(encoder.finish()?)
}

pub fn gunzip(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    GzDecoder::new(bytes).read_to_end(&mut out)?;
    Ok(out)
}

/// The ETag of a document version: the SHA-256 of its uncompressed JSON, in hex.
pub fn etag(json: &[u8]) -> String {
    format!("{:x}", Sha256::digest(json))
}

pub struct TermRow<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub season: &'a str,
    pub year: i64,
    /// Order in the term list, newest first.
    pub position: i64,
}

pub struct CourseRow<'a> {
    /// Five digits without a dash, such as `15122`.
    pub id: &'a str,
    /// Display code, such as `15-122`.
    pub code: &'a str,
    pub title: &'a str,
    pub department: Option<&'a str>,
    pub units: Option<f64>,
    pub units_max: Option<f64>,
    pub description: Option<&'a str>,
    pub prerequisites: Option<&'a str>,
    pub corequisites: Option<&'a str>,
}

pub struct OfferingRow<'a> {
    pub course_id: &'a str,
    pub term_id: &'a str,
    pub title: &'a str,
    pub department: Option<&'a str>,
    pub units: Option<f64>,
}

pub struct SectionRow<'a> {
    pub term_id: &'a str,
    pub course_id: &'a str,
    /// Section name, unique within a course and term, such as `A`, `1` or `A3`.
    pub name: &'a str,
    /// `lecture` or `section`, as in the term document.
    pub kind: &'a str,
    pub parent: Option<&'a str>,
    pub mini: Option<i64>,
}

pub struct MeetingRow<'a> {
    pub term_id: &'a str,
    pub course_id: &'a str,
    pub section: &'a str,
    /// The meeting's index within its section.
    pub position: i64,
    /// Weekday digits, 0 for Monday through 6 for Sunday, such as `024`; empty when TBA.
    pub days: &'a str,
    /// Minutes after midnight, or `None` when TBA.
    pub start: Option<i64>,
    pub end: Option<i64>,
}

/// Writes a new catalog. The file appears at its destination only when [`Writer::finish`] succeeds.
pub struct Writer {
    conn: Connection,
    staging: PathBuf,
    dest: PathBuf,
}

impl Writer {
    pub fn create(dest: &Path) -> Result<Self> {
        if let Some(parent) = dest.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        let staging = dest.with_extension("sqlite.partial");
        if staging.exists() {
            std::fs::remove_file(&staging)?;
        }
        let conn = Connection::open(&staging)
            .with_context(|| format!("creating {}", staging.display()))?;
        conn.execute_batch("PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF;")?;
        conn.execute_batch(SCHEMA)?;
        conn.execute_batch("BEGIN")?;
        Ok(Self {
            conn,
            staging,
            dest: dest.to_owned(),
        })
    }

    /// Stores one version of a document from its JSON bytes.
    pub fn put_raw(&self, name: &str, audience: Audience, json: &[u8]) -> Result<()> {
        if !valid_document_name(name) {
            bail!("invalid document name {name:?}");
        }
        let changed = self.conn.execute(
            "INSERT INTO documents(name, audience, etag, bytes, gzip) VALUES (?, ?, ?, ?, ?)",
            params![
                name,
                audience.as_str(),
                etag(json),
                json.len() as i64,
                gzip(json)?
            ],
        );
        match changed {
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                bail!("document {name:?} ({}) written twice", audience.as_str())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn put_json<T: Serialize + ?Sized>(
        &self,
        name: &str,
        audience: Audience,
        value: &T,
    ) -> Result<()> {
        self.put_raw(name, audience, &serde_json::to_vec(value)?)
    }

    /// Stores a document that everyone may read, with the same content for members.
    pub fn put_public<T: Serialize + ?Sized>(&self, name: &str, value: &T) -> Result<()> {
        self.put_json(name, Audience::Public, value)
    }

    /// Stores a document only signed-in CMU users may read.
    pub fn put_member<T: Serialize + ?Sized>(&self, name: &str, value: &T) -> Result<()> {
        self.put_json(name, Audience::Member, value)
    }

    /// Stores a document whose public version leaves fields out. The member version is stored only when it differs.
    pub fn put_split<P: Serialize + ?Sized, M: Serialize + ?Sized>(
        &self,
        name: &str,
        public: &P,
        member: &M,
    ) -> Result<()> {
        let public = serde_json::to_vec(public)?;
        let member = serde_json::to_vec(member)?;
        self.put_raw(name, Audience::Public, &public)?;
        if member != public {
            self.put_raw(name, Audience::Member, &member)?;
        }
        Ok(())
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO meta(key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn insert_term(&self, row: &TermRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO terms(id, name, season, year, position) VALUES (?, ?, ?, ?, ?)",
            params![row.id, row.name, row.season, row.year, row.position],
        )?;
        Ok(())
    }

    pub fn insert_course(&self, row: &CourseRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO courses(id, code, title, department, units, units_max, description, prerequisites, corequisites)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                row.id,
                row.code,
                row.title,
                row.department,
                row.units,
                row.units_max,
                row.description,
                row.prerequisites,
                row.corequisites
            ],
        )?;
        Ok(())
    }

    pub fn insert_offering(&self, row: &OfferingRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO offerings(course_id, term_id, title, department, units) VALUES (?, ?, ?, ?, ?)",
            params![row.course_id, row.term_id, row.title, row.department, row.units],
        )?;
        Ok(())
    }

    pub fn insert_section(&self, row: &SectionRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sections(term_id, course_id, name, kind, parent, mini) VALUES (?, ?, ?, ?, ?, ?)",
            params![row.term_id, row.course_id, row.name, row.kind, row.parent, row.mini],
        )?;
        Ok(())
    }

    pub fn insert_meeting(&self, row: &MeetingRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO meetings(term_id, course_id, section, position, days, start, end) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                row.term_id,
                row.course_id,
                row.section,
                row.position,
                row.days,
                row.start,
                row.end
            ],
        )?;
        Ok(())
    }

    /// Indexes the courses for search, compacts the file and moves it into place.
    pub fn finish(self) -> Result<PathBuf> {
        self.set_meta("schema_version", &SCHEMA_VERSION.to_string())?;
        self.conn.execute_batch(
            "INSERT INTO course_search(id, code, title, description)
               SELECT id, code, title, coalesce(description, '') FROM courses ORDER BY id;
             COMMIT;
             VACUUM;",
        )?;
        drop(self.conn);
        std::fs::rename(&self.staging, &self.dest)
            .with_context(|| format!("moving the catalog to {}", self.dest.display()))?;
        Ok(self.dest)
    }
}

/// One version of a document, as stored.
pub struct StoredDocument {
    pub audience: Audience,
    pub etag: String,
    /// Uncompressed size in bytes.
    pub bytes: u64,
    pub gzip: Vec<u8>,
}

pub enum Lookup {
    Found(StoredDocument),
    /// The document exists, but only for signed-in CMU users.
    MemberOnly,
    Missing,
}

/// Reads a catalog. Opened read-only; not `Sync`, so open one per thread or keep a pool.
pub struct Reader {
    conn: Connection,
}

impl Reader {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .with_context(|| format!("opening {}", path.display()))?;
        let reader = Self { conn };
        let version = reader.schema_version()?;
        if version != SCHEMA_VERSION {
            bail!(
                "{} has catalog schema {version}, but this build reads schema {SCHEMA_VERSION}",
                path.display()
            );
        }
        Ok(reader)
    }

    pub fn schema_version(&self) -> Result<u32> {
        let value = self
            .meta("schema_version")?
            .context("catalog has no schema_version")?;
        Ok(value.parse()?)
    }

    pub fn meta(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT value FROM meta WHERE key = ?", [key], |r| r.get(0))
            .optional()?)
    }

    /// The version of a document the caller may read: the member version for members when there is one, otherwise
    /// the public version.
    pub fn document(&self, name: &str, member: bool) -> Result<Lookup> {
        if !valid_document_name(name) {
            return Ok(Lookup::Missing);
        }
        let mut stmt = self.conn.prepare_cached(
            "SELECT audience, etag, bytes, gzip FROM documents WHERE name = ? ORDER BY audience",
        )?;
        let versions = stmt
            .query_map([name], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, Vec<u8>>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut public = None;
        let mut member_version = None;
        for (audience, etag, bytes, gzip) in versions {
            let doc = StoredDocument {
                audience: Audience::parse(&audience)?,
                etag,
                bytes: bytes as u64,
                gzip,
            };
            match doc.audience {
                Audience::Public => public = Some(doc),
                Audience::Member => member_version = Some(doc),
            }
        }
        Ok(match (member, member_version, public) {
            (true, Some(doc), _) => Lookup::Found(doc),
            (_, _, Some(doc)) => Lookup::Found(doc),
            (false, Some(_), None) => Lookup::MemberOnly,
            _ => Lookup::Missing,
        })
    }

    /// Every document name, with the audiences it has.
    pub fn document_names(&self) -> Result<Vec<(String, Audience)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, audience FROM documents ORDER BY name, audience")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        rows.map(|r| {
            let (name, audience) = r?;
            Ok((name, Audience::parse(&audience)?))
        })
        .collect()
    }

    /// The connection, for queries over the relational tables.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(dir: &Path) -> PathBuf {
        let writer = Writer::create(&dir.join(FILE_NAME)).unwrap();
        writer
            .put_public("terms", &[serde_json::json!({"id": "S27"})])
            .unwrap();
        writer
            .put_split(
                "term/S27",
                &serde_json::json!({"instructors": []}),
                &serde_json::json!({"instructors": ["Kosbie"]}),
            )
            .unwrap();
        writer
            .put_split("same", &serde_json::json!(1), &serde_json::json!(1))
            .unwrap();
        writer
            .put_member("fce/summary", &serde_json::json!({}))
            .unwrap();
        writer
            .insert_course(&CourseRow {
                id: "15122",
                code: "15-122",
                title: "Principles of Imperative Computation",
                department: Some("CS"),
                units: Some(12.0),
                units_max: None,
                description: Some("Pointers and invariants"),
                prerequisites: Some("15-112"),
                corequisites: None,
            })
            .unwrap();
        writer.finish().unwrap()
    }

    fn json(doc: &StoredDocument) -> serde_json::Value {
        serde_json::from_slice(&gunzip(&doc.gzip).unwrap()).unwrap()
    }

    #[test]
    fn audiences_pick_the_right_version() {
        let dir = tempfile::tempdir().unwrap();
        let reader = Reader::open(&build(dir.path())).unwrap();

        let Lookup::Found(doc) = reader.document("term/S27", false).unwrap() else {
            panic!("public term missing")
        };
        assert_eq!(json(&doc), serde_json::json!({"instructors": []}));
        let Lookup::Found(doc) = reader.document("term/S27", true).unwrap() else {
            panic!("member term missing")
        };
        assert_eq!(json(&doc), serde_json::json!({"instructors": ["Kosbie"]}));
        assert_eq!(doc.etag, etag(br#"{"instructors":["Kosbie"]}"#));

        assert!(
            matches!(reader.document("terms", true).unwrap(), Lookup::Found(d) if d.audience == Audience::Public)
        );
        assert!(matches!(
            reader.document("fce/summary", false).unwrap(),
            Lookup::MemberOnly
        ));
        assert!(matches!(
            reader.document("fce/summary", true).unwrap(),
            Lookup::Found(_)
        ));
        assert!(matches!(
            reader.document("nope", true).unwrap(),
            Lookup::Missing
        ));
        assert!(matches!(
            reader.document("../etc", true).unwrap(),
            Lookup::Missing
        ));

        // An identical member version isn't stored twice.
        let names = reader.document_names().unwrap();
        assert_eq!(names.iter().filter(|(n, _)| n == "same").count(), 1);
    }

    #[test]
    fn search_index_covers_courses() {
        let dir = tempfile::tempdir().unwrap();
        let reader = Reader::open(&build(dir.path())).unwrap();
        let id: String = reader
            .connection()
            .query_row(
                "SELECT id FROM course_search WHERE course_search MATCH 'invariants'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(id, "15122");
    }

    #[test]
    fn builds_are_byte_identical() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        assert_eq!(
            std::fs::read(build(a.path())).unwrap(),
            std::fs::read(build(b.path())).unwrap()
        );
    }

    #[test]
    fn rejects_bad_names_and_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        let writer = Writer::create(&dir.path().join(FILE_NAME)).unwrap();
        assert!(writer.put_public("a/../b", &1).is_err());
        assert!(writer.put_public("", &1).is_err());
        writer.put_public("x", &1).unwrap();
        assert!(writer.put_public("x", &2).is_err());
    }
}
