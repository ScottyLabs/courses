//! FCE (Faculty Course Evaluation) documents, ported from Courseweave's `scripts/build-fce.ts`.
//!
//! # Entry points
//!
//! - [`write`]`(writer, data_dir)` reads `<data_dir>/fces.csv` (the SmartEvals export) and stores the documents
//!   below in the catalog, member-only. It's what `courses-index build` calls.
//! - [`documents`]`(csv_text)` returns the same documents as `(name, JSON bytes)` pairs without a catalog, for tests.
//! - [`parse_rows`] and [`build`] are the two halves of [`documents`], and [`build`] returns typed documents.
//!
//! # Documents
//!
//! All are member-only (decision 16) and match Courseweave's `public/data/fce/` files:
//!
//! - `fce/summary`: `{ latest, recentTerms, courses: { [courseId]: SummaryTuple } }`, recent per-course averages for
//!   lists, sorting and the hours-versus-units chart. See [`SummaryTuple`].
//! - `fce/courses/<NN>`: `{ [courseId]: RowTuple[] }`, every FCE row of the courses whose number starts with `NN`,
//!   newest first. See [`RowTuple`].
//! - `fce/courses/index`: the `NN` prefixes that have a `fce/courses/<NN>` document, sorted. Courseweave has no such
//!   file; it lets the website skip prefixes without FCEs.
//! - `fce/instructors`: `{ instructors: { [nameKey]: { name, courses, terms } } }`, each instructor's history, keyed
//!   by [`names::fce_name_keys`]. See [`InstructorOut`].
//!
//! Courseweave's `generatedAt` is left out of `fce/summary` and `fce/instructors`, so the same CSV always gives the
//! same bytes (decision 22); nothing reads it.
//!
//! # Aggregates
//!
//! "Recent" is a course's last [`RECENT_TERMS`] terms with at least one response. Averages are weighted by
//! responses; rows without responses stay in the history but are never averaged. Co-taught sections have one row
//! per instructor, so course-level values (hours, course rating) are first averaged per section and sections are
//! then weighted by their largest response count. Minis are detected from section names (fall A1/B2, spring A3/B4,
//! summer A5/B6). Rows from every college are kept, including the few "Teaching Assistants" rows, as Courseweave
//! keeps them.

pub mod csv;
pub mod js;
pub mod names;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use anyhow::{Context, Result, bail};
use courses_catalog::{Audience, Writer};
use serde::Serialize;
use tracing::{info, warn};

use js::{Num, opt};
use names::{Half, Season};

/// Terms in a course's "recent" window.
pub const RECENT_TERMS: usize = 4;

/// A catalog document: its name, such as `fce/summary`, and its JSON.
pub type Document = (String, Vec<u8>);

/// The FCE export's file name inside the data directory.
pub const CSV_FILE: &str = "fces.csv";

/// The seven teaching questions, as the export's column headers name them, in the order rows list them.
pub const QUESTIONS: [&str; 7] = [
    "Interest in student learning",
    "Clearly explain course requirements",
    "Clear learning objectives & goals",
    "Instructor provides feedback to students to improve",
    "Demonstrate importance of subject matter",
    "Explains subject matter of course",
    "Show respect for all students",
];

/// One row of the export: one instructor teaching one section in one term.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    /// Five-digit course id, such as `15122`.
    pub course: String,
    /// Term code, such as `S26`.
    pub term: String,
    /// [`names::term_order`] of `term`.
    pub order: f64,
    pub section: String,
    /// Mini number from the section name, or `None` for a full-semester section.
    pub mini: Option<u8>,
    /// As in the export ("LAST, First"), with whitespace collapsed.
    pub instructor: String,
    pub enrolled: f64,
    pub responses: f64,
    pub hours: Option<f64>,
    pub teaching: Option<f64>,
    pub course_rate: Option<f64>,
    pub questions: [Option<f64>; 7],
}

/// `num()` in `build-fce.ts`: a finite number, or `None` for blank and unparseable cells.
fn num(s: Option<&str>) -> Option<f64> {
    let s = s?;
    if js::trim(s).is_empty() {
        return None;
    }
    let n = js::string_to_number(s);
    n.is_finite().then_some(n)
}

