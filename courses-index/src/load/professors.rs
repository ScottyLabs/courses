//! Build the [`Professor`] corpus by joining the Stellic instructor refs
//! attached to each course with the FCE survey rows. Matching keys both
//! sides into a `LASTNAME, F` shape (last name plus first-initial,
//! uppercased), buckets all Stellic refs by that key, then looks up each
//! FCE row's instructor string in the same map. Rows whose key resolves
//! to exactly one Stellic id get `instructor_id` filled in; ambiguous
//! keys with multiple Stellic candidates and fully-unmatched keys keep
//! `instructor_id = None`.
//!
//! Stellic-only and FCE-only instructors both produce `Professor` entries;
//! the only difference is whether `stellic_id` and/or `fce_key` are
//! populated. Synthetic ids for FCE-only professors start above the highest
//! Stellic id so the existing `SectionTime.instructors` and
//! `InstructorRef.id` references stay valid.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::doc::{
    Course, CourseTaught, FceAggregate, FceRow, InstructorId, Professor, SectionTime, Sem,
};

#[derive(Debug, Default)]
pub struct ProfessorBuild {
    pub professors: Vec<Professor>,
    pub fce_rows_matched: usize,
    pub fce_rows_ambiguous: usize,
    pub fce_rows_unmatched: usize,
    pub fce_only_professors: usize,
}

pub fn build_professors(
    courses: &[Course],
    sections: &[SectionTime],
    fce_rows: &mut [FceRow],
) -> ProfessorBuild {
    let mut stellic: BTreeMap<InstructorId, StellicBuilder> = BTreeMap::new();

    for course in courses {
        for instr in &course.instructors_recent {
            let entry = stellic
                .entry(instr.id)
                .or_insert_with(|| StellicBuilder::new(instr.id));
            entry.observe_course(course, instr.name.as_str(), instr.username.as_deref());
            for &(year, sem) in &instr.semesters_taught {
                entry.semesters.insert((year, sem));
                entry
                    .courses_semesters
                    .entry(course.code.clone())
                    .or_default()
                    .insert((year, sem));
            }
        }
    }

    let mut by_fce_key: HashMap<String, Vec<InstructorId>> = HashMap::new();
    for builder in stellic.values() {
        if let Some(key) = first_initial_key(&builder.name) {
            by_fce_key.entry(key).or_default().push(builder.stellic_id);
        }
    }

    let mut out = ProfessorBuild::default();
    let mut fce_only_keys: HashMap<String, FceOnlyBuilder> = HashMap::new();

    for row in fce_rows.iter_mut() {
        let key = match first_initial_key(&row.instructor_fce_key) {
            Some(k) => k,
            None => {
                out.fce_rows_unmatched += 1;
                continue;
            }
        };
        match by_fce_key.get(&key) {
            Some(ids) if ids.len() == 1 => {
                let id = ids[0];
                row.instructor_id = Some(id);
                out.fce_rows_matched += 1;
                if let Some(builder) = stellic.get_mut(&id) {
                    builder.fce_key.get_or_insert_with(|| key.clone());
                }
            }
            Some(_) => {
                out.fce_rows_ambiguous += 1;
            }
            None => {
                out.fce_rows_unmatched += 1;
                fce_only_keys
                    .entry(key)
                    .or_insert_with(FceOnlyBuilder::new)
                    .observe(row);
            }
        }
    }

    let mut fce_rows_by_instr: HashMap<InstructorId, Vec<&FceRow>> = HashMap::new();
    for row in fce_rows.iter() {
        if let Some(id) = row.instructor_id {
            fce_rows_by_instr.entry(id).or_default().push(row);
        }
    }

    let mut sections_taught: HashMap<InstructorId, u32> = HashMap::new();
    for section in sections {
        for &id in &section.instructors {
            *sections_taught.entry(id).or_insert(0) += 1;
        }
    }

    let mut professors: Vec<Professor> = Vec::with_capacity(stellic.len() + fce_only_keys.len());

    for builder in stellic.values() {
        let courses_taught = builder.assemble_courses_taught();
        let depts = builder.depts();
        let fce_aggregates = fce_rows_by_instr
            .get(&builder.stellic_id)
            .and_then(|rows| aggregate_all(rows));
        let n_sections = sections_taught
            .get(&builder.stellic_id)
            .copied()
            .unwrap_or(0);

        professors.push(Professor {
            id: builder.stellic_id,
            stellic_id: Some(builder.stellic_id),
            fce_key: builder.fce_key.clone(),
            name: builder.name.clone(),
            username: builder.username.clone(),
            depts,
            courses_taught,
            fce_aggregates,
            n_sections_taught: n_sections,
        });
    }

    let next_id_start = stellic.keys().copied().max().unwrap_or(0).saturating_add(1);
    let mut fce_only_keys: Vec<(String, FceOnlyBuilder)> = fce_only_keys.into_iter().collect();
    fce_only_keys.sort_by(|a, b| a.0.cmp(&b.0));
    let key_to_id: HashMap<String, InstructorId> = fce_only_keys
        .iter()
        .enumerate()
        .map(|(i, (k, _))| (k.clone(), next_id_start + i as InstructorId))
        .collect();

    let mut rows_by_id: HashMap<InstructorId, Vec<usize>> = HashMap::new();
    for (idx, row) in fce_rows.iter_mut().enumerate() {
        if row.instructor_id.is_none() {
            if let Some(key) = first_initial_key(&row.instructor_fce_key) {
                if let Some(&id) = key_to_id.get(&key) {
                    row.instructor_id = Some(id);
                    rows_by_id.entry(id).or_default().push(idx);
                }
            }
        }
    }

    for (key, builder) in fce_only_keys {
        let id = key_to_id[&key];
        let row_idxs = rows_by_id.remove(&id).unwrap_or_default();
        let attached: Vec<&FceRow> = row_idxs.iter().map(|&i| &fce_rows[i]).collect();
        let fce_aggregates = aggregate_all(&attached);
        let courses_taught = builder.assemble_courses_taught(&attached);
        let depts = builder.depts(&attached);

        professors.push(Professor {
            id,
            stellic_id: None,
            fce_key: Some(key.clone()),
            name: builder.display_name.unwrap_or_else(|| key.clone()),
            username: None,
            depts,
            courses_taught,
            fce_aggregates,
            n_sections_taught: 0,
        });
        out.fce_only_professors += 1;
    }

    professors.sort_by_key(|p| p.id);
    out.professors = professors;
    out
}

