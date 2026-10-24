//! `courses-index build`: reads the raw inputs, builds every document and the relational tables, and writes
//! `catalog.sqlite`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use courses_catalog::{
    Audience, CourseRow, MeetingRow, OfferingRow, Prepared, SectionRow, TermRow, Writer,
};
use rayon::prelude::*;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use tracing::{info, warn};

use crate::catalog::{self, Catalog};
use crate::config::{self, TermConfig};
use crate::finals;
use crate::model::{TermData, course_code};
use crate::requisite::{self, Requisite};
use crate::sources::Inputs;
use crate::sources::stellic::prefix_dept;
use crate::syllabi;
use crate::terms;

#[derive(Clone, Debug)]
pub struct Options {
    /// The scraper's output root.
    pub exported: PathBuf,
    /// The repository's `data/` directory.
    pub data: PathBuf,
    pub output: PathBuf,
    /// Build only these terms; empty for every term in `data/terms.json`.
    pub terms: Vec<String>,
    pub stellic_user: Option<String>,
    /// Also write each document as JSON under this directory: `public/<name>.json` and `member/<name>.json` (what a
    /// signed-in user gets).
    pub dump: Option<PathBuf>,
    /// The build time, in seconds since 1970 (`SOURCE_DATE_EPOCH`). Without it, the newest input timestamp.
    pub source_date_epoch: Option<i64>,
}

/// One document's versions, serialized.
struct Doc {
    name: String,
    public: Option<Vec<u8>>,
    member: Option<Vec<u8>>,
}

#[derive(Default)]
struct Docs(Vec<Doc>);

impl Docs {
    fn public(&mut self, name: impl Into<String>, value: &impl Serialize) -> Result<()> {
        self.raw(name.into(), Some(serde_json::to_vec(value)?), None);
        Ok(())
    }

    fn member(&mut self, name: impl Into<String>, value: &impl Serialize) -> Result<()> {
        self.raw(name.into(), None, Some(serde_json::to_vec(value)?));
        Ok(())
    }

    /// A public version and, when it differs, a member version.
    fn split(
        &mut self,
        name: impl Into<String>,
        public: &impl Serialize,
        member: &impl Serialize,
    ) -> Result<()> {
        let public = serde_json::to_vec(public)?;
        let member = serde_json::to_vec(member)?;
        let member = (member != public).then_some(member);
        self.raw(name.into(), Some(public), member);
        Ok(())
    }

    fn raw(&mut self, name: String, public: Option<Vec<u8>>, member: Option<Vec<u8>>) {
        self.0.push(Doc {
            name,
            public,
            member,
        });
    }

    fn sort(&mut self) -> Result<()> {
        self.0.sort_by(|a, b| a.name.cmp(&b.name));
        if let Some(w) = self.0.windows(2).find(|w| w[0].name == w[1].name) {
            bail!("document {:?} built twice", w[0].name);
        }
        Ok(())
    }

    fn versions(&self) -> impl Iterator<Item = (&str, Audience, &[u8])> {
        self.0.iter().flat_map(|d| {
            d.public
                .iter()
                .map(move |j| (d.name.as_str(), Audience::Public, j.as_slice()))
                .chain(
                    d.member
                        .iter()
                        .map(move |j| (d.name.as_str(), Audience::Member, j.as_slice())),
                )
        })
    }
}

pub struct Summary {
    pub output: PathBuf,
    pub documents: usize,
    pub json_bytes: u64,
    pub file_bytes: u64,
    pub version: String,
    pub terms: Vec<(String, usize, usize)>,
    pub catalog_courses: usize,
    pub syllabi: usize,
    pub finals: Vec<(String, usize)>,
    pub fce_documents: usize,
    pub program_documents: usize,
}

fn select(configs: Vec<TermConfig>, only: &[String]) -> Result<Vec<TermConfig>> {
    if only.is_empty() {
        return Ok(configs);
    }
    for id in only {
        if !configs.iter().any(|c| &c.id == id) {
            bail!("{id} isn't in data/terms.json");
        }
    }
    Ok(configs
        .into_iter()
        .filter(|c| only.contains(&c.id))
        .collect())
}

