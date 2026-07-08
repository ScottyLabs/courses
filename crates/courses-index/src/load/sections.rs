//! Walk `ly*_sm*.json` files to emit `SectionTime` rows and enrich the
//! courses produced by [`super::courses`]. Each file carries one course's
//! sections for a given (lyear, sem); the authoritative term comes from each
//! section's `start_date`, which is unambiguous across anchor shifts. Months
//! 1-5 map to Spring, 6-7 to Summer, 8-12 to Fall.
//!
//! The walk populates three outputs together:
//!
//! 1. `Vec<SectionTime>` for schedule-fit and overlap queries.
//! 2. A richer instructor set per course (section files reference ~3,221
//!    distinct instructor ids vs ~1,866 in `info.json`'s `instructors`).
//! 3. Extra (year, sem, campus) tuples added to each course's
//!    `semesters_offered`.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use tracing::{debug, warn};
use walkdir::WalkDir;

use crate::doc::{
    Course, CourseId, InstructorId, InstructorRef, SectionId, SectionTime, SectionType, Sem,
    SemesterOffered, Term,
};

#[derive(Debug, Deserialize)]
struct Sections {
    #[serde(default)]
    data_list: Vec<SectionsItem>,
}

#[derive(Debug, Deserialize)]
struct SectionsItem {
    #[serde(default)]
    course: CourseRef,
    #[serde(default)]
    section_groups: Vec<SectionGroup>,
}

#[derive(Debug, Default, Deserialize)]
struct CourseRef {
    #[serde(default)]
    code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SectionGroup {
    #[serde(default)]
    group_name: String,
    #[serde(default)]
    sections: Vec<RawSection>,
}

#[derive(Debug, Deserialize)]
struct RawSection {
    id: SectionId,
    #[serde(default)]
    start_date: Option<String>,
    #[serde(default)]
    campus: Option<u32>,
    #[serde(default)]
    instructors: Vec<RawInstructor>,
    #[serde(default)]
    timings: Vec<RawTiming>,
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
}

#[derive(Debug, Deserialize)]
struct RawTiming {
    #[serde(default)]
    time: Option<RawTimeWindow>,
    #[serde(default)]
    days: Vec<String>,
    #[serde(default)]
    building: Option<String>,
    #[serde(default)]
    room: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawTimeWindow {
    #[serde(default)]
    start: Option<String>,
    #[serde(default)]
    end: Option<String>,
}

/// Output of the sections phase.
#[derive(Debug, Default)]
pub struct SectionLoad {
    pub sections: Vec<SectionTime>,
    pub files_scanned: usize,
    pub files_skipped: usize,
    pub orphaned_course_codes: HashSet<String>,
}

/// Walk every `ly*_sm*.json` under `courses_history_root`, emit `SectionTime`
/// rows, and enrich each course in `courses` with section-level instructors
/// and additional `semesters_offered` tuples. Mutates `courses` in place.
pub fn load_sections(courses_history_root: &Path, courses: &mut [Course]) -> Result<SectionLoad> {
    let code_to_id: HashMap<&str, CourseId> =
        courses.iter().map(|c| (c.code.as_str(), c.id)).collect();

    let files: Vec<_> = WalkDir::new(courses_history_root)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.file_name()
                    .to_str()
                    .map(is_sections_filename)
                    .unwrap_or(false)
        })
        .collect();

    let parsed: Vec<ParsedFile> = files
        .par_iter()
        .filter_map(|e| parse_file(e.path(), &code_to_id))
        .collect();

    let mut out = SectionLoad {
        files_scanned: files.len(),
        files_skipped: files.len() - parsed.len(),
        ..Default::default()
    };

    // Per-course accumulators, keyed by id.
    let mut extra_instructors: HashMap<CourseId, HashMap<InstructorId, InstructorRef>> =
        HashMap::new();
    let mut extra_semesters: HashMap<CourseId, HashSet<SemesterOffered>> = HashMap::new();