struct StellicBuilder {
    stellic_id: InstructorId,
    name: String,
    username: Option<String>,
    fce_key: Option<String>,
    semesters: HashSet<(u16, Sem)>,
    courses_semesters: HashMap<String, HashSet<(u16, Sem)>>,
    course_depts: HashSet<String>,
}

impl StellicBuilder {
    fn new(stellic_id: InstructorId) -> Self {
        Self {
            stellic_id,
            name: String::new(),
            username: None,
            fce_key: None,
            semesters: HashSet::new(),
            courses_semesters: HashMap::new(),
            course_depts: HashSet::new(),
        }
    }

    fn observe_course(&mut self, course: &Course, name: &str, username: Option<&str>) {
        if self.name.is_empty() && !name.is_empty() {
            self.name = name.to_string();
        }
        if self.username.is_none() {
            self.username = username.map(str::to_string);
        }
        self.courses_semesters
            .entry(course.code.clone())
            .or_default();
        if !course.dept.is_empty() {
            self.course_depts.insert(course.dept.clone());
        }
    }

    fn assemble_courses_taught(&self) -> Vec<CourseTaught> {
        let mut out: Vec<CourseTaught> = self
            .courses_semesters
            .iter()
            .map(|(code, sems)| {
                let mut s: Vec<(u16, Sem)> = sems.iter().copied().collect();
                s.sort_by(|a, b| (b.0, a.1 as u8).cmp(&(a.0, b.1 as u8)));
                CourseTaught {
                    course_code: code.clone(),
                    semesters: s,
                }
            })
            .collect();
        out.sort_by(|a, b| a.course_code.cmp(&b.course_code));
        out
    }

    fn depts(&self) -> Vec<String> {
        let mut v: Vec<String> = self.course_depts.iter().cloned().collect();
        v.sort();
        v
    }
}

struct FceOnlyBuilder {
    display_name: Option<String>,
}

impl FceOnlyBuilder {
    fn new() -> Self {
        Self { display_name: None }
    }

    fn observe(&mut self, row: &FceRow) {
        if self.display_name.is_none() && !row.instructor_fce_key.is_empty() {
            self.display_name = Some(humanize_fce_name(&row.instructor_fce_key));
        }
    }

