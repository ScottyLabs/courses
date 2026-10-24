//! Degree program documents, ported from Courseweave's `scripts/build-programs.ts` and
//! `scripts/lib/concentrations.ts`.
//!
//! # Entry points
//!
//! - [`write`]`(writer, exported_root)` reads the Stellic program exports under
//!   `<exported_root>/<andrew id>/programs/<catalogId>/<auditId>.json` (the scraper's `--mode programs` output) and
//!   stores the documents below in the catalog, member-only. It's what `courses-index build` calls. Without exports
//!   it writes nothing and logs a warning, as Courseweave does.
//! - [`documents`]`(exported_root)` returns the same documents as `(name, JSON bytes)` pairs without a catalog.
//! - [`read_exports`] and [`build`] are the two halves of [`documents`]; [`build`] takes parsed exports, so tests can
//!   use synthetic ones, and returns typed documents.
//!
//! # Documents
//!
//! Both are member-only (decision 16) and match Courseweave's `public/data/programs/` files:
//!
//! - `programs/index`: `ProgramsIndex` (`plan-types.ts`), `{ programs: ProgramInfo[] }`, every program with its
//!   versions, sorted by name. Each version is `{ auditId, name, entryYear, campus }`, newest entry year first.
//! - `programs/<auditId>`: `ProgramVersion` (`degree-types.ts`), one audit version's normalized requirement tree
//!   ([`normalize::ReqNode`]) with its total units, sharing limits, attached gen-ed bundles and concentrations.
//!
//! Courseweave's `generatedAt` is left out of `programs/index`, so the same exports always give the same bytes
//! (decision 22).
//!
//! # Versions, bundles and concentrations
//!
//! When several exports have the same audit version, the file modified last wins. Versions whose program or audit
//! name starts with "DELETE" are dropped. A version's name, entry year and campus come from Stellic's audit and
//! program names ("EY2025 Pittsburgh - BS in ..."). A major's sharing limits that name a "requirements" program (or,
//! for programs not exported, one named like a gen-ed or degree check) attach it, at the version that fits the
//! major's entry year and campus. Each concentration is matched to the three requirement bundles that best fit its
//! name, see [`concentrations`].

pub mod concentrations;
pub mod normalize;
pub mod raw;

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::SystemTime;

use anyhow::{Context, Result};
use courses_catalog::{Audience, Writer};
use rayon::prelude::*;
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use tracing::{info, warn};

use crate::fce::Document;
use crate::fce::js;
use concentrations::{Candidate, Parent};
use normalize::{Ctx, DoubleCountRule, ReqNode};
use raw::{Js, Raw};

const CAMPUSES: [&str; 12] = [
    "Pittsburgh",
    "Doha",
    "Qatar",
    "Africa",
    "Rwanda",
    "Kigali",
    "Silicon Valley",
    "Lisbon",
    "Adelaide",
    "Los Angeles",
    "New York",
    "Washington",
];

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramVersionInfo {
    pub audit_id: i64,
    /// Such as "EY2025 Pittsburgh".
    pub name: String,
    /// Entry year the version applies to, from "EY2025"; `None` when unknown.
    pub entry_year: Option<i64>,
    pub campus: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramInfo {
    pub catalog_id: i64,
    /// Such as "BS in Computer Science".
    pub name: String,
    pub kind: &'static str,
    /// Newest entry year first.
    pub versions: Vec<ProgramVersionInfo>,
}

/// `programs/index`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProgramsIndex {
    pub programs: Vec<ProgramInfo>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedProgram {
    pub catalog_id: Raw,
    pub name: Raw,
    /// The version chosen for the major's entry year and campus; `None` when it wasn't exported.
    pub audit_id: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConcentrationProgram {
    pub catalog_id: i64,
    pub audit_id: i64,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ConcentrationInfo {
    /// Stellic's concentration id.
    pub id: Raw,
    pub name: Raw,
    /// Requirement bundles that likely implement it, best match first.
    pub programs: Vec<ConcentrationProgram>,
}

/// `programs/<auditId>`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramVersion {
    pub audit_id: i64,
    pub catalog_id: i64,
    pub program_name: String,
    pub name: String,
    pub kind: &'static str,
    pub entry_year: Option<i64>,
    pub campus: Option<&'static str>,
    /// The degree's overall unit minimum (the root's "at least N units").
    pub total_units: Option<js::Num>,
    /// College gen-ed and degree-check bundles audited alongside this program.
    pub attached: Vec<AttachedProgram>,
    pub double_count: Vec<DoubleCountRule>,
    pub concentrations: Vec<ConcentrationInfo>,
    pub root: ReqNode,
}

/// The typed program documents.
#[derive(Debug)]
pub struct Programs {
    pub index: ProgramsIndex,
    /// By audit id.
    pub versions: BTreeMap<i64, ProgramVersion>,
    /// Course ranges that couldn't be parsed.
    pub unknown_patterns: usize,
}

static ENTRY_YEAR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!("(?i:EY){}?([0-9]{{4}})", js::SPACE_CLASS)).unwrap());
static PAREN_YEAR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\(([0-9]{4})\)").unwrap());
static NAME_DASH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!("{0}+-{0}+", js::SPACE_CLASS)).unwrap());

