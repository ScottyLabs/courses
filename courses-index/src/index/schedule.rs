//! Schedule queries over [`SectionTime`]. Two access patterns matter for
//! the UI. One fits a section into an explicit `(term, days, time-window)`
//! constraint, and the other tests whether any pair of sections from two
//! courses collide.
//!
//! The owned section vec is kept sorted by `(term, start_minutes)` so the
//! per-term slice can be located via a `(start, end)` index map and scanned
//! in cache-order.

use std::collections::HashMap;

use crate::doc::{CourseId, SectionId, SectionTime, Term};

pub struct ScheduleIndex {
    pub sections: Vec<SectionTime>,
    by_term: HashMap<Term, (usize, usize)>,
    by_course: HashMap<CourseId, Vec<u32>>,
}

impl ScheduleIndex {
    pub fn build(mut sections: Vec<SectionTime>) -> Self {
        sections.sort_by(|a, b| {
            (a.term.year, a.term.sem as u8, a.start_minutes, a.section_id).cmp(&(
                b.term.year,
                b.term.sem as u8,
                b.start_minutes,
                b.section_id,
            ))
        });

        let mut by_term: HashMap<Term, (usize, usize)> = HashMap::new();
        let mut by_course: HashMap<CourseId, Vec<u32>> = HashMap::new();
        let mut current: Option<Term> = None;
        let mut start_idx = 0usize;
        for (i, s) in sections.iter().enumerate() {
            if Some(s.term) != current {
                if let Some(prev) = current {
                    by_term.insert(prev, (start_idx, i));
                }
                current = Some(s.term);
                start_idx = i;
            }
            by_course.entry(s.course_id).or_default().push(i as u32);
        }
        if let Some(prev) = current {
            by_term.insert(prev, (start_idx, sections.len()));
        }

        ScheduleIndex {
            sections,
            by_term,
            by_course,
        }
    }

    /// Section ids whose meeting time is fully contained inside the supplied
    /// window. `days_mask` is the same bitmask shape as
    /// [`SectionTime::days`]. Async sections (`days == 0`) are excluded
    /// because they have no time to fit.
    pub fn schedule_fit(
        &self,
        term: Term,
        days_mask: u8,
        start_min: u16,
        end_min: u16,
    ) -> Vec<SectionId> {
        let Some(&(lo, hi)) = self.by_term.get(&term) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for s in &self.sections[lo..hi] {
            if s.days == 0 {
                continue;
            }
            if (s.days & !days_mask) != 0 {
                continue;
            }
            if s.start_minutes < start_min || s.end_minutes > end_min {
                continue;
            }
            out.push(s.section_id);
        }
        out
    }

    /// True if any pair of sections from `a` and `b` clashes (same term,
    /// shared weekday, overlapping minutes).
    pub fn courses_overlap(&self, a: CourseId, b: CourseId) -> bool {
        let Some(idx_a) = self.by_course.get(&a) else {
            return false;
        };
        let Some(idx_b) = self.by_course.get(&b) else {
            return false;
        };
        for &i in idx_a {
            let sa = &self.sections[i as usize];
            if sa.days == 0 {
                continue;
            }
            for &j in idx_b {
                let sb = &self.sections[j as usize];
                if sa.term != sb.term || sb.days == 0 {
                    continue;
                }
                if (sa.days & sb.days) == 0 {
                    continue;
                }
                if sa.start_minutes < sb.end_minutes && sb.start_minutes < sa.end_minutes {
                    return true;
                }
            }
        }
        false
    }

    pub fn sections_for_course(&self, course_id: CourseId) -> &[u32] {
        self.by_course
            .get(&course_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}