    fn assemble_courses_taught(&self, rows: &[&FceRow]) -> Vec<CourseTaught> {
        let mut by_course: BTreeMap<u32, HashSet<(u16, Sem)>> = BTreeMap::new();
        for row in rows {
            by_course
                .entry(row.course_id)
                .or_default()
                .insert((row.year, row.sem));
        }
        // FceRow only has course_id, not code; without a back-reference here we
        // emit empty `course_code` for fce-only profs and let the index step
        // resolve it from the courses table.
        by_course
            .into_iter()
            .map(|(_id, sems)| {
                let mut s: Vec<(u16, Sem)> = sems.into_iter().collect();
                s.sort_by(|a, b| (b.0, a.1 as u8).cmp(&(a.0, b.1 as u8)));
                CourseTaught {
                    course_code: String::new(),
                    semesters: s,
                }
            })
            .collect()
    }

    fn depts(&self, _rows: &[&FceRow]) -> Vec<String> {
        Vec::new()
    }
}

fn humanize_fce_name(raw: &str) -> String {
    let trimmed = raw.trim();
    let Some((last, first)) = trimmed.split_once(',') else {
        return trimmed.to_string();
    };
    let last = title_word(last.trim());
    let first = first.trim();
    if first.is_empty() {
        last
    } else {
        let first = title_word(first);
        format!("{first} {last}")
    }
}

fn title_word(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => {
                    c.to_ascii_uppercase().to_string() + &chars.as_str().to_ascii_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Normalize either form ("First Last" from Stellic, "LAST, FIRST" from FCE)
/// into "LAST, F" uppercase.
fn first_initial_key(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    let (last, first_initial) = if let Some((last, first)) = s.split_once(',') {
        let last = last.trim();
        let first = first.trim();
        let initial = first.chars().find(|c| c.is_alphabetic())?;
        (last.to_string(), initial)
    } else {
        let mut parts = s.split_whitespace();
        let first = parts.next()?;
        let last = parts.last().unwrap_or(first);
        let initial = first.chars().find(|c| c.is_alphabetic())?;
        (last.to_string(), initial)
    };
    if last.is_empty() {
        return None;
    }
    Some(format!(
        "{}, {}",
        last.to_ascii_uppercase(),
        initial_uppercase(first_initial)
    ))
}

fn initial_uppercase(c: char) -> char {
    c.to_ascii_uppercase()
}

fn aggregate_all(rows: &[&FceRow]) -> Option<FceAggregate> {
    let mut acc = SumAcc::default();
    let mut terms: HashSet<(u16, Sem)> = HashSet::new();
    for row in rows {
        terms.insert((row.year, row.sem));
        acc.push(row);
    }
    if acc.any() == 0 {
        return None;
    }
    Some(FceAggregate {
        n_semesters: terms.len() as u32,
        hrs_per_week: acc.hrs.mean(),
        interest: acc.interest.mean(),
        clarity: acc.clarity.mean(),
        feedback: acc.feedback.mean(),
        importance: acc.importance.mean(),
        explanation: acc.explanation.mean(),
        respect: acc.respect.mean(),
        overall_teaching: acc.overall_teaching.mean(),
        overall_course: acc.overall_course.mean(),
    })
}

#[derive(Default)]
struct SumAcc {
    hrs: SumCount,
    interest: SumCount,
    clarity: SumCount,
    feedback: SumCount,
    importance: SumCount,
    explanation: SumCount,
    respect: SumCount,
    overall_teaching: SumCount,
    overall_course: SumCount,
}

impl SumAcc {
    fn push(&mut self, row: &FceRow) {
        self.hrs.add(row.hrs_per_week);
        self.interest.add(row.interest);
        self.clarity.add(row.instructor_clarity);
        self.feedback.add(row.feedback);
        self.importance.add(row.importance);
        self.explanation.add(row.explanation);
        self.respect.add(row.respect);
        self.overall_teaching.add(row.overall_teaching);
        self.overall_course.add(row.overall_course);
    }

    fn any(&self) -> u32 {
        self.hrs.n
            + self.interest.n
            + self.clarity.n
            + self.feedback.n
            + self.importance.n
            + self.explanation.n
            + self.respect.n
            + self.overall_teaching.n
            + self.overall_course.n
    }
}

#[derive(Default)]
struct SumCount {
    sum: f64,
    n: u32,
}

impl SumCount {
    fn add(&mut self, v: Option<f32>) {
        if let Some(x) = v {
            self.sum += x as f64;
            self.n += 1;
        }
    }
    fn mean(&self) -> f32 {
        if self.n == 0 {
            0.0
        } else {
            (self.sum / self.n as f64) as f32
        }
    }
}
