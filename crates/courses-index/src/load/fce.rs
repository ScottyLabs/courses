//! Read `data/fces.csv` into one [`FceRow`] per survey row and roll the rows
//! up into a per-course [`FceAggregate`]. Likert ratings are kept as
//! `Option<f32>` because roughly 5% of rows have no responses and report
//! enrollment counts only; those rows still land in the row store but are
//! skipped when averaging.
//!
//! Course matching uses the dashless `Num` column. Most rows are 5-char
//! `<dept2><num3>`, but the dump also contains a small number of legacy
//! prefixed forms like `FA14-12-100` and a handful of rows with the dash
//! already inserted (`05-436`). Both shapes are normalized to `XX-YYY`.
//!
//! Instructor ids are *not* assigned here. Each row carries
//! `instructor_fce_key` (a normalized `LASTNAME, FIRST` string) and leaves
//! `instructor_id` at `None`. A later professor-merge pass joins those keys
//! against the Stellic instructor set.
//!
//! Rows whose `College` is `Teaching Assistants` are dropped at parse
//! time. They reuse the course's Num but list TA usernames as instructors
//! and never carry Likert data, so they would only pollute the row store.

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::{debug, warn};

use crate::doc::{Course, CourseId, FceAggregate, FceRow, Sem};

/// Number of most-recent semesters folded into [`Course::fce_aggregates`].
const AGGREGATE_RECENT_SEMESTERS: usize = 4;

/// Output of the FCE phase.
#[derive(Debug, Default)]
pub struct FceLoad {
    pub rows: Vec<FceRow>,
    pub rows_skipped_no_match: usize,
    pub rows_skipped_ta: usize,
    pub rows_skipped_bad_num: usize,
}

/// Read `fce_csv` and attach aggregates to each course in `courses`. Returns
/// the per-row store; the orchestrator owns it.
pub fn load_fces(fce_csv: &Path, courses: &mut [Course]) -> Result<FceLoad> {
    let code_to_id: HashMap<String, CourseId> =
        courses.iter().map(|c| (c.code.clone(), c.id)).collect();

    let file = File::open(fce_csv).with_context(|| format!("opening {}", fce_csv.display()))?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let mut out = FceLoad::default();

    for record in reader.deserialize::<RawRow>() {
        let raw = match record {
            Ok(r) => r,
            Err(err) => {
                warn!(?err, "fce row parse failed");
                continue;
            }
        };

        if raw.college.trim() == "Teaching Assistants" {
            out.rows_skipped_ta += 1;
            continue;
        }

        let code = match normalize_code(&raw.num) {
            Some(c) => c,
            None => {
                out.rows_skipped_bad_num += 1;
                continue;
            }
        };
        let Some(&course_id) = code_to_id.get(&code) else {
            out.rows_skipped_no_match += 1;
            continue;
        };

        let Some(sem) = Sem::from_csv(&raw.sem) else {
            continue;
        };

        out.rows.push(FceRow {
            course_id,
            year: raw.year,
            sem,
            section: raw.section,
            instructor_id: None,
            instructor_fce_key: normalize_instructor_key(&raw.instructor),
            hrs_per_week: parse_opt_f32(&raw.hrs_per_week),
            interest: parse_opt_f32(&raw.interest),
            instructor_clarity: parse_opt_f32(&raw.clarity),
            feedback: parse_opt_f32(&raw.feedback),
            importance: parse_opt_f32(&raw.importance),
            explanation: parse_opt_f32(&raw.explanation),
            respect: parse_opt_f32(&raw.respect),
            overall_teaching: parse_opt_f32(&raw.overall_teaching),
            overall_course: parse_opt_f32(&raw.overall_course),
            n_responses: parse_opt_u32(&raw.n_responses).unwrap_or(0),
            response_rate: parse_opt_f32(&raw.response_rate),
            total_students: parse_opt_u32(&raw.total_students),
            course_level: nonempty(raw.course_level),
            college: nonempty(raw.college),
        });
    }

    attach_aggregates(&out.rows, courses);

    debug!(
        rows = out.rows.len(),
        skipped_no_match = out.rows_skipped_no_match,
        skipped_ta = out.rows_skipped_ta,
        skipped_bad_num = out.rows_skipped_bad_num,
        "fce load complete"
    );
    Ok(out)
}

#[derive(Debug, Deserialize)]
struct RawRow {
    #[serde(rename = "Year")]
    year: u16,
    #[serde(rename = "Sem")]
    sem: String,
    #[serde(rename = "College")]
    college: String,
    #[serde(rename = "Dept")]
    #[allow(dead_code)]
    dept: String,
    #[serde(rename = "Num")]
    num: String,
    #[serde(rename = "Section")]
    section: String,
    #[serde(rename = "Instructor")]
    instructor: String,
    #[serde(rename = "Course Name")]
    #[allow(dead_code)]
    course_name: String,
    #[serde(rename = "Course Level")]
    course_level: String,
    #[serde(rename = "Total # Students")]
    total_students: String,
    #[serde(rename = "# Responses")]
    n_responses: String,
    #[serde(rename = "Response Rate")]
    response_rate: String,
    #[serde(rename = "Hrs Per Week")]
    hrs_per_week: String,
    #[serde(rename = "Interest in student learning")]
    interest: String,
    #[serde(rename = "Clearly explain course requirements")]
    clarity: String,
    #[serde(rename = "Clear learning objectives & goals")]
    #[allow(dead_code)]
    objectives: String,
    #[serde(rename = "Instructor provides feedback to students to improve")]
    feedback: String,
    #[serde(rename = "Demonstrate importance of subject matter")]
    importance: String,
    #[serde(rename = "Explains subject matter of course")]
    explanation: String,
    #[serde(rename = "Show respect for all students")]
    respect: String,
    #[serde(rename = "Overall teaching rate")]
    overall_teaching: String,
    #[serde(rename = "Overall course rate")]
    overall_course: String,
}