    for file in parsed {
        let ParsedFile {
            sections,
            instructors_by_course,
            orphaned_course_codes,
            campus_by_section,
        } = file;
        for section in sections {
            let course_id = section.course_id;
            let campus = campus_by_section
                .get(&section.section_id)
                .copied()
                .unwrap_or(0);
            extra_semesters
                .entry(course_id)
                .or_default()
                .insert(SemesterOffered {
                    year: section.term.year,
                    sem: section.term.sem,
                    campus_id: campus,
                });
            out.sections.push(section);
        }
        for (course_id, instructors) in instructors_by_course {
            let entry = extra_instructors.entry(course_id).or_default();
            for instr in instructors {
                entry
                    .entry(instr.id)
                    .and_modify(|existing: &mut InstructorRef| {
                        for sem in &instr.semesters_taught {
                            if !existing.semesters_taught.contains(sem) {
                                existing.semesters_taught.push(*sem);
                            }
                        }
                    })
                    .or_insert(instr);
            }
        }
        for code in orphaned_course_codes {
            out.orphaned_course_codes.insert(code);
        }
    }

    for course in courses.iter_mut() {
        if let Some(extras) = extra_instructors.remove(&course.id) {
            merge_instructors(&mut course.instructors_recent, extras);
        }
        if let Some(extras) = extra_semesters.remove(&course.id) {
            for sem in extras {
                if !course.semesters_offered.contains(&sem) {
                    course.semesters_offered.push(sem);
                }
            }
            course.semesters_offered.sort_by(|a, b| {
                (b.year, a.sem as u8, a.campus_id).cmp(&(a.year, b.sem as u8, b.campus_id))
            });
        }
    }

    debug!(
        scanned = out.files_scanned,
        skipped = out.files_skipped,
        sections = out.sections.len(),
        orphans = out.orphaned_course_codes.len(),
        "sections load complete"
    );
    Ok(out)
}

fn is_sections_filename(name: &str) -> bool {
    let rest = match name.strip_prefix("ly") {
        Some(r) => r,
        None => return false,
    };
    let (lyear, rest) = match rest.split_once("_sm") {
        Some(x) => x,
        None => return false,
    };
    let (sem, ext) = match rest.split_once('.') {
        Some(x) => x,
        None => return false,
    };
    ext == "json" && lyear.parse::<u8>().is_ok() && sem.parse::<u8>().is_ok()
}

struct ParsedFile {
    sections: Vec<SectionTime>,
    instructors_by_course: HashMap<CourseId, Vec<InstructorRef>>,
    orphaned_course_codes: HashSet<String>,
    campus_by_section: HashMap<SectionId, u32>,
}

fn parse_file(path: &Path, code_to_id: &HashMap<&str, CourseId>) -> Option<ParsedFile> {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(err) => {
            warn!(?err, path = %path.display(), "sections read failed");
            return None;
        }
    };
    let parsed: Sections = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(err) => {
            warn!(?err, path = %path.display(), "sections parse failed");
            return None;
        }
    };

    let mut out = ParsedFile {
        sections: Vec::new(),
        instructors_by_course: HashMap::new(),
        orphaned_course_codes: HashSet::new(),
        campus_by_section: HashMap::new(),
    };

    for item in parsed.data_list {
        let code = match item.course.code.as_deref().filter(|s| !s.is_empty()) {
            Some(c) => c.to_string(),
            None => continue,
        };
        let Some(&course_id) = code_to_id.get(code.as_str()) else {
            out.orphaned_course_codes.insert(code);
            continue;
        };
        for group in item.section_groups {
            let section_type = SectionType::from_label(&group.group_name);
            for raw in group.sections {
                let Some(term) = raw
                    .start_date
                    .as_deref()
                    .and_then(parse_term_from_start_date)
                else {
                    continue;
                };
                out.campus_by_section
                    .insert(raw.id, raw.campus.unwrap_or(0));

                let instructor_refs: Vec<InstructorRef> = raw
                    .instructors
                    .iter()
                    .map(|i| InstructorRef {
                        id: i.id,
                        name: format_name(i.first_name.as_deref(), i.last_name.as_deref()),
                        username: i.username.clone(),
                        semesters_taught: vec![(term.year, term.sem)],
                    })
                    .collect();
                if !instructor_refs.is_empty() {
                    out.instructors_by_course
                        .entry(course_id)
                        .or_default()
                        .extend(instructor_refs.iter().cloned());
                }

                let instructor_ids = instructor_refs.into_iter().map(|i| i.id).collect();

                let (days, start_minutes, end_minutes, building, room) = match raw.timings.first() {
                    Some(t) => {
                        let days_mask = days_bitmask(&t.days);
                        let (sm, em) = t.time.as_ref().map(timing_minutes).unwrap_or((0, 0));
                        (days_mask, sm, em, t.building.clone(), t.room.clone())
                    }
                    None => (0u8, 0, 0, None, None),
                };

                out.sections.push(SectionTime {
                    section_id: raw.id,
                    course_id,
                    term,
                    days,
                    start_minutes,
                    end_minutes,
                    building,
                    room,
                    section_type,
                    instructors: instructor_ids,
                });
            }
        }
    }

    Some(out)
}

