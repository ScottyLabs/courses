//! Numeric range filters and sort orders. f32 fields use `NaN` as the
//! "missing" sentinel so the per-doc lookup is a flat `Vec<f32>` without
//! `Option` overhead. Range filters and ordered scans both fall out of the
//! sorted `(value, doc_id)` companion vector.

use crate::doc::Course;

/// Sentinel value for "this doc doesn't carry this f32 field." NaN compares
/// false in every direction, so range queries naturally exclude it.
pub const F32_MISSING: f32 = f32::NAN;

pub struct NumericFieldF32 {
    pub sorted: Vec<(f32, u32)>,
    pub by_doc: Vec<f32>,
}

impl NumericFieldF32 {
    pub fn build<F>(courses: &[Course], mut value_for: F) -> Self
    where
        F: FnMut(&Course) -> Option<f32>,
    {
        let mut by_doc: Vec<f32> = vec![F32_MISSING; courses.len()];
        let mut sorted: Vec<(f32, u32)> = Vec::with_capacity(courses.len());
        for course in courses {
            if let Some(v) = value_for(course)
                && !v.is_nan()
            {
                by_doc[course.id as usize] = v;
                sorted.push((v, course.id));
            }
        }
        sorted.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.1.cmp(&b.1))
        });
        NumericFieldF32 { sorted, by_doc }
    }

    pub fn get(&self, doc_id: u32) -> f32 {
        self.by_doc
            .get(doc_id as usize)
            .copied()
            .unwrap_or(F32_MISSING)
    }
}

pub struct NumericFieldU32 {
    pub sorted: Vec<(u32, u32)>,
    pub by_doc: Vec<u32>,
}

impl NumericFieldU32 {
    pub fn build<F>(courses: &[Course], mut value_for: F) -> Self
    where
        F: FnMut(&Course) -> u32,
    {
        let mut by_doc: Vec<u32> = vec![0; courses.len()];
        let mut sorted: Vec<(u32, u32)> = Vec::with_capacity(courses.len());
        for course in courses {
            let v = value_for(course);
            by_doc[course.id as usize] = v;
            sorted.push((v, course.id));
        }
        sorted.sort();
        NumericFieldU32 { sorted, by_doc }
    }

    pub fn get(&self, doc_id: u32) -> u32 {
        self.by_doc.get(doc_id as usize).copied().unwrap_or(0)
    }
}

pub struct NumericFieldU16Optional {
    pub sorted: Vec<(u16, u32)>,
    /// Sentinel value `0` means missing. Year 0 is impossible for course
    /// catalogs, so this is unambiguous.
    pub by_doc: Vec<u16>,
}

impl NumericFieldU16Optional {
    pub fn build<F>(courses: &[Course], mut value_for: F) -> Self
    where
        F: FnMut(&Course) -> Option<u16>,
    {
        let mut by_doc: Vec<u16> = vec![0; courses.len()];
        let mut sorted: Vec<(u16, u32)> = Vec::with_capacity(courses.len());
        for course in courses {
            if let Some(v) = value_for(course) {
                by_doc[course.id as usize] = v;
                sorted.push((v, course.id));
            }
        }
        sorted.sort();
        NumericFieldU16Optional { sorted, by_doc }
    }

    pub fn get(&self, doc_id: u32) -> Option<u16> {
        match self.by_doc.get(doc_id as usize).copied() {
            Some(0) | None => None,
            Some(v) => Some(v),
        }
    }
}

pub struct NumericIndex {
    pub course_num: NumericFieldU32,
    pub units: NumericFieldF32,
    pub fce_hrs_per_week: NumericFieldF32,
    pub fce_interest: NumericFieldF32,
    pub fce_overall_teaching: NumericFieldF32,
    pub fce_overall_course: NumericFieldF32,
    pub pagerank: NumericFieldF32,
    pub max_year_offered: NumericFieldU16Optional,
    /// Min-max scaled pagerank, baked at build time for cheap score blending.
    pub pagerank_normalized: Vec<f32>,
}

impl NumericIndex {
    pub fn build(courses: &[Course]) -> Self {
        let pagerank = NumericFieldF32::build(courses, |c| Some(c.pagerank));
        let pagerank_normalized = compute_pagerank_normalized(&pagerank.by_doc);
        NumericIndex {
            course_num: NumericFieldU32::build(courses, |c| c.course_num),
            units: NumericFieldF32::build(courses, |c| Some(c.units)),
            fce_hrs_per_week: NumericFieldF32::build(courses, |c| {
                c.fce_aggregates.map(|a| a.hrs_per_week)
            }),
            fce_interest: NumericFieldF32::build(courses, |c| c.fce_aggregates.map(|a| a.interest)),
            fce_overall_teaching: NumericFieldF32::build(courses, |c| {
                c.fce_aggregates.map(|a| a.overall_teaching)
            }),
            fce_overall_course: NumericFieldF32::build(courses, |c| {
                c.fce_aggregates.map(|a| a.overall_course)
            }),
            pagerank,
            max_year_offered: NumericFieldU16Optional::build(courses, |c| {
                c.semesters_offered.iter().map(|s| s.year).max()
            }),
            pagerank_normalized,
        }
    }
}

fn compute_pagerank_normalized(by_doc: &[f32]) -> Vec<f32> {
    let max = by_doc
        .iter()
        .copied()
        .filter(|v| !v.is_nan())
        .fold(0.0f32, f32::max);
    if max <= 0.0 {
        return vec![0.0; by_doc.len()];
    }
    by_doc
        .iter()
        .map(|&v| if v.is_nan() { 0.0 } else { v / max })
        .collect()
}
