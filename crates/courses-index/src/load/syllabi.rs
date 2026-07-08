//! Walk `exported/syllabi/<term>/<dept>/<dashless>-<section>.<ext>` and set
//! `Course.has_syllabus_terms` to the sorted list of short term codes
//! (`F19`, `S22`, `M25`, ...) for which any syllabus file exists. The file
//! contents themselves are not opened; this pass is just a presence map.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::Result;
use tracing::debug;
use walkdir::WalkDir;

use crate::doc::{Course, CourseId};

#[derive(Debug, Default)]
pub struct SyllabusLoad {
    pub files_scanned: usize,
    pub files_unparsable: usize,
    pub orphan_codes: HashSet<String>,
    pub courses_with_any_syllabus: usize,
}

/// Scan `syllabi_root` and populate `has_syllabus_terms` on each course.
pub fn load_syllabi(syllabi_root: &Path, courses: &mut [Course]) -> Result<SyllabusLoad> {
    let code_to_id: HashMap<String, CourseId> =
        courses.iter().map(|c| (c.code.clone(), c.id)).collect();

    let mut by_course: HashMap<CourseId, HashSet<String>> = HashMap::new();
    let mut out = SyllabusLoad::default();

    for entry in WalkDir::new(syllabi_root).min_depth(3).max_depth(3) {
        let Ok(entry) = entry else {
            continue;
        };
        if !entry.file_type().is_file() {
            continue;
        }
        out.files_scanned += 1;

        let term = match entry
            .path()
            .components()
            .rev()
            .nth(2)
            .and_then(|c| c.as_os_str().to_str())
        {
            Some(t) => t.to_string(),
            None => {
                out.files_unparsable += 1;
                continue;
            }
        };

        let stem = match entry.path().file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => {
                out.files_unparsable += 1;
                continue;
            }
        };

        let dashless = match stem.split_once('-') {
            Some((d, _)) => d,
            None => stem,
        };
        let Some(code) = normalize_code(dashless) else {
            out.files_unparsable += 1;
            continue;
        };
        let Some(&course_id) = code_to_id.get(&code) else {
            out.orphan_codes.insert(code);
            continue;
        };
        by_course.entry(course_id).or_default().insert(term);
    }

    for course in courses.iter_mut() {
        if let Some(terms) = by_course.remove(&course.id) {
            let mut sorted: Vec<String> = terms.into_iter().collect();
            sorted.sort_by(|a, b| {
                term_sort_key(a)
                    .cmp(&term_sort_key(b))
                    .then_with(|| a.cmp(b))
            });
            course.has_syllabus_terms = sorted.into_iter().map(std::sync::Arc::from).collect();
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
    let bytes = term.as_bytes();
    if bytes.len() != 3 {
        return (0, 0);
    }
    let letter = bytes[0];
    let yy: i32 = std::str::from_utf8(&bytes[1..])
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let sem_rank: u8 = match letter {
        b'F' => 0,
        b'S' => 1,
        b'M' | b'N' => 2,
        _ => 3,
    };
    (-yy, sem_rank)
}