fn entry_year_of(names: &[&str]) -> Option<i64> {
    names.iter().find_map(|n| {
        let caps = ENTRY_YEAR.captures(n).or_else(|| PAREN_YEAR.captures(n))?;
        caps[1].parse().ok()
    })
}

fn campus_of(names: &[&str]) -> Option<&'static str> {
    names.iter().find_map(|n| {
        CAMPUSES
            .iter()
            .find(|c| n.contains(*c))
            .map(|&c| if c == "Qatar" { "Doha" } else { c })
    })
}

fn version_name(audit_name: &str, program_name: &str) -> String {
    let a = js::trim(audit_name);
    if !a.is_empty() && a.to_lowercase() != "published" {
        // "EY2025 Pittsburgh - BS in Computer Science" to "EY2025 Pittsburgh".
        let first = NAME_DASH.find(a).map_or(a, |m| &a[..m.start()]);
        let short = js::trim(first);
        return if short.is_empty() { a } else { short }.to_owned();
    }
    match entry_year_of(&[program_name]) {
        Some(year) => format!("EY{year}"),
        None => "Current".to_owned(),
    }
}

/// `/^\s*DELETE\b/i`.
fn starts_with_delete(name: &str) -> bool {
    let rest = name.trim_start_matches(js::is_space);
    rest.get(..6)
        .is_some_and(|p| p.eq_ignore_ascii_case("delete"))
        && !rest[6..]
            .bytes()
            .next()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn is_deleted(raw: Js) -> bool {
    starts_with_delete(&raw.get("program_name").string())
        || starts_with_delete(&raw.get("audit_name").string())
}

fn sorted_entries(dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("listing {}", dir.display()))?
        .map(|e| e.map(|e| (e.file_name(), e.path())))
        .collect::<Result<_, _>>()?;
    // Node lists directories sorted by name, byte by byte.
    entries.sort();
    Ok(entries
        .into_iter()
        .map(|(name, path)| (name.to_string_lossy().into_owned(), path))
        .collect())
}

/// Every export's parsed JSON, in the order Courseweave reads them, without deleted versions. Empty when there's no
/// export.
pub fn read_exports(exported_root: &Path) -> Result<Vec<Value>> {
    if !exported_root.is_dir() {
        return Ok(Vec::new());
    }
    // The newest file wins when several exports have the same audit version; it keeps the first one's position.
    let mut files: Vec<(PathBuf, SystemTime)> = Vec::new();
    let mut positions: HashMap<u64, usize> = HashMap::new();
    for (_, user) in sorted_entries(exported_root)? {
        let dir = user.join("programs");
        if !dir.is_dir() {
            continue;
        }
        for (_, catalog) in sorted_entries(&dir)? {
            if !catalog.is_dir() {
                continue;
            }
            for (name, path) in sorted_entries(&catalog)? {
                let Some(stem) = name.strip_suffix(".json") else {
                    continue;
                };
                let id = js::string_to_number(stem);
                let key = if id.is_nan() {
                    f64::NAN.to_bits()
                } else {
                    (id + 0.0).to_bits()
                };
                let mtime = std::fs::metadata(&path)
                    .and_then(|m| m.modified())
                    .with_context(|| format!("reading {}", path.display()))?;
                match positions.get(&key) {
                    Some(&i) if files[i].1 < mtime => files[i] = (path, mtime),
                    Some(_) => {}
                    None => {
                        positions.insert(key, files.len());
                        files.push((path, mtime));
                    }
                }
            }
        }
    }
    let parsed: Vec<Value> = files
        .par_iter()
        .map(|(path, _)| {
            let bytes =
                std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
            serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
        })
        .collect::<Result<_>>()?;
    Ok(parsed
        .into_iter()
        .filter(|raw| !is_deleted(Js::new(raw)))
        .collect())
}

