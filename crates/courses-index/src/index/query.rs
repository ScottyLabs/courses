//! Public query type and executor. The hot path lives on
//! [`Searcher`], which carries reusable scratch buffers (a per-doc f32
//! accumulator with epoch-based dirty tracking, plus a `Vec<u32>` of
//! touched doc ids) so successive queries don't re-allocate or zero out
//! 9k+ entries.
//!
//! Score blending follows PLAN.md. Pure browse sorts by PageRank
//! descending, while text queries multiply baked BM25 scores by
//! `1 + alpha * pagerank_normalized` with `alpha = 0.2`.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use roaring::RoaringBitmap;
use serde::{Deserialize, Serialize};

use super::Index;
use crate::doc::CourseId;

const TEXT_PAGERANK_ALPHA: f32 = 0.2;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Query {
    pub text: Option<String>,
    pub fuzzy: bool,
    pub facets: FacetFilters,
    pub numeric: NumericFilters,
    pub sort: SortOrder,
    pub limit: usize,
    pub offset: usize,
    pub count_facets: Vec<FacetAxis>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FacetAxis {
    Dept,
    School,
    Level,
    AttributeTags,
    GenedTags,
    Skills,
    Campuses,
    SemestersOffered,
    ProgramId,
    ProgramType,
    HasSyllabusTerms,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FacetCounts {
    pub dept: Vec<(String, u64)>,
    pub school: Vec<(String, u64)>,
    pub level: Vec<(String, u64)>,
    pub attribute_tags: Vec<(String, u64)>,
    pub gened_tags: Vec<(String, u64)>,
    pub skills: Vec<(String, u64)>,
    pub campuses: Vec<(u32, u64)>,
    pub semesters_offered: Vec<((u16, u8), u64)>,
    pub program_id: Vec<(u32, u64)>,
    pub program_type: Vec<(u8, u64)>,
    pub has_syllabus_terms: Vec<(String, u64)>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FacetFilters {
    pub dept: Vec<String>,
    pub school: Vec<String>,
    pub level: Vec<String>,
    pub attribute_tags: Vec<String>,
    pub gened_tags: Vec<String>,
    pub skills: Vec<String>,
    pub campuses: Vec<u32>,
    pub program_id: Vec<u32>,
    pub program_type: Vec<u8>,
    pub has_syllabus_terms: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NumericFilters {
    pub course_num: Option<(u32, u32)>,
    pub units: Option<(f32, f32)>,
    pub fce_hrs_per_week: Option<(f32, f32)>,
    pub fce_interest: Option<(f32, f32)>,
    pub fce_overall_teaching: Option<(f32, f32)>,
    pub fce_overall_course: Option<(f32, f32)>,
    pub min_year_offered: Option<u16>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum SortOrder {
    #[default]
    Relevance,
    PageRankDesc,
    FceHrsPerWeekAsc,
    FceInterestDesc,
    FceOverallTeachingDesc,
    CourseNumAsc,
    CodeAsc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hit {
    pub course_id: CourseId,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub hits: Vec<Hit>,
    pub total_matched: u64,
    /// Course codes nearby the query string. Populated when the query parses
    /// as a course code and one or more digit-permutations of the trailing
    /// three digits also exist in the catalog.
    pub did_you_mean_codes: Vec<String>,
    pub facet_counts: FacetCounts,
}

/// Default LRU capacity for the per-Searcher result cache. Sized to absorb
/// the typical search-as-you-type prefix sequence (`l`, `li`, `lin`, ...)
/// without evicting recent entries.
pub const DEFAULT_CACHE_CAPACITY: usize = 64;

struct CacheEntry {
    key: Vec<u8>,
    result: QueryResult,
}

/// Reusable per-thread query workspace. Holds the score accumulator with an
/// epoch counter for cheap reset, plus a list of doc ids touched by the
/// current query. The Index is supplied per call so a single Searcher can
/// outlive any one Index reference and live alongside it in a wasm-bindgen
/// struct without lifetime gymnastics.
///
/// A bounded LRU cache of recent (Query, QueryResult) pairs sits in front of
/// the executor, keyed off the bincode-encoded query. A search-as-you-type
/// UI usually issues the same query several times across keystrokes, so the
/// cache turns most of those into a clone instead of a fresh scan. Construct
/// the Searcher with [`Searcher::with_cache_capacity(0)`] to disable it for
/// benchmarks.
pub struct Searcher {
    acc: Vec<f32>,
    epoch: Vec<u32>,
    current: u32,
    touched: Vec<u32>,
    cache: Vec<CacheEntry>,
    cache_capacity: usize,
    cache_buffer: Vec<u8>,
}

impl Searcher {
    pub fn new(n_docs: usize) -> Self {
        Self::with_cache_capacity(n_docs, DEFAULT_CACHE_CAPACITY)
    }

    pub fn with_cache_capacity(n_docs: usize, cache_capacity: usize) -> Self {
        Self {
            acc: vec![0.0; n_docs],
            epoch: vec![0; n_docs],
            current: 0,
            touched: Vec::with_capacity(2048),
            cache: Vec::with_capacity(cache_capacity),
            cache_capacity,
            cache_buffer: Vec::with_capacity(256),
        }
    }

    pub fn query(&mut self, index: &Index, q: &Query) -> QueryResult {
        if self.cache_capacity > 0 {
            self.cache_buffer.clear();
            if bincode::serde::encode_into_std_write(
                q,
                &mut self.cache_buffer,
                bincode::config::standard(),
            )
            .is_ok()
                && let Some(idx) = self.cache.iter().position(|e| e.key == self.cache_buffer)
            {
                let entry = self.cache.remove(idx);
                let result = entry.result.clone();
                self.cache.push(entry);
                return result;
            }
        }

        let result = self.query_uncached(index, q);

        if self.cache_capacity > 0 && !self.cache_buffer.is_empty() {
            let key = std::mem::take(&mut self.cache_buffer);
            self.cache.push(CacheEntry {
                key,
                result: result.clone(),
            });
            if self.cache.len() > self.cache_capacity {
                self.cache.remove(0);
            }
        }

        result
    }

    pub fn query_uncached(&mut self, index: &Index, q: &Query) -> QueryResult {
        let filter_bitmap = self.build_filter_bitmap(index, q);
        let want_counts = !q.count_facets.is_empty();

        let (mut result, result_bitmap): (QueryResult, Option<RoaringBitmap>) =
            if let Some(text) = q.text.as_deref() {
                self.run_text(index, text, q.fuzzy, &filter_bitmap, q, want_counts)
            } else if matches!(q.sort, SortOrder::Relevance | SortOrder::PageRankDesc) {
                self.run_browse_pagerank(index, &filter_bitmap, q, want_counts)
            } else {
                self.run_browse_other(index, &filter_bitmap, q, want_counts)
            };

        if let Some(text) = q.text.as_deref() {
            result.did_you_mean_codes = self.code_swap_suggestions(index, text);
        }
        if let Some(bitmap) = result_bitmap {
            result.facet_counts = self.compute_facet_counts(index, &bitmap, &q.count_facets);
        }
        result
    }

    fn compute_facet_counts(
        &self,
        index: &Index,
        base: &RoaringBitmap,
        axes: &[FacetAxis],
    ) -> FacetCounts {
        let mut out = FacetCounts::default();
        let f = &index.facets;
        for axis in axes {
            match axis {
                FacetAxis::Dept => out.dept = count_str_axis(base, &f.dept),
                FacetAxis::School => out.school = count_str_axis(base, &f.school),
                FacetAxis::Level => out.level = count_str_axis(base, &f.level),
                FacetAxis::AttributeTags => {
                    out.attribute_tags = count_str_axis(base, &f.attribute_tags)
                }
                FacetAxis::GenedTags => out.gened_tags = count_str_axis(base, &f.gened_tags),
                FacetAxis::Skills => out.skills = count_str_axis(base, &f.skills),
                FacetAxis::Campuses => out.campuses = count_int_axis(base, &f.campuses),
                FacetAxis::SemestersOffered => {
                    out.semesters_offered = count_int_axis(base, &f.semesters_offered)
                }
                FacetAxis::ProgramId => out.program_id = count_int_axis(base, &f.program_id),
                FacetAxis::ProgramType => out.program_type = count_int_axis(base, &f.program_type),
                FacetAxis::HasSyllabusTerms => {
                    out.has_syllabus_terms = count_str_axis(base, &f.has_syllabus_terms)
                }
            }
        }
        out
    }

    fn code_swap_suggestions(&self, index: &Index, query: &str) -> Vec<String> {
        let trimmed = query.trim();
        let Some((dept, num)) = trimmed.split_once('-') else {
            return Vec::new();
        };
        if dept.len() != 2
            || num.len() != 3
            || !dept.chars().all(|c| c.is_ascii_digit())
            || !num.chars().all(|c| c.is_ascii_digit())
        {
            return Vec::new();
        }
        let bytes = num.as_bytes();
        let perms = [
            [bytes[0], bytes[1], bytes[2]],
            [bytes[0], bytes[2], bytes[1]],
            [bytes[1], bytes[0], bytes[2]],
            [bytes[1], bytes[2], bytes[0]],
            [bytes[2], bytes[0], bytes[1]],
            [bytes[2], bytes[1], bytes[0]],
        ];
        let mut out: Vec<String> = Vec::new();
        for perm in perms.iter().skip(1) {
            let candidate = format!(
                "{}-{}{}{}",
                dept, perm[0] as char, perm[1] as char, perm[2] as char
            );
            if candidate == trimmed {
                continue;
            }
            if index.code_to_id.contains_key(&candidate) && !out.contains(&candidate) {
                out.push(candidate);
            }
        }
        out
    }

    fn run_text(
        &mut self,
        index: &Index,
        text: &str,
        fuzzy: bool,
        filter: &Option<RoaringBitmap>,
        q: &Query,
        want_counts: bool,
    ) -> (QueryResult, Option<RoaringBitmap>) {
        self.current = self.current.wrapping_add(1);
        if self.current == 0 {
            self.epoch.fill(0);
            self.current = 1;
        }
        self.touched.clear();

        let acc = &mut self.acc;
        let epoch = &mut self.epoch;
        let touched = &mut self.touched;
        let cur = self.current;

        index.text.for_each_exact_posting(text, |doc_id, score| {
            let i = doc_id as usize;
            if epoch[i] != cur {
                epoch[i] = cur;
                acc[i] = 0.0;
                touched.push(doc_id);
            }
            acc[i] += score;
        });

        if fuzzy && touched.is_empty() {
            index.text.for_each_fuzzy_posting(text, |doc_id, score| {
                let i = doc_id as usize;
                if epoch[i] != cur {
                    epoch[i] = cur;
                    acc[i] = 0.0;
                    touched.push(doc_id);
                }
                acc[i] += score;
            });
        }

        let pr = &index.numeric.pagerank_normalized;

        // Fast path: relevance / pagerank-desc sort lets us stream into a
        // bounded heap with an early-skip check. Once the heap is full at
        // K, any doc whose unblended accumulator times the max blend
        // factor (1 + alpha) can't reach the K-th score gets dropped
        // before we pay for the pagerank multiply or the heap push.
        if matches!(q.sort, SortOrder::Relevance | SortOrder::PageRankDesc) {
            let need = q.offset.saturating_add(q.limit.max(1));
            let max_blend_factor = 1.0 + TEXT_PAGERANK_ALPHA;
            let mut heap: BinaryHeap<MinHeapEntry> = BinaryHeap::with_capacity(need + 1);
            let mut threshold: f32 = f32::NEG_INFINITY;
            let mut total_matched: u64 = 0;
            let mut result_bitmap = if want_counts {
                Some(RoaringBitmap::new())
            } else {
                None
            };
            let need_full_heap = need;

            let bm = filter.as_ref();
            for &doc_id in touched.iter() {
                if let Some(bm) = bm
                    && !bm.contains(doc_id)
                {
                    continue;
                }
                total_matched += 1;
                if let Some(rbm) = result_bitmap.as_mut() {
                    rbm.insert(doc_id);
                }
                let i = doc_id as usize;
                let unblended = acc[i];
                if heap.len() >= need_full_heap && unblended * max_blend_factor <= threshold {
                    continue;
                }
                let blended = unblended * (1.0 + TEXT_PAGERANK_ALPHA * pr[i]);
                if heap.len() >= need_full_heap && blended <= threshold {
                    continue;
                }
                heap.push(MinHeapEntry(doc_id, blended));
                if heap.len() > need_full_heap {
                    heap.pop();
                    if let Some(min) = heap.peek() {
                        threshold = min.1;
                    }
                }
            }

            let mut scored: Vec<(u32, f32)> = heap.into_iter().map(|e| (e.0, e.1)).collect();
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
            return (finalize(scored, total_matched, q), result_bitmap);
        }

        // Slow path: non-relevance sorts need every score materialized so
        // the secondary sort can run.
        let mut scored: Vec<(u32, f32)> = Vec::with_capacity(touched.len());
        if let Some(bm) = filter {
            for &doc_id in touched.iter() {
                if !bm.contains(doc_id) {
                    continue;
                }
                let s = acc[doc_id as usize] * (1.0 + TEXT_PAGERANK_ALPHA * pr[doc_id as usize]);
                scored.push((doc_id, s));
            }
        } else {
            for &doc_id in touched.iter() {
                let s = acc[doc_id as usize] * (1.0 + TEXT_PAGERANK_ALPHA * pr[doc_id as usize]);
                scored.push((doc_id, s));
            }
        }

        let total_matched = scored.len() as u64;
        let result_bitmap = if want_counts {
            Some(scored.iter().map(|(d, _)| *d).collect::<RoaringBitmap>())
        } else {
            None
        };
        let scored = self.sort_scored(index, scored, q.sort);
        (finalize(scored, total_matched, q), result_bitmap)
    }

    fn run_browse_pagerank(
        &self,
        index: &Index,
        filter: &Option<RoaringBitmap>,
        q: &Query,
        want_counts: bool,
    ) -> (QueryResult, Option<RoaringBitmap>) {
        let need = q.offset.saturating_add(q.limit.max(1));
        let mut out: Vec<(u32, f32)> = Vec::with_capacity(need);
        match filter {
            Some(bm) => {
                for &(score, doc_id) in index.numeric.pagerank.sorted.iter().rev() {
                    if !bm.contains(doc_id) {
                        continue;
                    }
                    out.push((doc_id, score));
                    if out.len() >= need {
                        break;
                    }
                }
            }
            None => {
                for &(score, doc_id) in index.numeric.pagerank.sorted.iter().rev() {
                    out.push((doc_id, score));
                    if out.len() >= need {
                        break;
                    }
                }
            }
        }
        let total_matched = match filter {
            Some(bm) => bm.len(),
            None => index.n_docs as u64,
        };
        let result_bitmap = if want_counts {
            Some(match filter {
                Some(bm) => bm.clone(),
                None => (0..index.n_docs).collect(),
            })
        } else {
            None
        };
        (finalize(out, total_matched, q), result_bitmap)
    }

    fn run_browse_other(
        &self,
        index: &Index,
        filter: &Option<RoaringBitmap>,
        q: &Query,
        want_counts: bool,
    ) -> (QueryResult, Option<RoaringBitmap>) {
        let mut scored: Vec<(u32, f32)> = match filter {
            Some(bm) => bm.iter().map(|d| (d, 0.0)).collect(),
            None => (0..index.n_docs).map(|d| (d, 0.0)).collect(),
        };
        let total_matched = scored.len() as u64;
        let result_bitmap = if want_counts {
            Some(scored.iter().map(|(d, _)| *d).collect::<RoaringBitmap>())
        } else {
            None
        };
        scored = self.sort_scored(index, scored, q.sort);
        (finalize(scored, total_matched, q), result_bitmap)
    }

    fn sort_scored(
        &self,
        index: &Index,
        mut scored: Vec<(u32, f32)>,
        sort: SortOrder,
    ) -> Vec<(u32, f32)> {
        match sort {
            SortOrder::Relevance | SortOrder::PageRankDesc => {
                scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
            }
            SortOrder::FceHrsPerWeekAsc => {
                sort_by_f32(&mut scored, &index.numeric.fce_hrs_per_week.by_doc, false)
            }
            SortOrder::FceInterestDesc => {
                sort_by_f32(&mut scored, &index.numeric.fce_interest.by_doc, true)
            }
            SortOrder::FceOverallTeachingDesc => sort_by_f32(
                &mut scored,
                &index.numeric.fce_overall_teaching.by_doc,
                true,
            ),
            SortOrder::CourseNumAsc => {
                sort_by_u32(&mut scored, &index.numeric.course_num.by_doc, false)
            }
            SortOrder::CodeAsc => {
                let courses = &index.courses;
                scored.sort_by(|a, b| courses[a.0 as usize].code.cmp(&courses[b.0 as usize].code));
            }
        }
        scored
    }

    fn build_filter_bitmap(&self, index: &Index, q: &Query) -> Option<RoaringBitmap> {
        let mut active: Option<RoaringBitmap> = None;
        let f = &index.facets;

        intersect_str_axis(&mut active, &f.dept, &q.facets.dept);
        intersect_str_axis(&mut active, &f.school, &q.facets.school);
        intersect_str_axis(&mut active, &f.level, &q.facets.level);
        intersect_str_axis(&mut active, &f.attribute_tags, &q.facets.attribute_tags);
        intersect_str_axis(&mut active, &f.gened_tags, &q.facets.gened_tags);
        intersect_str_axis(&mut active, &f.skills, &q.facets.skills);
        intersect_str_axis(
            &mut active,
            &f.has_syllabus_terms,
            &q.facets.has_syllabus_terms,
        );
        intersect_int_axis(&mut active, &f.campuses, &q.facets.campuses);
        intersect_int_axis(&mut active, &f.program_id, &q.facets.program_id);
        intersect_int_axis(&mut active, &f.program_type, &q.facets.program_type);

        if let Some((lo, hi)) = q.numeric.course_num {
            let mut bm = RoaringBitmap::new();
            for &(v, d) in &index.numeric.course_num.sorted {
                if v >= lo && v <= hi {
                    bm.insert(d);
                }
            }
            intersect(&mut active, bm);
        }
        intersect_range_f32(&mut active, &index.numeric.units.sorted, q.numeric.units);
        intersect_range_f32(
            &mut active,
            &index.numeric.fce_hrs_per_week.sorted,
            q.numeric.fce_hrs_per_week,
        );
        intersect_range_f32(
            &mut active,
            &index.numeric.fce_interest.sorted,
            q.numeric.fce_interest,
        );
        intersect_range_f32(
            &mut active,
            &index.numeric.fce_overall_teaching.sorted,
            q.numeric.fce_overall_teaching,
        );
        intersect_range_f32(
            &mut active,
            &index.numeric.fce_overall_course.sorted,
            q.numeric.fce_overall_course,
        );

        if let Some(min_year) = q.numeric.min_year_offered {
            let mut bm = RoaringBitmap::new();
            for &(v, d) in &index.numeric.max_year_offered.sorted {
                if v >= min_year {
                    bm.insert(d);
                }
            }
            intersect(&mut active, bm);
        }

        active
    }
}

/// Convenience wrapper for one-shot queries that don't reuse a Searcher.
impl Index {
    pub fn query(&self, q: &Query) -> QueryResult {
        Searcher::new(self.n_docs as usize).query(self, q)
    }
}

fn finalize(scored: Vec<(u32, f32)>, total_matched: u64, q: &Query) -> QueryResult {
    let hits: Vec<Hit> = scored
        .into_iter()
        .skip(q.offset)
        .take(q.limit.max(1))
        .map(|(d, s)| Hit {
            course_id: d,
            score: s,
        })
        .collect();
    QueryResult {
        hits,
        total_matched,
        did_you_mean_codes: Vec::new(),
        facet_counts: FacetCounts::default(),
    }
}

/// Maximum number of values returned per facet axis. High-cardinality axes
/// (notably ProgramId at ~1500 values) only show a top slice in the UI;
/// capping here lets `count_*_axis` terminate early once the K-th best
/// count beats every remaining axis value's `bitmap.len()` upper bound.
const FACET_TOPK: usize = 100;

fn count_str_axis(base: &RoaringBitmap, axis: &super::facets::StringAxis) -> Vec<(String, u64)> {
    let base_len = base.len();
    let mut heap: BinaryHeap<std::cmp::Reverse<(u64, u32)>> =
        BinaryHeap::with_capacity(FACET_TOPK + 1);
    let mut threshold: u64 = 0;
    for &idx in &axis.sorted_by_size {
        let bm = &axis.bitmaps[idx as usize];
        let max_possible = bm.len().min(base_len);
        if heap.len() >= FACET_TOPK && max_possible <= threshold {
            break;
        }
        let n = base.intersection_len(bm);
        if n == 0 || (heap.len() >= FACET_TOPK && n <= threshold) {
            continue;
        }
        heap.push(std::cmp::Reverse((n, idx)));
        if heap.len() > FACET_TOPK {
            heap.pop();
            if let Some(std::cmp::Reverse((min, _))) = heap.peek() {
                threshold = *min;
            }
        }
    }
    let mut out: Vec<(String, u64)> = heap
        .into_iter()
        .map(|std::cmp::Reverse((n, idx))| (axis.values[idx as usize].clone(), n))
        .collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

fn count_int_axis<T: Copy + Eq + std::hash::Hash + Ord>(
    base: &RoaringBitmap,
    axis: &super::facets::IntAxis<T>,
) -> Vec<(T, u64)> {
    let base_len = base.len();
    let mut heap: BinaryHeap<std::cmp::Reverse<(u64, u32)>> =
        BinaryHeap::with_capacity(FACET_TOPK + 1);
    let mut threshold: u64 = 0;
    for &idx in &axis.sorted_by_size {
        let bm = &axis.bitmaps[idx as usize];
        let max_possible = bm.len().min(base_len);
        if heap.len() >= FACET_TOPK && max_possible <= threshold {
            break;
        }
        let n = base.intersection_len(bm);
        if n == 0 || (heap.len() >= FACET_TOPK && n <= threshold) {
            continue;
        }
        heap.push(std::cmp::Reverse((n, idx)));
        if heap.len() > FACET_TOPK {
            heap.pop();
            if let Some(std::cmp::Reverse((min, _))) = heap.peek() {
                threshold = *min;
            }
        }
    }
    let mut out: Vec<(T, u64)> = heap
        .into_iter()
        .map(|std::cmp::Reverse((n, idx))| (axis.values[idx as usize], n))
        .collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

fn intersect_str_axis(
    active: &mut Option<RoaringBitmap>,
    axis: &super::facets::StringAxis,
    values: &[String],
) {
    if values.is_empty() {
        return;
    }
    let mut union = RoaringBitmap::new();
    for v in values {
        if let Some(bm) = axis.lookup(v) {
            union |= bm;
        }
    }
    intersect(active, union);
}

fn intersect_int_axis<T: Copy + Eq + std::hash::Hash + Ord>(
    active: &mut Option<RoaringBitmap>,
    axis: &super::facets::IntAxis<T>,
    values: &[T],
) {
    if values.is_empty() {
        return;
    }
    let mut union = RoaringBitmap::new();
    for &v in values {
        if let Some(bm) = axis.lookup(v) {
            union |= bm;
        }
    }
    intersect(active, union);
}

fn intersect(active: &mut Option<RoaringBitmap>, new_bm: RoaringBitmap) {
    *active = Some(match active.take() {
        Some(curr) => curr & new_bm,
        None => new_bm,
    });
}

fn intersect_range_f32(
    active: &mut Option<RoaringBitmap>,
    sorted: &[(f32, u32)],
    range: Option<(f32, f32)>,
) {
    if let Some((lo, hi)) = range {
        let mut bm = RoaringBitmap::new();
        for &(v, d) in sorted {
            if v >= lo && v <= hi {
                bm.insert(d);
            }
        }
        intersect(active, bm);
    }
}

fn sort_by_f32(items: &mut [(u32, f32)], by_doc: &[f32], desc: bool) {
    items.sort_by(|a, b| {
        let xa = by_doc.get(a.0 as usize).copied().unwrap_or(f32::NAN);
        let xb = by_doc.get(b.0 as usize).copied().unwrap_or(f32::NAN);
        match (xa.is_nan(), xb.is_nan()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => {
                let cmp = xa.partial_cmp(&xb).unwrap_or(Ordering::Equal);
                if desc { cmp.reverse() } else { cmp }
            }
        }
    });
}

fn sort_by_u32(items: &mut [(u32, f32)], by_doc: &[u32], desc: bool) {
    items.sort_by(|a, b| {
        let xa = by_doc.get(a.0 as usize).copied().unwrap_or(0);
        let xb = by_doc.get(b.0 as usize).copied().unwrap_or(0);
        let cmp = xa.cmp(&xb);
        if desc { cmp.reverse() } else { cmp }
    });
}

struct MinHeapEntry(u32, f32);

impl PartialEq for MinHeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl Eq for MinHeapEntry {}

impl PartialOrd for MinHeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MinHeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse so BinaryHeap acts as a min-heap on score.
        other
            .1
            .partial_cmp(&self.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| self.0.cmp(&other.0))
    }
}
