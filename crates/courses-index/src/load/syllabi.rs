//! Walk `exported/syllabi/<term>/<dept>/<code_nodash>[-<section>].<ext>`
//! and attach one [`SyllabusRef`] per file to its course, so "does this
//! specific section in this specific term have a syllabus" is answerable
//! downstream. The file contents themselves are not opened; refs point at
//! the CDN copies of the same tree.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use tracing::debug;
use walkdir::WalkDir;

use crate::doc::{Course, CourseId, SyllabusRef};

#[derive(Debug, Default)]
pub struct SyllabusLoad {
    pub files_scanned: usize,
    pub files_unparsable: usize,
    pub orphan_codes: HashSet<String>,
    pub courses_with_any_syllabus: usize,
}

/// Scan `syllabi_root` and populate `syllabi` on each course.
pub fn load_syllabi(syllabi_root: &Path, courses: &mut [Course]) -> Result<SyllabusLoad> {
    let code_to_id: HashMap<String, CourseId> =
        courses.iter().map(|c| (c.code.clone(), c.id)).collect();

    let mut by_course: HashMap<CourseId, Vec<SyllabusRef>> = HashMap::new();
    let mut out = SyllabusLoad::default();

    for entry in WalkDir::new(syllabi_root).min_depth(3).max_depth(3) {
        let Ok(entry) = entry else {
            continue;
        };
        if !entry.file_type().is_file() {
            continue;
        }

        out.files_scanned += 1;

        let mut rev = entry.path().components().rev();
        let dept = rev.nth(1).and_then(|c| c.as_os_str().to_str());
        let term = rev.next().and_then(|c| c.as_os_str().to_str());
        let stem = entry.path().file_stem().and_then(|s| s.to_str());
        let ext = entry.path().extension().and_then(|s| s.to_str());
        let (Some(dept), Some(term), Some(stem), Some(ext)) = (dept, term, stem, ext) else {
            out.files_unparsable += 1;
            continue;
        };

        let (dashless, section) = match stem.split_once('-') {
            Some((d, s)) => (d, s),
            None => (stem, ""),
        };
        let Some(code) = normalize_code(dashless) else {
            out.files_unparsable += 1;
            continue;
        };
        let Some(&course_id) = code_to_id.get(&code) else {
            out.orphan_codes.insert(code);
            continue;
        };

        by_course.entry(course_id).or_default().push(SyllabusRef {
            term: Arc::from(term),
            dept: Arc::from(dept),
            section: section.to_string(),
            ext: ext.to_string(),
        });
    }

    for course in courses.iter_mut() {
        if let Some(mut refs) = by_course.remove(&course.id) {
            refs.sort_by(|a, b| {
                term_sort_key(&a.term)
                    .cmp(&term_sort_key(&b.term))
                    .then_with(|| a.term.cmp(&b.term))
                    .then_with(|| a.section.cmp(&b.section))
            });

            course.syllabi = refs;
            out.courses_with_any_syllabus += 1;
        }
    }

    debug!(
        scanned = out.files_scanned,
        unparsable = out.files_unparsable,
        orphans = out.orphan_codes.len(),
        courses = out.courses_with_any_syllabus,
        "syllabi load complete"
    );
    Ok(out)
}

fn normalize_code(raw: &str) -> Option<String> {
    if !raw.chars().all(|c| c.is_ascii_digit()) || raw.len() < 3 {
        return None;
    }

    let padded = format!("{:0>5}", raw);
    let (dept, num) = padded.split_at(2);
    Some(format!("{dept}-{num}"))
}

/// Sort terms newest-first. F26 > S26 > M25 > F25 > ...
fn term_sort_key(term: &str) -> (i32, u8) {
    let (season, year) = term.split_at(1);
    let year: i32 = year.parse().unwrap_or(0);

    let season_rank = match season {
        "F" => 3,
        "N" => 2,
        "M" => 1,
        "S" => 0,
        _ => 4,
    };
    (-year, 4 - season_rank)
}