/// A "requirements" program that may implement a concentration.
struct Bundle<'a> {
    info: &'a ProgramInfo,
    department: Option<&'a str>,
}

impl Candidate for Bundle<'_> {
    fn name(&self) -> &str {
        &self.info.name
    }

    fn campuses(&self) -> Vec<Option<&str>> {
        self.info.versions.iter().map(|v| v.campus).collect()
    }

    fn department(&self) -> Option<&str> {
        self.department
    }
}

/// A department as the scoring reads it: `None` for null, the text otherwise.
fn department_of(raw: Js) -> Option<String> {
    let dep = raw.get("program_reqs").get("department");
    (!dep.is_nullish()).then(|| dep.string())
}

struct Index {
    infos: Vec<ProgramInfo>,
    positions: HashMap<i64, usize>,
}

impl Index {
    fn get(&self, catalog_id: i64) -> Option<&ProgramInfo> {
        self.positions.get(&catalog_id).map(|&i| &self.infos[i])
    }

    /// The version for an entry year and campus: the newest that started by `entry_year`, preferring the campus.
    fn pick_version(
        &self,
        catalog_id: i64,
        entry_year: Option<i64>,
        campus: Option<&str>,
    ) -> Option<&ProgramVersionInfo> {
        let versions = &self.get(catalog_id)?.versions;
        let same_campus: Vec<&ProgramVersionInfo> = versions
            .iter()
            .filter(|v| campus.is_none() || v.campus.is_none() || v.campus == campus)
            .collect();
        let pool: Vec<&ProgramVersionInfo> = if same_campus.is_empty() {
            versions.iter().collect()
        } else {
            same_campus
        };
        if let Some(year) = entry_year
            && let Some(fit) = pool
                .iter()
                .find(|v| v.entry_year.is_some_and(|y| y <= year))
        {
            return Some(fit);
        }
        pool.first().copied()
    }
}

fn id_of(raw: Js, field: &str) -> Result<i64> {
    raw.get(field)
        .id()
        .with_context(|| format!("a program export's {field} isn't an integer"))
}

/// `/general education|degree check|core requirements/i`.
fn looks_like_bundle(name: &str) -> bool {
    ["general education", "degree check", "core requirements"]
        .iter()
        .any(|n| concentrations::contains_ascii_ci(name, n))
}