/// Reads the export into rows, returning them with the number of rows dropped for lacking a course number, a year
/// or a season.
pub fn parse_rows(csv_text: &str) -> (Vec<Row>, usize) {
    let records = csv::Records::parse(csv_text);
    let col = |name: &str| records.column(name);
    let (num_col, year_col, sem_col, section_col, instructor_col) = (
        col("Num"),
        col("Year"),
        col("Sem"),
        col("Section"),
        col("Instructor"),
    );
    let (enrolled_col, responses_col, hours_col, teaching_col, course_col) = (
        col("Total # Students"),
        col("# Responses"),
        col("Hrs Per Week"),
        col("Overall teaching rate"),
        col("Overall course rate"),
    );
    let question_cols = QUESTIONS.map(col);

    let mut rows = Vec::with_capacity(records.len());
    let mut dropped = 0;
    for r in records.rows() {
        let course = names::normalize_fce_course(r.get(num_col).unwrap_or(""));
        let year = r.get(year_col).map_or(f64::NAN, js::string_to_number);
        let season = r.get(sem_col).and_then(Season::parse);
        let (Some(course), Some(season)) = (course, season) else {
            dropped += 1;
            continue;
        };
        if year == 0.0 || year.is_nan() {
            dropped += 1;
            continue;
        }
        let term = names::fce_term_code(year, season);
        let section = r.get(section_col).unwrap_or("").to_owned();
        rows.push(Row {
            course,
            order: names::term_order(&term),
            term,
            mini: names::fce_section_mini(&section, season),
            section,
            instructor: js::trim(&js::collapse_spaces(r.get(instructor_col).unwrap_or("")))
                .to_owned(),
            enrolled: num(r.get(enrolled_col)).unwrap_or(0.0),
            responses: num(r.get(responses_col)).unwrap_or(0.0),
            hours: num(r.get(hours_col)),
            teaching: num(r.get(teaching_col)),
            course_rate: num(r.get(course_col)),
            questions: question_cols.map(|c| num(r.get(c))),
        });
    }
    (rows, dropped)
}

/// Response-weighted mean of `pick` over the items that have it, summed in order as Courseweave does.
fn weighted<T>(
    items: impl IntoIterator<Item = T>,
    pick: impl Fn(&T) -> Option<f64>,
    weight: impl Fn(&T) -> f64,
) -> Option<f64> {
    let (mut sum, mut w) = (0.0, 0.0);
    for x in items {
        let wt = weight(&x);
        let Some(v) = pick(&x) else { continue };
        if wt <= 0.0 {
            continue;
        }
        sum += v * wt;
        w += wt;
    }
    (w != 0.0).then(|| sum / w)
}

/// Groups items by key, keeping keys in first-seen order like a JavaScript `Map`.
fn group_by<'a, T, K: std::hash::Hash + Eq + Clone>(
    items: impl IntoIterator<Item = &'a T>,
    key: impl Fn(&'a T) -> K,
) -> Vec<(K, Vec<&'a T>)> {
    let mut index: HashMap<K, usize> = HashMap::new();
    let mut groups: Vec<(K, Vec<&'a T>)> = Vec::new();
    for item in items {
        let k = key(item);
        match index.get(&k) {
            Some(&i) => groups[i].1.push(item),
            None => {
                index.insert(k.clone(), groups.len());
                groups.push((k, vec![item]));
            }
        }
    }
    groups
}

/// Distinct values in first-seen order, like `[...new Set(values)]`.
fn distinct<'a>(values: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
    let mut seen = BTreeSet::new();
    values.into_iter().filter(|v| seen.insert(*v)).collect()
}

/// A history row: `[term, section, instructor, enrolled, responses, hours, teachingRate, courseRate, questions,
/// mini]`.
pub type RowTuple = (
    String,
    String,
    String,
    Num,
    Num,
    Option<Num>,
    Option<Num>,
    Option<Num>,
    [Option<Num>; 7],
    Option<u8>,
);

/// A course's recent averages: `[hours, courseRate, teachingRate, responses, recentTerms, totalTerms, lastTerm,
/// hoursDelta, enrolledPerTerm, miniShare, miniHalf]`.
///
/// - `hours` is rounded to 0.1 and the ratings to 0.01; `hoursDelta` is recent hours minus the hours of the
///   [`RECENT_TERMS`] terms before them, when both exist.
/// - `miniShare` is the share (0-1, to 0.01) of recent sections that were minis; `miniHalf` is 0 for none, 1 for the
///   first half, 2 for the second, 3 for both.
pub type SummaryTuple = (
    Option<Num>,
    Option<Num>,
    Option<Num>,
    Num,
    usize,
    usize,
    String,
    Option<Num>,
    Num,
    Num,
    u8,
);