fn parse_term_from_start_date(s: &str) -> Option<Term> {
    let mut parts = s.splitn(3, '-');
    let year: u16 = parts.next()?.parse().ok()?;
    let month: u8 = parts.next()?.parse().ok()?;
    let sem = match month {
        1..=5 => Sem::Spring,
        6..=7 => Sem::Summer,
        8..=12 => Sem::Fall,
        _ => return None,
    };
    Some(Term { year, sem })
}

fn timing_minutes(window: &RawTimeWindow) -> (u16, u16) {
    let start = window
        .start
        .as_deref()
        .and_then(parse_clock_minutes)
        .unwrap_or(0);
    let end = window
        .end
        .as_deref()
        .and_then(parse_clock_minutes)
        .unwrap_or(0);
    (start, end)
}

/// Parse `08:35AM` or `12:20PM` into minutes from midnight.
fn parse_clock_minutes(s: &str) -> Option<u16> {
    let s = s.trim();
    if s.len() < 4 {
        return None;
    }
    let (time, meridiem) = if let Some(core) = s.strip_suffix("AM").or_else(|| s.strip_suffix("am"))
    {
        (core.trim(), false)
    } else {
        let core = s.strip_suffix("PM").or_else(|| s.strip_suffix("pm"))?;
        (core.trim(), true)
    };
    let (hh, mm) = time.split_once(':')?;
    let hour: u16 = hh.parse().ok()?;
    let minute: u16 = mm.parse().ok()?;
    let hour24 = match (hour, meridiem) {
        (12, false) => 0,
        (h, false) => h,
        (12, true) => 12,
        (h, true) => h + 12,
    };
    Some(hour24 * 60 + minute)
}

fn days_bitmask(days: &[String]) -> u8 {
    let mut mask = 0u8;
    for d in days {
        let bit = match d.as_str() {
            "monday" => 0,
            "tuesday" => 1,
            "wednesday" => 2,
            "thursday" => 3,
            "friday" => 4,
            "saturday" => 5,
            "sunday" => 6,
            _ => continue,
        };
        mask |= 1 << bit;
    }
    mask
}

fn format_name(first: Option<&str>, last: Option<&str>) -> String {
    match (first, last) {
        (Some(f), Some(l)) => format!("{f} {l}"),
        (Some(f), None) => f.to_string(),
        (None, Some(l)) => l.to_string(),
        (None, None) => String::new(),
    }
}

fn merge_instructors(
    existing: &mut Vec<InstructorRef>,
    extras: HashMap<InstructorId, InstructorRef>,
) {
    let mut by_id: HashMap<InstructorId, InstructorRef> =
        existing.drain(..).map(|i| (i.id, i)).collect();
    for (id, extra) in extras {
        by_id
            .entry(id)
            .and_modify(|curr| {
                for sem in &extra.semesters_taught {
                    if !curr.semesters_taught.contains(sem) {
                        curr.semesters_taught.push(*sem);
                    }
                }
                if curr.username.is_none() {
                    curr.username.clone_from(&extra.username);
                }
                if curr.name.is_empty() {
                    curr.name.clone_from(&extra.name);
                }
            })
            .or_insert(extra);
    }
    let mut merged: Vec<InstructorRef> = by_id.into_values().collect();
    for refn in merged.iter_mut() {
        refn.semesters_taught
            .sort_by(|a, b| (b.0, a.1 as u8).cmp(&(a.0, b.1 as u8)));
        refn.semesters_taught.dedup();
    }
    merged.sort_by_key(|a| a.id);
    *existing = merged;
}