/// Builds the documents from parsed exports, in the order [`read_exports`] returns them.
pub fn build(raws: &[Value]) -> Result<Programs> {
    // The index first: versions need it to resolve attached programs and concentrations.
    let mut index = Index {
        infos: Vec::new(),
        positions: HashMap::new(),
    };
    let mut departments: HashMap<i64, Option<String>> = HashMap::new();
    for raw in raws {
        let raw = Js::new(raw);
        let catalog_id = id_of(raw, "catalog_id")?;
        let program_name = raw.get("program_name").str();
        let audit_name = raw.get("audit_name").str();
        let i = *index.positions.entry(catalog_id).or_insert_with(|| {
            index.infos.push(ProgramInfo {
                catalog_id,
                name: js::trim(program_name).to_owned(),
                kind: normalize::kind_of(&raw.get("program_type").string())
                    .unwrap_or("requirements"),
                versions: Vec::new(),
            });
            index.infos.len() - 1
        });
        index.infos[i].versions.push(ProgramVersionInfo {
            audit_id: id_of(raw, "audit_id")?,
            name: version_name(audit_name, program_name),
            entry_year: entry_year_of(&[audit_name, program_name]),
            campus: campus_of(&[audit_name, program_name]),
        });
        // The first non-empty department, as `if (!departments.get(id))` keeps it.
        let current = departments.get(&catalog_id);
        if current.is_none_or(|d| d.as_deref().is_none_or(str::is_empty)) {
            departments.insert(catalog_id, department_of(raw));
        }
    }
    for info in &mut index.infos {
        info.versions.sort_by(|a, b| {
            b.entry_year
                .unwrap_or(0)
                .cmp(&a.entry_year.unwrap_or(0))
                .then((b.campus == Some("Pittsburgh")).cmp(&(a.campus == Some("Pittsburgh"))))
                .then(b.audit_id.cmp(&a.audit_id))
        });
    }

    let bundles: Vec<Bundle> = index
        .infos
        .iter()
        .filter(|i| i.kind == "requirements")
        .map(|info| Bundle {
            info,
            department: departments.get(&info.catalog_id).and_then(Option::as_deref),
        })
        .collect();

    let program_name = |id: Js| match id.id().and_then(|id| index.get(id)) {
        Some(info) => info.name.clone(),
        None => format!("Program {}", id.string()),
    };
    let built: Vec<(ProgramVersion, usize)> = raws
        .par_iter()
        .map(|raw| {
            let raw = Js::new(raw);
            let mut ctx = Ctx {
                double_count: Vec::new(),
                program_name: &program_name,
                unknown_patterns: 0,
            };
            let mut root = normalize::normalize(raw.get("tree"), &mut ctx, true);
            let (catalog_id, audit_id) = (id_of(raw, "catalog_id")?, id_of(raw, "audit_id")?);
            let info = index.get(catalog_id).context("indexed above")?;
            let vinfo = info
                .versions
                .iter()
                .find(|v| v.audit_id == audit_id)
                .context("indexed above")?;

            // The root's "at least N units" is the degree total; keep it off the node so it isn't reported twice.
            let total_units = root.min_units;
            if matches!(info.kind, "major" | "additional-major") {
                root.min_units = None;
            }

            let mut attached: Vec<AttachedProgram> = Vec::new();
            for rule in &ctx.double_count {
                for (i, id) in rule.programs.iter().enumerate() {
                    let target = id.js().id().and_then(|id| index.get(id));
                    let name = rule.program_names.get(i).map(Raw::js).unwrap_or(Js(None));
                    let is_bundle = match target {
                        Some(t) => t.kind == "requirements",
                        None => looks_like_bundle(&if name.is_nullish() {
                            String::new()
                        } else {
                            name.string()
                        }),
                    };
                    if !is_bundle
                        || attached
                            .iter()
                            .any(|a| a.catalog_id.js().strict_eq(id.js()))
                    {
                        continue;
                    }
                    let version = target.and_then(|t| {
                        index.pick_version(t.catalog_id, vinfo.entry_year, vinfo.campus)
                    });
                    attached.push(AttachedProgram {
                        catalog_id: id.clone(),
                        name: match target {
                            Some(t) => Raw(Value::String(t.name.clone())),
                            None if !name.is_nullish() => name.to_raw(),
                            None => Raw(Value::String(format!("Program {}", id.js().string()))),
                        },
                        audit_id: version.map(|v| v.audit_id),
                    });
                }
            }

            let parent_department = department_of(raw);
            let parent = Parent {
                name: raw.get("program_name").str(),
                campus: vinfo.campus,
                department: parent_department.as_deref(),
            };
            let concentrations = raw
                .get("program_reqs")
                .get("concentrations")
                .items()
                .map(|c| {
                    let ranked = concentrations::rank_concentration_programs(
                        &c.get("name").string(),
                        &parent,
                        &bundles,
                    );
                    ConcentrationInfo {
                        id: c.get("id").to_raw(),
                        name: c.get("name").to_raw(),
                        programs: ranked
                            .into_iter()
                            .take(3)
                            .filter_map(|b| {
                                let v = index.pick_version(
                                    b.info.catalog_id,
                                    vinfo.entry_year,
                                    vinfo.campus,
                                )?;
                                Some(ConcentrationProgram {
                                    catalog_id: b.info.catalog_id,
                                    audit_id: v.audit_id,
                                    name: b.info.name.clone(),
                                })
                            })
                            .collect(),
                    }
                })
                .collect();

            let version = ProgramVersion {
                audit_id,
                catalog_id,
                program_name: info.name.clone(),
                name: vinfo.name.clone(),
                kind: info.kind,
                entry_year: vinfo.entry_year,
                campus: vinfo.campus,
                total_units,
                attached,
                double_count: ctx.double_count,
                concentrations,
                root,
            };
            Ok((version, ctx.unknown_patterns))
        })
        .collect::<Result<_>>()?;

    let mut versions = BTreeMap::new();
    let mut unknown_patterns = 0;
    for (version, unknown) in built {
        unknown_patterns += unknown;
        // A later export of the same audit id replaces an earlier one, as rewriting its file does.
        versions.insert(version.audit_id, version);
    }
    let mut programs = index.infos;
    programs.sort_by(|a, b| js::locale_compare(&a.name, &b.name));
    Ok(Programs {
        index: ProgramsIndex { programs },
        versions,
        unknown_patterns,
    })
}