/// An instructor's history with one course: `[terms, responses, teachingRate, courseRate, hours, lastTerm,
/// firstTerm]`.
pub type InstructorCourse = (
    usize,
    Num,
    Option<Num>,
    Option<Num>,
    Option<Num>,
    String,
    String,
);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryDoc {
    /// The newest term in the export.
    pub latest: String,
    pub recent_terms: usize,
    pub courses: BTreeMap<String, SummaryTuple>,
}

#[derive(Debug, Serialize)]
pub struct InstructorOut {
    /// Display name, such as "Iliano Cervesato".
    pub name: String,
    pub courses: BTreeMap<String, InstructorCourse>,
    /// `[term, teachingRate, responses]`, oldest first, for terms with responses.
    pub terms: Vec<(String, Option<Num>, Num)>,
}

#[derive(Debug, Serialize)]
pub struct InstructorsDoc {
    pub instructors: BTreeMap<String, InstructorOut>,
}

/// One `fce/courses/<NN>` document.
pub type Chunk = BTreeMap<String, Vec<RowTuple>>;

/// The typed FCE documents.
#[derive(Debug)]
pub struct Fce {
    pub summary: SummaryDoc,
    /// By course-number prefix.
    pub chunks: BTreeMap<String, Chunk>,
    pub instructors: InstructorsDoc,
}

/// A co-taught section's course-level values, from its rows with responses.
struct Section<'a> {
    term: &'a str,
    mini: Option<u8>,
    responses: f64,
    enrolled: f64,
    hours: Option<f64>,
    course_rate: Option<f64>,
}

fn sections_of<'a>(rows: &[&'a Row]) -> Vec<Section<'a>> {
    group_by(rows.iter().copied(), |r| {
        (r.term.as_str(), r.section.as_str())
    })
    .into_iter()
    .map(|(_, g)| Section {
        term: &g[0].term,
        mini: g[0].mini,
        responses: g
            .iter()
            .map(|r| r.responses)
            .fold(f64::NEG_INFINITY, js::max),
        enrolled: g
            .iter()
            .map(|r| r.enrolled)
            .fold(f64::NEG_INFINITY, js::max),
        hours: weighted(g.iter(), |r| r.hours, |r| r.responses),
        course_rate: weighted(g.iter(), |r| r.course_rate, |r| r.responses),
    })
    .collect()
}

fn row_tuple(r: &Row) -> RowTuple {
    (
        r.term.clone(),
        r.section.clone(),
        r.instructor.clone(),
        Num(r.enrolled),
        Num(r.responses),
        opt(r.hours),
        opt(r.teaching),
        opt(r.course_rate),
        r.questions.map(opt),
        r.mini,
    )
}

/// Terms sorted newest first.
fn newest_first(mut terms: Vec<&str>) -> Vec<&str> {
    terms.sort_by(|a, b| js::cmp_num(names::term_order(b), names::term_order(a)));
    terms
}

fn course_summary(rows: &[&Row]) -> Option<SummaryTuple> {
    let answered: Vec<&Row> = rows.iter().copied().filter(|r| r.responses > 0.0).collect();
    let terms = newest_first(distinct(answered.iter().map(|r| r.term.as_str())));
    if terms.is_empty() {
        return None;
    }
    let recent: BTreeSet<&str> = terms.iter().copied().take(RECENT_TERMS).collect();
    let previous: BTreeSet<&str> = terms
        .iter()
        .copied()
        .skip(RECENT_TERMS)
        .take(RECENT_TERMS)
        .collect();
    let sections = sections_of(&answered);
    let recent_sections: Vec<&Section> = sections
        .iter()
        .filter(|s| recent.contains(s.term))
        .collect();
    let recent_rows = answered.iter().filter(|r| recent.contains(r.term.as_str()));

    let hours = weighted(recent_sections.iter(), |s| s.hours, |s| s.responses);
    let prev_hours = weighted(
        sections.iter().filter(|s| previous.contains(s.term)),
        |s| s.hours,
        |s| s.responses,
    );
    let enrolled: f64 = recent_sections.iter().fold(0.0, |n, s| n + s.enrolled);
    let responses: f64 = recent_sections.iter().fold(0.0, |n, s| n + s.responses);
    let minis: Vec<&&Section> = recent_sections
        .iter()
        .filter(|s| s.mini.is_some())
        .collect();
    let halves: BTreeSet<Half> = minis
        .iter()
        .filter_map(|s| s.mini.map(names::mini_half))
        .collect();
    let half_code = if halves.len() == 2 {
        3
    } else if halves.contains(&Half::First) {
        1
    } else if halves.contains(&Half::Second) {
        2
    } else {
        0
    };
    #[expect(clippy::cast_precision_loss, reason = "counts are small")]
    let (recent_len, mini_share) = (
        recent.len() as f64,
        minis.len() as f64 / recent_sections.len() as f64,
    );

    Some((
        opt(js::round_to(hours, 1)),
        opt(js::round_to(
            weighted(recent_sections.iter(), |s| s.course_rate, |s| s.responses),
            2,
        )),
        opt(js::round_to(
            weighted(recent_rows, |r| r.teaching, |r| r.responses),
            2,
        )),
        Num(responses),
        recent.len(),
        terms.len(),
        terms[0].to_owned(),
        match (hours, prev_hours) {
            (Some(h), Some(p)) => opt(js::round_to(Some(h - p), 1)),
            _ => None,
        },
        Num(js::round(enrolled / recent_len)),
        Num(js::round(mini_share * 100.0) / 100.0),
        half_code,
    ))
}