pub fn build(opts: &Options) -> Result<Summary> {
    let started = Instant::now();
    let configs = select(
        config::read_terms(&opts.data.join("terms.json"))?,
        &opts.terms,
    )?;
    let inputs = Inputs {
        exported: opts.exported.clone(),
        data: opts.data.clone(),
        stellic_user: opts.stellic_user.clone(),
    };
    if !inputs.exported.exists() {
        bail!("no exported root at {}", inputs.exported.display());
    }

    let (((catalog, built), (scan, exams)), (fce, programs)) = rayon::join(
        || {
            rayon::join(
                || {
                    rayon::join(
                        || catalog::build(&inputs.exported),
                        || {
                            configs
                                .par_iter()
                                .map(|c| {
                                    terms::build_term(c, &inputs)
                                        .with_context(|| format!("building {}", c.id))
                                })
                                .collect::<Result<Vec<_>>>()
                        },
                    )
                },
                || {
                    rayon::join(
                        || syllabi::scan(&inputs.exported.join("syllabi")),
                        || finals::read_dir(&inputs.data.join("finals")),
                    )
                },
            )
        },
        || {
            rayon::join(
                || fce_documents(&inputs.data),
                || {
                    crate::programs::documents(&inputs.exported).with_context(|| {
                        format!(
                            "building program documents from {}",
                            inputs.exported.display()
                        )
                    })
                },
            )
        },
    );
    let (catalog, built, scan, mut exams) = (catalog?, built?, scan?, exams?);
    if !opts.terms.is_empty() {
        exams.retain(|term, _| opts.terms.contains(term));
    }
    let (fce, programs) = (fce?, programs?);
    info!(elapsed = ?started.elapsed(), "inputs read");

    // Timestamps come from the inputs, never the clock, so a rebuild gives the same bytes.
    let latest = built
        .iter()
        .filter_map(|t| t.fetched_at.clone())
        .chain(catalog.fetched_at.clone())
        .max();
    let generated_at = opts
        .source_date_epoch
        .map(crate::js::iso_from_unix)
        .or(latest)
        .unwrap_or_else(|| crate::js::iso_from_unix(0));
    let pass_fail = catalog.pass_fail();
    let terms: Vec<TermData> = built
        .into_iter()
        .map(|mut t| {
            terms::apply_pass_fail(&mut t.data, &pass_fail);
            t.data.generated_at = t.fetched_at.unwrap_or_else(|| generated_at.clone());
            t.data
        })
        .collect();

    let mut docs = Docs::default();
    let index = terms::term_index(&terms);
    let order: Vec<String> = index.iter().map(|t| t.term.id.clone()).collect();
    docs.raw(
        "terms".into(),
        Some(serde_json::to_vec_pretty(&index)?),
        None,
    );
    let term_docs: Vec<(String, Vec<u8>, Vec<u8>)> = terms
        .par_iter()
        .map(|t| -> Result<_> {
            Ok((
                format!("term/{}", t.term.id),
                serde_json::to_vec(&t.redacted())?,
                serde_json::to_vec(t)?,
            ))
        })
        .collect::<Result<_>>()?;
    for (name, public, member) in term_docs {
        let member = (member != public).then_some(member);
        docs.raw(name, Some(public), member);
    }
    docs.public("offerings", &terms::offerings(&terms, &order))?;
    docs.member("teaching", &terms::teaching(&terms, &order))?;

    let catalog_generated_at = catalog
        .fetched_at
        .clone()
        .unwrap_or_else(|| generated_at.clone());
    docs.public("catalog/index", &catalog.index(&catalog_generated_at))?;
    for (prefix, chunk) in catalog.details_chunks() {
        docs.public(format!("catalog/details/{prefix}"), &chunk)?;
    }

    for (prefix, chunk) in syllabi::index(&scan.files) {
        docs.member(format!("syllabi/{prefix}"), &chunk)?;
    }
    for (term, list) in &exams {
        docs.split(format!("finals/{term}"), &finals::redacted(list), list)?;
    }

    // FCEs and degree programs: member-only, built by their own modules (decisions 21 and 22).
    let fce_summary = fce.map(|(list, summary)| {
        for (name, json) in list {
            docs.raw(name, None, Some(json));
        }
        summary
    });
    if fce_summary.is_none() {
        warn!("no data/fces.csv; skipping FCE documents");
    }
    let programs_summary = programs.map(|(list, summary)| {
        for (name, json) in list {
            docs.raw(name, None, Some(json));
        }
        summary
    });
    if programs_summary.is_none() {
        warn!("no Stellic program exports; skipping program documents");
    }

    docs.sort()?;
    let version = {
        let mut h = Sha256::new();
        for (name, audience, json) in docs.versions() {
            h.update(format!(
                "{name}\t{}\t{}\n",
                audience.as_str(),
                courses_catalog::etag(json)
            ));
        }
        format!("{:x}", h.finalize())[..16].to_owned()
    };
    let build_info = json!({
        "version": version,
        "generatedAt": generated_at,
        "schemaVersion": courses_catalog::SCHEMA_VERSION,
        "terms": order,
    });
    docs.public("build", &build_info)?;
    docs.sort()?;

    if let Some(dir) = &opts.dump {
        dump(dir, &docs)?;
    }

    let versions: Vec<(&str, Audience, &[u8])> = docs.versions().collect();
    let prepared = versions
        .par_iter()
        .map(|(_, _, json)| Prepared::new(json))
        .collect::<Result<Vec<_>>>()?;
    let writer = Writer::create(&opts.output)?;
    let mut json_bytes = 0u64;
    for ((name, audience, json), doc) in versions.iter().zip(&prepared) {
        json_bytes += json.len() as u64;
        writer.put_prepared(name, *audience, doc)?;
    }
    write_tables(&writer, &terms, &index, &catalog)?;
    let inputs_summary = json!({
        "terms": terms.iter().map(|t| (t.term.id.clone(), json!({
            "courses": t.courses.len(),
            "sections": t.section_count(),
            "sources": t.term.sources,
            "generatedAt": t.generated_at,
        }))).collect::<BTreeMap<_, _>>(),
        "catalog": { "courses": catalog.entries.len(), "socDetailsFetchedAt": catalog.fetched_at },
        "syllabi": { "entries": scan.files.len(), "skipped": scan.skipped.len() },
        "finals": exams.iter().map(|(t, l)| (t.clone(), l.len())).collect::<BTreeMap<_, _>>(),
        "fce": fce_summary,
        "programs": programs_summary,
    });
    writer.set_meta("generated_at", &generated_at)?;
    writer.set_meta("version", &version)?;
    writer.set_meta("inputs", &inputs_summary.to_string())?;
    let output = writer.finish()?;
    let file_bytes = std::fs::metadata(&output)?.len();
    info!(elapsed = ?started.elapsed(), "catalog written");

    Ok(Summary {
        output,
        documents: docs.0.len(),
        json_bytes,
        file_bytes,
        version,
        terms: terms
            .iter()
            .map(|t| (t.term.id.clone(), t.courses.len(), t.section_count()))
            .collect(),
        catalog_courses: catalog.entries.len(),
        syllabi: scan.files.len(),
        finals: exams.iter().map(|(t, l)| (t.clone(), l.len())).collect(),
        fce_documents: fce_summary.as_ref().map_or(0, |s| s.documents),
        program_documents: programs_summary.as_ref().map_or(0, |s| s.documents),
    })
}