impl Programs {
    /// The documents as `(name, JSON bytes)` pairs: the index, then each version by audit id.
    pub fn documents(&self) -> Result<Vec<Document>> {
        let mut docs = vec![(
            "programs/index".to_owned(),
            serde_json::to_vec(&self.index)?,
        )];
        let versions: Vec<Document> = self
            .versions
            .par_iter()
            .map(|(id, v)| Ok((format!("programs/{id}"), serde_json::to_vec(v)?)))
            .collect::<Result<_>>()?;
        docs.extend(versions);
        Ok(docs)
    }
}

/// What a build wrote, for the builder's log and the catalog's `meta`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub programs: usize,
    /// Audit versions, one `programs/<auditId>` document each.
    pub versions: usize,
    /// Programs per kind.
    pub kinds: BTreeMap<String, usize>,
    pub documents: usize,
    /// Uncompressed JSON bytes over all documents.
    pub bytes: usize,
    /// Course ranges that couldn't be parsed.
    pub unknown_patterns: usize,
}

/// Builds every program document from the exports under `exported_root`. `None` when there are none.
pub fn documents(exported_root: &Path) -> Result<Option<(Vec<Document>, Summary)>> {
    let raws = read_exports(exported_root)?;
    if raws.is_empty() {
        return Ok(None);
    }
    let programs = build(&raws)?;
    let docs = programs.documents()?;
    let mut kinds = BTreeMap::new();
    for p in &programs.index.programs {
        *kinds.entry(p.kind.to_owned()).or_default() += 1;
    }
    let summary = Summary {
        programs: programs.index.programs.len(),
        versions: programs.versions.len(),
        kinds,
        documents: docs.len(),
        bytes: docs.iter().map(|(_, json)| json.len()).sum(),
        unknown_patterns: programs.unknown_patterns,
    };
    Ok(Some((docs, summary)))
}

/// Reads the Stellic program exports under `exported_root` and stores the program documents in the catalog,
/// member-only.
pub fn write(writer: &Writer, exported_root: &Path) -> Result<Summary> {
    let Some((docs, summary)) = documents(exported_root).with_context(|| {
        format!(
            "building program documents from {}",
            exported_root.display()
        )
    })?
    else {
        warn!(root = %exported_root.display(), "no Stellic program exports; skipping program documents");
        return Ok(Summary::default());
    };
    for (name, json) in &docs {
        writer.put_raw(name, Audience::Member, json)?;
    }
    info!(
        programs = summary.programs,
        versions = summary.versions,
        documents = summary.documents,
        bytes = summary.bytes,
        unknown_patterns = summary.unknown_patterns,
        "wrote program documents"
    );
    Ok(summary)
}

#[cfg(test)]
mod tests;
