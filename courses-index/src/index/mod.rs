//! In-memory query index over the loaded `Corpus`. Splits cleanly into a
//! per-field text index ([`text::TextIndex`]), categorical filter bitmaps
//! ([`facets::FacetIndex`]), numeric range/sort indexes
//! ([`numeric::NumericIndex`]), a schedule index over `SectionTime`, and a
//! query executor that combines them. The whole index is built from the
//! in-memory corpus produced by [`crate::load::run`] or by reading a
//! catalog file via [`crate::binary`].

pub mod facets;
pub mod numeric;
pub mod query;
pub mod schedule;
pub mod text;
pub mod tokenize;

pub use facets::FacetIndex;
pub use numeric::NumericIndex;
pub use query::{
    FacetAxis, FacetCounts, FacetFilters, Hit, NumericFilters, Query, QueryResult, Searcher,
    SortOrder,
};
pub use schedule::ScheduleIndex;
pub use text::{FieldWeights, PrebuiltField, PrebuiltText, TextIndex};

use std::collections::HashMap;

use crate::doc::{Course, CourseId, FceRow, Professor};
use crate::load::Corpus;

pub struct Index {
    pub courses: Vec<Course>,
    pub professors: Vec<Professor>,
    pub fce_rows: Vec<FceRow>,
    pub text: TextIndex,
    pub facets: FacetIndex,
    pub numeric: NumericIndex,
    pub schedule: ScheduleIndex,
    pub n_docs: u32,
    pub code_to_id: HashMap<String, CourseId>,
}

impl Index {
    pub fn build(corpus: Corpus) -> Self {
        Self::build_inner(corpus, None).expect("text index build is infallible from courses")
    }

    /// Build using a pre-baked text index (FST bytes + arena), skipping the
    /// most expensive build phase. Errors if the supplied bytes don't decode
    /// as a valid FST.
    pub fn build_with_prebuilt_text(
        corpus: Corpus,
        prebuilt: PrebuiltText,
    ) -> anyhow::Result<Self> {
        Self::build_inner(corpus, Some(prebuilt))
    }

    fn build_inner(corpus: Corpus, prebuilt: Option<PrebuiltText>) -> anyhow::Result<Self> {
        let Corpus {
            courses,
            professors,
            sections,
            fce_rows,
        } = corpus;

        let n_docs = courses.len() as u32;
        let text = match prebuilt {
            Some(p) => TextIndex::from_prebuilt(n_docs, p)?,
            None => TextIndex::build(&courses),
        };
        let facets = FacetIndex::build(&courses);
        let numeric = NumericIndex::build(&courses);
        let schedule = ScheduleIndex::build(sections);
        let code_to_id: HashMap<String, CourseId> =
            courses.iter().map(|c| (c.code.clone(), c.id)).collect();
        Ok(Index {
            courses,
            professors,
            fce_rows,
            text,
            facets,
            numeric,
            schedule,
            n_docs,
            code_to_id,
        })
    }
}