/// The FCE documents from `<data>/fces.csv`, or `None` without the export.
fn fce_documents(data: &Path) -> Result<Option<(Vec<crate::fce::Document>, crate::fce::Summary)>> {
    let path = data.join("fces.csv");
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    crate::fce::documents(&String::from_utf8_lossy(&bytes))
        .with_context(|| format!("building FCE documents from {}", path.display()))
        .map(Some)
}

fn dump(dir: &Path, docs: &Docs) -> Result<()> {
    for d in &docs.0 {
        let write = |audience: &str, json: &[u8]| -> Result<()> {
            let path = dir.join(audience).join(format!("{}.json", d.name));
            std::fs::create_dir_all(path.parent().unwrap())?;
            std::fs::write(&path, json).with_context(|| format!("writing {}", path.display()))
        };
        if let Some(json) = &d.public {
            write("public", json)?;
        }
        if let Some(json) = d.member.as_ref().or(d.public.as_ref()) {
            write("member", json)?;
        }
    }
    Ok(())
}

fn days(days: &[u8]) -> String {
    days.iter().map(|d| d.to_string()).collect()
}

/// The relational tables: public fields only.
fn write_tables(
    writer: &Writer,
    terms: &[TermData],
    index: &[crate::model::TermIndexEntry],
    catalog: &Catalog,
) -> Result<()> {
    for (position, t) in index.iter().enumerate() {
        writer.insert_term(&TermRow {
            id: &t.term.id,
            name: &t.term.name,
            season: &t.term.season,
            year: t.term.year,
            position: position as i64,
        })?;
    }

    // The newest offering of each course, for courses the catalog doesn't know and for departments.
    let by_id: BTreeMap<&str, &TermData> = terms.iter().map(|t| (t.term.id.as_str(), t)).collect();
    let mut newest: BTreeMap<&str, &crate::model::Course> = BTreeMap::new();
    for t in index {
        for c in &by_id[t.term.id.as_str()].courses {
            newest.entry(c.id.as_str()).or_insert(c);
        }
    }
    let mut ids: Vec<&str> = catalog.entries.keys().map(String::as_str).collect();
    ids.extend(newest.keys().copied());
    ids.sort();
    ids.dedup();
    for id in ids {
        let entry = catalog.entries.get(id);
        let offered = newest.get(id);
        let code = course_code(id);
        let title = entry
            .map(|e| e.title.clone())
            .or_else(|| offered.map(|c| c.title.clone()))
            .unwrap_or_else(|| code.clone());
        let department = offered
            .map(|c| c.dept.clone())
            .or_else(|| prefix_dept(&id[..2]).map(str::to_owned));
        let description = catalog
            .details
            .get(&id[..2])
            .and_then(|d| d.get(id))
            .and_then(|d| d.description.clone());
        let prerequisites = entry
            .and_then(|e| e.prereqs.as_ref())
            .map(|r| requisite::format(r, false));
        let corequisites = entry.filter(|e| !e.coreqs.is_empty()).map(|e| {
            let any = Requisite::Any(
                e.coreqs
                    .iter()
                    .map(|c| Requisite::Course {
                        id: c.clone(),
                        min_grade: None,
                    })
                    .collect(),
            );
            requisite::format(&requisite::simplify(any), false)
        });
        writer.insert_course(&CourseRow {
            id,
            code: &code,
            title: &title,
            department: department.as_deref(),
            units: entry
                .and_then(|e| e.units)
                .or_else(|| offered.and_then(|c| c.units)),
            units_max: entry.and_then(|e| e.units_max),
            description: description.as_deref(),
            prerequisites: prerequisites.as_deref(),
            corequisites: corequisites.as_deref(),
        })?;
    }

    for t in terms {
        for c in &t.courses {
            writer.insert_offering(&OfferingRow {
                course_id: &c.id,
                term_id: &t.term.id,
                title: &c.title,
                department: Some(&c.dept),
                units: c.units,
            })?;
            let mut seen = std::collections::HashSet::new();
            for s in &c.sections {
                // A source can list a section name twice; the first wins.
                if !seen.insert(s.name.as_str()) {
                    continue;
                }
                writer.insert_section(&SectionRow {
                    term_id: &t.term.id,
                    course_id: &c.id,
                    name: &s.name,
                    kind: s.kind.as_str(),
                    parent: s.parent.as_deref(),
                    mini: s.mini.map(i64::from),
                })?;
                for (position, m) in s.meetings.iter().enumerate() {
                    writer.insert_meeting(&MeetingRow {
                        term_id: &t.term.id,
                        course_id: &c.id,
                        section: &s.name,
                        position: position as i64,
                        days: &days(&m.days),
                        start: m.start.map(i64::from),
                        end: m.end.map(i64::from),
                    })?;
                }
            }
        }
    }
    Ok(())
}