fn normalize_code(raw: &str) -> Option<String> {
    let trimmed = raw.trim();

    // The legacy "FA14-12-100" / "SP19-21-127" forms carry a non-numeric
    // term prefix that needs to come off before the rest can be parsed.
    let without_prefix = match trimmed.split_once('-') {
        Some((head, tail)) if head.chars().any(|c| c.is_ascii_alphabetic()) => tail,
        _ => trimmed,
    };

    if let Some((dept, num)) = without_prefix.split_once('-') {
        if dept.chars().all(|c| c.is_ascii_digit()) && num.chars().all(|c| c.is_ascii_digit()) {
            return Some(format!("{:0>2}-{:0>3}", dept, num));
        }
        return None;
    }

    if !without_prefix.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let padded = format!("{:0>5}", without_prefix);
    let (dept, num) = padded.split_at(2);
    Some(format!("{dept}-{num}"))
}

fn normalize_instructor_key(raw: &str) -> String {
    raw.trim().trim_matches('"').trim().to_ascii_uppercase()
}

fn parse_opt_f32(s: &str) -> Option<f32> {
    let t = s.trim();
    if t.is_empty() { None } else { t.parse().ok() }
}

fn parse_opt_u32(s: &str) -> Option<u32> {
    let t = s.trim();
    if t.is_empty() { None } else { t.parse().ok() }
}

fn nonempty(s: String) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

fn attach_aggregates(rows: &[FceRow], courses: &mut [Course]) {
    let mut by_course: HashMap<CourseId, Vec<&FceRow>> = HashMap::new();
    for row in rows {
        by_course.entry(row.course_id).or_default().push(row);
    }

    for course in courses.iter_mut() {
        let Some(rows) = by_course.remove(&course.id) else {
            continue;
        };
        course.fce_aggregates = aggregate_recent(rows);
    }
}

fn aggregate_recent(mut rows: Vec<&FceRow>) -> Option<FceAggregate> {
    rows.sort_by_key(|r| std::cmp::Reverse((r.year, r.sem as u8)));

    let mut kept: Vec<&FceRow> = Vec::new();
    let mut seen_terms: Vec<(u16, Sem)> = Vec::new();
    for row in rows {
        let term = (row.year, row.sem);
        if !seen_terms.contains(&term) {
            if seen_terms.len() == AGGREGATE_RECENT_SEMESTERS {
                break;
            }
            seen_terms.push(term);
        }
        kept.push(row);
    }
    if kept.is_empty() {
        return None;
    }

    let mut acc = Accumulator::default();
    for row in &kept {
        acc.push(row);
    }
    acc.finish(seen_terms.len() as u32)
}

#[derive(Default)]
struct Accumulator {
    hrs: SumCount,
    interest: SumCount,
    clarity: SumCount,
    feedback: SumCount,
    importance: SumCount,
    explanation: SumCount,
    respect: SumCount,
    overall_teaching: SumCount,
    overall_course: SumCount,
}

impl Accumulator {
    fn push(&mut self, row: &FceRow) {
        self.hrs.add(row.hrs_per_week);
        self.interest.add(row.interest);
        self.clarity.add(row.instructor_clarity);
        self.feedback.add(row.feedback);
        self.importance.add(row.importance);
        self.explanation.add(row.explanation);
        self.respect.add(row.respect);
        self.overall_teaching.add(row.overall_teaching);
        self.overall_course.add(row.overall_course);
    }

    fn finish(self, n_semesters: u32) -> Option<FceAggregate> {
        let any = self.hrs.n
            + self.interest.n
            + self.clarity.n
            + self.feedback.n
            + self.importance.n
            + self.explanation.n
            + self.respect.n
            + self.overall_teaching.n
            + self.overall_course.n;
        if any == 0 {
            return None;
        }
        Some(FceAggregate {
            n_semesters,
            hrs_per_week: self.hrs.mean(),
            interest: self.interest.mean(),
            clarity: self.clarity.mean(),
            feedback: self.feedback.mean(),
            importance: self.importance.mean(),
            explanation: self.explanation.mean(),
            respect: self.respect.mean(),
            overall_teaching: self.overall_teaching.mean(),
            overall_course: self.overall_course.mean(),
        })
    }
}

#[derive(Default)]
struct SumCount {
    sum: f64,
    n: u32,
}

impl SumCount {
    fn add(&mut self, v: Option<f32>) {
        if let Some(x) = v {
            self.sum += x as f64;
            self.n += 1;
        }
    }

    fn mean(&self) -> f32 {
        if self.n == 0 {
            0.0
        } else {
            (self.sum / self.n as f64) as f32
        }
    }
}