/// Whether a name has a lowercase ASCII letter (`/[a-z]/`), marking a mixed-case spelling.
fn has_lowercase(s: &str) -> bool {
    s.bytes().any(|b| b.is_ascii_lowercase())
}

fn instructor(rows: &[&Row]) -> InstructorOut {
    // Prefer a mixed-case spelling ("ABASCAL, Brooke") when one exists; otherwise the most common.
    let mut variants: Vec<(&str, usize)> =
        group_by(rows.iter().copied(), |r| r.instructor.as_str())
            .into_iter()
            .map(|(name, g)| (name, g.len()))
            .collect();
    variants.sort_by(|a, b| {
        has_lowercase(b.0)
            .cmp(&has_lowercase(a.0))
            .then(b.1.cmp(&a.1))
    });
    let name = names::display_fce_name(variants[0].0);

    let mut courses = BTreeMap::new();
    for (course, cr) in group_by(rows.iter().copied(), |r| r.course.as_str()) {
        let answered: Vec<&Row> = cr.iter().copied().filter(|r| r.responses > 0.0).collect();
        let mut taught = distinct(cr.iter().map(|r| r.term.as_str()));
        taught.sort_by(|a, b| js::cmp_num(names::term_order(a), names::term_order(b)));
        courses.insert(
            course.to_owned(),
            (
                taught.len(),
                Num(answered.iter().fold(0.0, |n, r| n + r.responses)),
                opt(js::round_to(
                    weighted(answered.iter(), |r| r.teaching, |r| r.responses),
                    2,
                )),
                opt(js::round_to(
                    weighted(answered.iter(), |r| r.course_rate, |r| r.responses),
                    2,
                )),
                opt(js::round_to(
                    weighted(answered.iter(), |r| r.hours, |r| r.responses),
                    1,
                )),
                taught[taught.len() - 1].to_owned(),
                taught[0].to_owned(),
            ),
        );
    }

    let mut terms = group_by(rows.iter().copied().filter(|r| r.responses > 0.0), |r| {
        r.term.as_str()
    });
    terms.sort_by(|a, b| js::cmp_num(names::term_order(a.0), names::term_order(b.0)));
    let terms = terms
        .into_iter()
        .map(|(term, tr)| {
            (
                term.to_owned(),
                opt(js::round_to(
                    weighted(tr.iter(), |r| r.teaching, |r| r.responses),
                    2,
                )),
                Num(tr.iter().fold(0.0, |n, r| n + r.responses)),
            )
        })
        .collect();
    InstructorOut {
        name,
        courses,
        terms,
    }
}

