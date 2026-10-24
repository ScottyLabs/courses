//! Source adapters: each reads one kind of raw input into a term's courses (Courseweave's `scripts/sources`).
//! Adapters fill `id`, `code`, `title`, `units`, `dept`, `level` and `sections`; the term build recomputes
//! `instructors` and `crossListed`.

pub mod csv;
pub mod soc;
pub mod stellic;

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use serde_json::Value;

use crate::config::SourceConfig;
use crate::model::{Course, TermMeta};

/// Where the raw inputs are.
#[derive(Clone, Debug)]
pub struct Inputs {
    /// The scraper's output root (`exported/`).
    pub exported: PathBuf,
    /// The repository's `data/` directory.
    pub data: PathBuf,
    /// Whose Stellic export the `lyear` values in `data/terms.json` are for, when there are several.
    pub stellic_user: Option<String>,
}

/// One source's courses for a term.
pub struct Loaded {
    pub courses: Vec<Course>,
    /// When the source was fetched, as an ISO timestamp, when it records that.
    pub fetched_at: Option<String>,
}

pub fn load(source: &SourceConfig, term: &TermMeta, inputs: &Inputs) -> Result<Loaded> {
    match source.kind.as_str() {
        "soc" => soc::load(source, term, inputs),
        "csv" => csv::load(source, term, inputs),
        "stellic" => stellic::load(source, term, inputs),
        other => bail!("{}: unknown source type {other:?}", term.id),
    }
}

pub fn read_gzip(path: &Path) -> Result<Vec<u8>> {
    let file = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let mut out = Vec::new();
    GzDecoder::new(&file[..])
        .read_to_end(&mut out)
        .with_context(|| format!("decompressing {}", path.display()))?;
    Ok(out)
}

pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
}

/// `fetchedAt` from a `meta.json`, when there is one.
pub fn fetched_at(meta: &Path) -> Option<String> {
    let value: Value = read_json(meta).ok()?;
    value.get("fetchedAt")?.as_str().map(str::to_owned)
}

/// The Stellic course export (`exported/<andrew id>/courses_history`). Stellic numbers years from the exporting
/// account's first term, so this must be one account's export: the only one, or the one `user` names.
pub fn stellic_export_dir(inputs: &Inputs) -> Result<Option<PathBuf>> {
    if let Some(user) = &inputs.stellic_user {
        return Ok(Some(inputs.exported.join(user).join("courses_history")));
    }
    let users = stellic_users(&inputs.exported);
    if users.len() > 1 {
        bail!(
            "several Stellic exports in {} ({}); pass --stellic-user for the one data/terms.json's lyear values are for",
            inputs.exported.display(),
            users.join(", ")
        );
    }
    Ok(users
        .first()
        .map(|u| inputs.exported.join(u).join("courses_history")))
}

/// Accounts with a Stellic course export, sorted.
pub fn stellic_users(exported: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(exported) else {
        return Vec::new();
    };
    let mut users: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join("courses_history").is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    users.sort();
    users
}
