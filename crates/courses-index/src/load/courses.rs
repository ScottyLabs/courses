//! Parse every `exported/courses_history/<code>/info.json` into a partial
//! `Course`, populating every field derivable from `info.json` alone (code,
//! name, units, description, attributes, gened tags, skills, prereq text,
//! co/anti/equiv cross-references, campuses, offerings, info.json-listed
//! instructors). Fields sourced from elsewhere stay at their defaults.
//!
//! Course ids are assigned at the end by sorting surviving courses by `code`
//! ascending and numbering from 0, which gives the stable-across-runs
//! property the binary format requires.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::Deserialize;
use tracing::{debug, warn};
use walkdir::WalkDir;

use crate::doc::{Course, CourseId, GenEdTag, InstructorId, InstructorRef, Sem, SemesterOffered};

/// Raw subset of an `info.json` file that contributes to the in-memory
/// [`Course`]. Fields we ignore at the catalog level (`enrollment_action_windows`,
/// `student_context`, `alerts`, etc.) are simply not listed.
#[derive(Debug, Deserialize)]
struct InfoJson {
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    long_desc: Option<String>,
    #[serde(default)]
    units: Option<f32>,
    #[serde(default)]
    min_units: Option<f32>,
    #[serde(default)]
    max_units: Option<f32>,
    #[serde(default)]
    website: Option<String>,
    #[serde(default)]
    attributes: Vec<Attribute>,
    #[serde(default)]
    offering_tags: Vec<OfferingTag>,
    #[serde(default)]
    skills: Vec<Skill>,
    #[serde(default)]
    prereqs: Option<Prereqs>,
    #[serde(default)]
    co_reqs: Vec<CourseRef>,
    #[serde(default)]
    anti_reqs: Vec<CourseRef>,
    #[serde(default)]
    equiv: Vec<CourseRef>,
    #[serde(default)]
    offered_in_campuses: Vec<u32>,
    #[serde(default)]
    offerings: Vec<Offering>,
    #[serde(default)]
    instructors: Vec<RawInstructor>,
    #[serde(default)]
    student_sets: Vec<StudentSet>,
}

#[derive(Debug, Deserialize)]
struct Attribute {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OfferingTag {
    name: String,
    #[serde(default)]
    sem: Option<u8>,
}

#[derive(Debug, Deserialize)]
struct Skill {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Prereqs {
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CourseRef {
    #[serde(default)]
    code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Offering {
    #[serde(default)]
    campus_id: Option<u32>,
    #[serde(default)]
    semesters: Vec<OfferingSemester>,
}

#[derive(Debug, Deserialize)]
struct OfferingSemester {
    semester: u8,
    year: u16,
}

#[derive(Debug, Deserialize)]
struct RawInstructor {
    id: InstructorId,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    first_name: Option<String>,
    #[serde(default)]
    last_name: Option<String>,
    #[serde(default)]
    semesters_taught: Vec<OfferingSemester>,
}

#[derive(Debug, Deserialize)]
struct StudentSet {
    name: String,
}

/// Output of the courses phase.
#[derive(Debug)]
pub struct CourseLoad {
    pub courses: Vec<Course>,
    pub skipped_malformed_dirname: usize,
    pub skipped_missing_info: usize,
    pub skipped_parse_error: usize,
}

/// Walk every subdirectory of `courses_history_root`, parse its `info.json`,
/// and build a partial `Course` per entry.
pub fn load_courses(courses_history_root: &Path) -> Result<CourseLoad> {
    let dirs: Vec<_> = WalkDir::new(courses_history_root)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| match e {
            Ok(entry) if entry.file_type().is_dir() => Some(entry),
            Ok(_) => None,
            Err(err) => {
                warn!(?err, "walkdir error in courses_history");
                None
            }
        })
        .collect();

    let (rows, malformed, missing, parse_err) = parse_all(&dirs);

    let mut staged = rows;
    staged.sort_by(|a, b| a.code.cmp(&b.code));
    staged.dedup_by(|a, b| a.code == b.code);

    let courses: Vec<Course> = staged
        .into_iter()
        .enumerate()
        .map(|(i, c)| Course {
            id: i as CourseId,
            ..c
        })
        .collect();

    debug!(
        kept = courses.len(),
        malformed_dirname = malformed,
        missing_info = missing,
        parse_errors = parse_err,
        "courses load complete"
    );

    Ok(CourseLoad {
        courses,
        skipped_malformed_dirname: malformed,
        skipped_missing_info: missing,
        skipped_parse_error: parse_err,
    })
}

fn parse_all(dirs: &[walkdir::DirEntry]) -> (Vec<Course>, usize, usize, usize) {
    let results: Vec<ParsedRow> = dirs.par_iter().map(parse_one).collect();
    let mut rows = Vec::with_capacity(results.len());
    let (mut malformed, mut missing, mut parse_err) = (0, 0, 0);
    for row in results {
        match row {
            ParsedRow::Course(c) => rows.push(*c),
            ParsedRow::MalformedDirname => malformed += 1,
            ParsedRow::MissingInfo => missing += 1,
            ParsedRow::ParseError => parse_err += 1,
        }
    }
    (rows, malformed, missing, parse_err)
}

enum ParsedRow {
    Course(Box<Course>),
    MalformedDirname,
    MissingInfo,
    ParseError,
}

fn parse_one(entry: &walkdir::DirEntry) -> ParsedRow {
    let dir_name = match entry.file_name().to_str() {
        Some(s) => s.to_string(),
        None => return ParsedRow::MalformedDirname,
    };
    if dir_name.len() < 3 {
        return ParsedRow::MalformedDirname;
    }
    let info_path = entry.path().join("info.json");
    if !info_path.exists() {
        return ParsedRow::MissingInfo;
    }
    let bytes =
        match fs::read(&info_path).with_context(|| format!("reading {}", info_path.display())) {
            Ok(b) => b,
            Err(err) => {
                warn!(?err, "read failed");
                return ParsedRow::ParseError;
            }
        };
    let info: InfoJson = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(err) => {
            warn!(path = %info_path.display(), ?err, "parse failed, skipping");
            return ParsedRow::ParseError;
        }
    };

