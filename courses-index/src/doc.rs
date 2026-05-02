//! In-memory document shapes produced by the loader and consumed by the
//! index builder. The on-disk wire format lives in [`crate::binary`], which
//! encodes these structs with bincode and frames them into named regions.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Dense integer id assigned at build time. Stable for a given build run and
/// never reused across codes.
pub type CourseId = u32;
pub type InstructorId = u32;
pub type SectionId = u32;

/// Semester enum, matching Stellic's 1/2/3 numbering.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Sem {
    Fall = 1,
    Spring = 2,
    Summer = 3,
}

impl Sem {
    pub fn from_u8(n: u8) -> Option<Self> {
        match n {
            1 => Some(Sem::Fall),
            2 => Some(Sem::Spring),
            3 => Some(Sem::Summer),
            _ => None,
        }
    }

    pub fn from_csv(label: &str) -> Option<Self> {
        match label.trim() {
            "Fall" => Some(Sem::Fall),
            "Spring" => Some(Sem::Spring),
            "Summer" => Some(Sem::Summer),
            _ => None,
        }
    }
}

/// Year-plus-semester, the granular offering unit.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Term {
    pub year: u16,
    pub sem: Sem,
}

impl Term {
    /// Format as the syllabus-registry short code used on disk, e.g. `F26`, `S26`, `M25`.
    /// Summer (Stellic sem 3) always maps to `M`; the syllabi registry distinguishes
    /// `M` vs `N` by session, which we collapse here since catalog data does not.
    pub fn short_code(&self) -> String {
        let yy = self.year % 100;
        let letter = match self.sem {
            Sem::Fall => 'F',
            Sem::Spring => 'S',
            Sem::Summer => 'M',
        };
        format!("{letter}{yy:02}")
    }
}

/// A single course offered by CMU, denormalized across the scrape sources.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Course {
    pub id: CourseId,
    pub code: String,
    pub dept: String,
    pub course_num: u32,
    pub name: String,
    /// Backed by `Arc<str>` so identical descriptions across courses share
    /// a single allocation. The post-load dedup pass populates this; raw
    /// loaders just construct fresh Arcs.
    pub description: Arc<str>,
    pub units: f32,
    pub units_min: f32,
    pub units_max: f32,
    /// Low-cardinality fields back onto `Arc<str>` so the post-load dedup
    /// pass can collapse repeated values to a single shared allocation
    /// without changing on-disk shape.
    pub school: Option<Arc<str>>,
    pub level: Option<Arc<str>>,
    pub attribute_tags: Vec<Arc<str>>,
    pub gened_tags: Vec<GenEdTag>,
    pub skills: Vec<Arc<str>>,
    pub prereqs_text: Option<String>,
    pub coreq_codes: Vec<String>,
    pub antireq_codes: Vec<String>,
    pub equiv_codes: Vec<String>,
    pub website: Option<String>,
    pub campuses: Vec<u32>,
    pub semesters_offered: Vec<SemesterOffered>,
    pub instructors_recent: Vec<InstructorRef>,
    pub programs: Vec<ProgramMembership>,
    pub fce_aggregates: Option<FceAggregate>,
    pub has_syllabus_terms: Vec<Arc<str>>,
    /// PageRank over the prereq graph. Populated in a later pipeline stage;
    /// zero at load time.
    pub pagerank: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenEdTag {
    pub name: Arc<str>,
    pub sem: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SemesterOffered {
    pub year: u16,
    pub sem: Sem,
    pub campus_id: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstructorRef {
    pub id: InstructorId,
    pub name: String,
    pub username: Option<String>,
    pub semesters_taught: Vec<(u16, Sem)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProgramMembership {
    pub program_id: u32,
    pub program_name: String,
    pub audit_id: u32,
    pub catalog_id: u32,
    pub program_type: u8,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FceAggregate {
    pub n_semesters: u32,
    pub hrs_per_week: f32,
    pub interest: f32,
    pub clarity: f32,
    pub feedback: f32,
    pub importance: f32,
    pub explanation: f32,
    pub respect: f32,
    pub overall_teaching: f32,
    pub overall_course: f32,
}

/// A professor or instructor known to the catalog.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Professor {
    pub id: InstructorId,
    pub stellic_id: Option<u32>,
    pub fce_key: Option<String>,
    pub name: String,
    pub username: Option<String>,
    pub depts: Vec<String>,
    pub courses_taught: Vec<CourseTaught>,
    pub fce_aggregates: Option<FceAggregate>,
    pub n_sections_taught: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CourseTaught {
    pub course_code: String,
    pub semesters: Vec<(u16, Sem)>,
}

/// Per-section timing for schedule-fit and overlap queries.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionTime {
    pub section_id: SectionId,
    pub course_id: CourseId,
    pub term: Term,
    /// Bitmask with bit 0 = Monday through bit 6 = Sunday.
    pub days: u8,
    /// Minutes from midnight, 0..1440. Async sections have `days == 0` and
    /// `start_minutes == end_minutes == 0`.
    pub start_minutes: u16,
    pub end_minutes: u16,
    pub building: Option<String>,
    pub room: Option<String>,
    pub section_type: SectionType,
    pub instructors: Vec<InstructorId>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SectionType {
    Lecture,
    Recitation,
    Studio,
    IndependentStudy,
    SpecialEquipmentLab,
    ColloquiumSeminar,
    MediaSoftwareLab,
    InternshipStudyAbroad,
    Other,
}

impl SectionType {
    pub fn from_label(s: &str) -> Self {
        match s {
            "Lecture" => SectionType::Lecture,
            "Recitation" => SectionType::Recitation,
            "Studio" => SectionType::Studio,
            "Independent Study" => SectionType::IndependentStudy,
            "Special Equipment Lab" => SectionType::SpecialEquipmentLab,
            "Colloquium Seminar" => SectionType::ColloquiumSeminar,
            "Media/Software Lab" => SectionType::MediaSoftwareLab,
            "Internship/Study Abroad" => SectionType::InternshipStudyAbroad,
            _ => SectionType::Other,
        }
    }
}

/// One FCE survey row. Preserves the raw per-(year, sem, section, instructor)
/// data so queries can average over arbitrary subsets.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FceRow {
    pub course_id: CourseId,
    pub year: u16,
    pub sem: Sem,
    pub section: String,
    pub instructor_id: Option<InstructorId>,
    pub instructor_fce_key: String,
    pub hrs_per_week: Option<f32>,
    pub interest: Option<f32>,
    pub instructor_clarity: Option<f32>,
    pub feedback: Option<f32>,
    pub importance: Option<f32>,
    pub explanation: Option<f32>,
    pub respect: Option<f32>,
    pub overall_teaching: Option<f32>,
    pub overall_course: Option<f32>,
    pub n_responses: u32,
    pub response_rate: Option<f32>,
    pub total_students: Option<u32>,
    pub course_level: Option<String>,
    pub college: Option<String>,
}
