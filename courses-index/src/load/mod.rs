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

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
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

/// Run every loader pass against `exported_root` and `fce_csv`, returning the
/// fully joined corpus.
pub fn run(exported_root: &Path, fce_csv: &Path) -> Result<Corpus> {
    let courses_root = exported_root.join("courses_history");
    let programs_root = exported_root.join("programs");
    let syllabi_root = exported_root.join("syllabi");
    let CourseLoad { mut courses, .. } = load_courses(&courses_root)?;
    let SectionLoad { sections, .. } = load_sections(&courses_root, &mut courses)?;
    let FceLoad {
        rows: mut fce_rows, ..
    } = load_fces(fce_csv, &mut courses)?;
    let _ = load_programs(&programs_root, &mut courses)?;
    let _ = load_syllabi(&syllabi_root, &mut courses)?;
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