    let code = info
        .code
        .clone()
        .filter(|c| !c.is_empty())
        .unwrap_or_else(|| derive_code_from_dirname(&dir_name));

    let (dept, course_num) = split_code(&code);
    ParsedRow::Course(Box::new(assemble(info, code, dept, course_num)))
}

fn derive_code_from_dirname(dir_name: &str) -> String {
    let (first, rest) = dir_name.split_at(2);
    format!("{first}-{rest}")
}

fn split_code(code: &str) -> (String, u32) {
    if let Some((a, b)) = code.split_once('-') {
        let num = b.parse().unwrap_or(0);
        (a.to_string(), num)
    } else {
        (String::new(), 0)
    }
}

fn assemble(info: InfoJson, code: String, dept: String, course_num: u32) -> Course {
    let units = info.units.unwrap_or(0.0);
    let units_min = info.min_units.unwrap_or(units);
    let units_max = info.max_units.unwrap_or(units);

    let attribute_tags = info
        .attributes
        .into_iter()
        .map(|a| Arc::from(a.name))
        .collect();
    let gened_tags = info
        .offering_tags
        .into_iter()
        .filter_map(|t| {
            t.sem.map(|sem| GenEdTag {
                name: Arc::from(t.name),
                sem,
            })
        })
        .collect();
    let skills: Vec<Arc<str>> = info.skills.into_iter().map(|s| Arc::from(s.name)).collect();

    let prereqs_text = info.prereqs.and_then(|p| p.text).filter(|s| !s.is_empty());

    let coreq_codes = info
        .co_reqs
        .into_iter()
        .filter_map(|c| c.code)
        .filter(|c| !c.is_empty())
        .collect();
    let antireq_codes = info
        .anti_reqs
        .into_iter()
        .filter_map(|c| c.code)
        .filter(|c| !c.is_empty())
        .collect();
    let equiv_codes = info
        .equiv
        .into_iter()
        .filter_map(|c| c.code)
        .filter(|c| !c.is_empty())
        .collect();

    let mut semesters_offered: Vec<SemesterOffered> = info
        .offerings
        .into_iter()
        .flat_map(|o| {
            let campus_id = o.campus_id.unwrap_or(0);
            o.semesters.into_iter().filter_map(move |s| {
                Sem::from_u8(s.semester).map(|sem| SemesterOffered {
                    year: s.year,
                    sem,
                    campus_id,
                })
            })
        })
        .collect();
    semesters_offered.sort_by(|a, b| {
        (b.year, a.sem as u8, a.campus_id).cmp(&(a.year, b.sem as u8, b.campus_id))
    });
    semesters_offered.dedup();

    let instructors_recent: Vec<InstructorRef> = info
        .instructors
        .into_iter()
        .map(|i| {
            let name = format_name(i.first_name.as_deref(), i.last_name.as_deref());
            let semesters_taught = i
                .semesters_taught
                .into_iter()
                .filter_map(|s| Sem::from_u8(s.semester).map(|sem| (s.year, sem)))
                .collect();
            InstructorRef {
                id: i.id,
                name,
                username: i.username,
                semesters_taught,
            }
        })
        .collect();

    let level: Option<Arc<str>> = info
        .student_sets
        .first()
        .map(|s| Arc::from(title_case(&s.name)));

    Course {
        id: 0,
        code,
        dept,
        course_num,
        name: info.name.unwrap_or_default(),
        description: Arc::from(info.long_desc.unwrap_or_default()),
        units,
        units_min,
        units_max,
        school: None,
        level,
        attribute_tags,
        gened_tags,
        skills,
        prereqs_text,
        coreq_codes,
        antireq_codes,
        equiv_codes,
        website: info.website.filter(|s| !s.is_empty()),
        campuses: info.offered_in_campuses,
        semesters_offered,
        instructors_recent,
        programs: Vec::new(),
        fce_aggregates: None,
        has_syllabus_terms: Vec::new(),
        pagerank: 0.0,
    }
}

fn format_name(first: Option<&str>, last: Option<&str>) -> String {
    match (first, last) {
        (Some(f), Some(l)) => format!("{f} {l}"),
        (Some(f), None) => f.to_string(),
        (None, Some(l)) => l.to_string(),
        (None, None) => String::new(),
    }
}

fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}