/// Builds the documents from the export's rows, in their CSV order.
pub fn build(rows: &[Row]) -> Result<Fce> {
    let Some(first) = rows.first() else {
        bail!("the FCE export has no rows");
    };
    // The first row of the newest term, as `reduce` with `>` finds it.
    let latest = rows
        .iter()
        .fold(first, |best, r| if r.order > best.order { r } else { best });

    let mut courses = BTreeMap::new();
    let mut chunks: BTreeMap<String, Chunk> = BTreeMap::new();
    for (course, mut course_rows) in group_by(rows, |r| r.course.as_str()) {
        course_rows.sort_by(|a, b| {
            js::cmp_num(b.order, a.order)
                .then_with(|| js::locale_compare(&a.section, &b.section))
                .then_with(|| js::locale_compare(&a.instructor, &b.instructor))
        });
        chunks.entry(course[..2].to_owned()).or_default().insert(
            course.to_owned(),
            course_rows.iter().map(|r| row_tuple(r)).collect(),
        );
        if let Some(summary) = course_summary(&course_rows) {
            courses.insert(course.to_owned(), summary);
        }
    }

    let mut keys: HashMap<&str, String> = HashMap::new();
    let mut by_instructor: Vec<(String, Vec<&Row>)> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for r in rows.iter().filter(|r| !r.instructor.is_empty()) {
        let key = keys
            .entry(r.instructor.as_str())
            .or_insert_with(|| names::fce_name_keys(&r.instructor).full);
        match index.get(key.as_str()) {
            Some(&i) => by_instructor[i].1.push(r),
            None => {
                index.insert(key.clone(), by_instructor.len());
                by_instructor.push((key.clone(), vec![r]));
            }
        }
    }
    let instructors = by_instructor
        .into_iter()
        .map(|(key, list)| (key, instructor(&list)))
        .collect();

    Ok(Fce {
        summary: SummaryDoc {
            latest: latest.term.clone(),
            recent_terms: RECENT_TERMS,
            courses,
        },
        chunks,
        instructors: InstructorsDoc { instructors },
    })
}

impl Fce {
    /// The documents as `(name, JSON bytes)` pairs, sorted by name.
    pub fn documents(&self) -> Result<Vec<Document>> {
        let mut docs = vec![
            ("fce/summary".to_owned(), serde_json::to_vec(&self.summary)?),
            (
                "fce/courses/index".to_owned(),
                serde_json::to_vec(&self.chunks.keys().collect::<Vec<_>>())?,
            ),
            (
                "fce/instructors".to_owned(),
                serde_json::to_vec(&self.instructors)?,
            ),
        ];
        for (prefix, chunk) in &self.chunks {
            docs.push((format!("fce/courses/{prefix}"), serde_json::to_vec(chunk)?));
        }
        docs.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(docs)
    }
}

/// What a build wrote, for the builder's log and the catalog's `meta`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    /// Rows read from the export.
    pub rows: usize,
    /// Rows without a course number, year or season.
    pub dropped: usize,
    /// Courses with responses, in `fce/summary`.
    pub courses: usize,
    pub instructors: usize,
    pub earliest: Option<String>,
    pub latest: Option<String>,
    pub documents: usize,
    /// Uncompressed JSON bytes over all documents.
    pub bytes: usize,
}

/// Builds every FCE document from the export's text.
pub fn documents(csv_text: &str) -> Result<(Vec<Document>, Summary)> {
    documents_and_data(csv_text).map(|(docs, summary, _)| (docs, summary))
}

/// [`documents`], plus the typed documents for builders that read them (the instructor lists).
pub fn documents_and_data(csv_text: &str) -> Result<(Vec<Document>, Summary, Fce)> {
    let (rows, dropped) = parse_rows(csv_text);
    let fce = build(&rows)?;
    let docs = fce.documents()?;
    let earliest = rows.iter().fold(
        &rows[0],
        |best, r| if r.order < best.order { r } else { best },
    );
    let summary = Summary {
        rows: rows.len(),
        dropped,
        courses: fce.summary.courses.len(),
        instructors: fce.instructors.instructors.len(),
        earliest: Some(earliest.term.clone()),
        latest: Some(fce.summary.latest.clone()),
        documents: docs.len(),
        bytes: docs.iter().map(|(_, json)| json.len()).sum(),
    };
    Ok((docs, summary, fce))
}

/// Reads `<data_dir>/fces.csv` and stores the FCE documents in the catalog, member-only. Without the export it
/// writes nothing and logs a warning, as Courseweave does.
pub fn write(writer: &Writer, data_dir: &Path) -> Result<Summary> {
    let path = data_dir.join(CSV_FILE);
    if !path.exists() {
        warn!(path = %path.display(), "no FCE export; skipping FCE documents");
        return Ok(Summary::default());
    }
    let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    let (docs, summary) = documents(&String::from_utf8_lossy(&bytes))
        .with_context(|| format!("building FCE documents from {}", path.display()))?;
    for (name, json) in &docs {
        writer.put_raw(name, Audience::Member, json)?;
    }
    info!(
        rows = summary.rows,
        dropped = summary.dropped,
        courses = summary.courses,
        instructors = summary.instructors,
        documents = summary.documents,
        bytes = summary.bytes,
        "wrote FCE documents"
    );
    Ok(summary)
}

#[cfg(test)]
mod tests;
