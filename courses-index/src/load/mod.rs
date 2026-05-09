//! Load the scrape sources under `exported/` and `data/fces.csv` into the
//! in-memory document structs in [`crate::doc`]. Submodules each read one
//! source and contribute a disjoint set of fields to the `Course` /
//! `Professor` / `SectionTime` / `FceRow` stores, so passes that touch
//! non-overlapping fields can run in parallel. The orchestrator lives in
//! [`run`].

pub mod courses;
pub mod fce;
pub mod pagerank;
pub mod professors;
pub mod programs;
pub mod sections;
pub mod syllabi;

pub use courses::{CourseLoad, load_courses};
pub use fce::{FceLoad, load_fces};
pub use pagerank::{PageRankResult, compute_pagerank};
pub use professors::{ProfessorBuild, build_professors};
pub use programs::{ProgramLoad, load_programs};
pub use sections::{SectionLoad, load_sections};
pub use syllabi::{SyllabusLoad, load_syllabi};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::doc::{Course, FceRow, Professor, SectionTime};

/// Aggregated output of a full load pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corpus {
    pub courses: Vec<Course>,
    pub professors: Vec<Professor>,
    pub sections: Vec<SectionTime>,
    pub fce_rows: Vec<FceRow>,
}

/// Walk every immediate subdirectory of `exported_root` (one per student,
/// named by AndrewID) and consolidate the per-student scrapes into a single
/// corpus. `info.json` (catalog data) is loaded from the first student's
/// subdir since it is identical across students. Sections, programs, and
/// syllabi are loaded from every subdir and deduped by `section_id` /
/// natural keys.
pub fn run(exported_root: &Path, fce_csv: &Path) -> Result<Corpus> {
    let student_roots = find_student_roots(exported_root)?;
    let primary = student_roots
        .first()
        .ok_or_else(|| anyhow!("no student subdirs found under {}", exported_root.display()))?;

    let CourseLoad { mut courses, .. } = load_courses(&primary.join("courses_history"))?;

    let mut sections: Vec<SectionTime> = Vec::new();
    for root in &student_roots {
        let SectionLoad { sections: rs, .. } =
            load_sections(&root.join("courses_history"), &mut courses)?;
        sections.extend(rs);
    }

    // Same section can appear in multiple students' scrapes; dedup by section_id.
    let mut seen: HashSet<_> = HashSet::new();
    sections.retain(|s| seen.insert(s.section_id));

    let FceLoad {
        rows: mut fce_rows, ..
    } = load_fces(fce_csv, &mut courses)?;

    for root in &student_roots {
        let _ = load_programs(&root.join("programs"), &mut courses)?;
    }

    let _ = load_syllabi(&exported_root.join("syllabi"), &mut courses)?;
    let _ = compute_pagerank(&mut courses);
    let ProfessorBuild {
        professors,
        fce_rows_matched,
        fce_rows_ambiguous,
        fce_rows_unmatched,
        fce_only_professors,
    } = build_professors(&courses, &sections, &mut fce_rows);
    tracing::debug!(
        professors = professors.len(),
        fce_rows_matched,
        fce_rows_ambiguous,
        fce_rows_unmatched,
        fce_only_professors,
        "professor merge complete"
    );
    let mut corpus = Corpus {
        courses,
        professors,
        sections,
        fce_rows,
    };
    dedup_descriptions(&mut corpus);
    Ok(corpus)
}

/// Collapse repeated string values across courses to shared `Arc<str>`
/// allocations. Targets every field with low expected cardinality:
/// `description` (boilerplate text repeats across thousands of courses),
/// `level` and `school` (handful of distinct values total),
/// `attribute_tags` and `gened_tags` (a few dozen each), `skills` (a few
/// hundred), and `has_syllabus_terms` (~30 short term codes). Pure heap
/// optimization with no on-disk format change.
pub fn dedup_descriptions(corpus: &mut Corpus) {
    let mut interner = StrInterner::default();
    let mut bytes_before_desc: usize = 0;
    let mut unique_descriptions = 0usize;

    for course in &mut corpus.courses {
        bytes_before_desc += course.description.len();
        let new_desc = interner.intern(&course.description, &mut unique_descriptions);
        course.description = new_desc;

        if let Some(level) = course.level.take() {
            course.level = Some(interner.intern(&level, &mut 0));
        }
        if let Some(school) = course.school.take() {
            course.school = Some(interner.intern(&school, &mut 0));
        }
        for tag in &mut course.attribute_tags {
            *tag = interner.intern(tag, &mut 0);
        }
        for tag in &mut course.gened_tags {
            tag.name = interner.intern(&tag.name, &mut 0);
        }
        for skill in &mut course.skills {
            *skill = interner.intern(skill, &mut 0);
        }
        for term in &mut course.has_syllabus_terms {
            *term = interner.intern(term, &mut 0);
        }
    }

    let bytes_after_desc: usize = interner
        .table
        .keys()
        .filter(|k| k.len() > 30)
        .map(|k| k.len())
        .sum();
    tracing::debug!(
        n_courses = corpus.courses.len(),
        unique_descriptions,
        unique_strings = interner.table.len(),
        desc_bytes_before = bytes_before_desc,
        desc_bytes_after = bytes_after_desc,
        desc_savings_percent = (bytes_before_desc.saturating_sub(bytes_after_desc)) as f64
            / bytes_before_desc.max(1) as f64
            * 100.0,
        "string dedup"
    );
}

#[derive(Default)]
struct StrInterner {
    table: HashMap<Arc<str>, Arc<str>>,
}

impl StrInterner {
    fn intern(&mut self, value: &Arc<str>, unique_counter: &mut usize) -> Arc<str> {
        if let Some(existing) = self.table.get(value) {
            return Arc::clone(existing);
        }
        *unique_counter += 1;
        let arc = Arc::clone(value);
        self.table.insert(Arc::clone(&arc), Arc::clone(&arc));
        arc
    }
}

/// Enumerate the per-student subdirectories under `exported_root`.
/// A subdir counts as a student dir if it has `courses_history/` or
/// `programs/` directly underneath. (`syllabi/` lives at `exported_root`,
/// not under a student.)
fn find_student_roots(exported_root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(exported_root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path();
        let has_subtree = ["courses_history", "programs"]
            .iter()
            .any(|sub| path.join(sub).is_dir());
        if has_subtree {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}
