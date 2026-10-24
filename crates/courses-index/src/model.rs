//! Courseweave's semester data model (`src/lib/types.ts`). Field order and omission follow the objects its build
//! scripts create, so serialized documents match its files.

use serde::{Deserialize, Serialize};

use crate::js::{Object, ser_num, ser_opt_num};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MiniMeta {
    pub number: u8,
    pub name: String,
    pub start: String,
    pub end: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
    pub name: String,
}

/// A term's metadata, as the term documents and the term index carry it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TermMeta {
    pub id: String,
    pub name: String,
    pub season: String,
    pub year: i64,
    pub minis: Vec<MiniMeta>,
    pub classes_begin: String,
    pub classes_end: String,
    pub no_classes: Vec<DateRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<String>>,
    /// Source adapters the term was built from.
    pub sources: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Meeting {
    /// 0 for Monday through 6 for Sunday; empty when TBA.
    pub days: Vec<u8>,
    /// Minutes after midnight, or `None` when TBA.
    pub start: Option<i32>,
    pub end: Option<i32>,
    pub building: Option<String>,
    pub room: Option<String>,
}

impl Meeting {
    pub fn tba() -> Self {
        Self {
            days: Vec::new(),
            start: None,
            end: None,
            building: None,
            room: None,
        }
    }

    /// `days|start|end|building|room`, Courseweave's `meetingKey`.
    pub fn key(&self) -> String {
        let opt = |v: Option<i32>| v.map_or_else(|| "null".to_owned(), |v| v.to_string());
        let place = |v: &Option<String>| v.clone().unwrap_or_else(|| "null".to_owned());
        let days: String = self.days.iter().map(|d| d.to_string()).collect();
        format!(
            "{days}|{}|{}|{}|{}",
            opt(self.start),
            opt(self.end),
            place(&self.building),
            place(&self.room)
        )
    }

    /// Has days and a start time.
    pub fn is_scheduled(&self) -> bool {
        !self.days.is_empty() && self.start.is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Lecture,
    Section,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Lecture => "lecture",
            Kind::Section => "section",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Section {
    pub name: String,
    pub kind: Kind,
    pub parent: Option<String>,
    pub mini: Option<u8>,
    pub mode: String,
    pub instructors: Vec<String>,
    pub meetings: Vec<Meeting>,
    /// Set when the section's title differs from the course's (topics courses).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservations: Option<Vec<String>>,
    /// Enrollment cap.
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "ser_opt_num"
    )]
    pub capacity: Option<f64>,
}

impl Section {
    pub fn new(
        name: String,
        kind: Kind,
        parent: Option<String>,
        mini: Option<u8>,
        mode: String,
    ) -> Self {
        Self {
            name,
            kind,
            parent,
            mini,
            mode,
            instructors: Vec::new(),
            meetings: Vec::new(),
            title: None,
            reservations: None,
            capacity: None,
        }
    }

    /// Adds a meeting unless one with the same key is there, as Courseweave's `addMeeting`.
    pub fn add_meeting(&mut self, meeting: Meeting) {
        let key = meeting.key();
        if !self.meetings.iter().any(|m| m.key() == key) {
            self.meetings.push(meeting);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    /// Five digits without a dash, such as `15122`.
    pub id: String,
    /// Such as `15-122`.
    pub code: String,
    pub title: String,
    /// `None` for variable units.
    #[serde(serialize_with = "ser_opt_num")]
    pub units: Option<f64>,
    pub dept: String,
    /// `U` or `G`.
    pub level: String,
    pub instructors: Vec<String>,
    pub sections: Vec<Section>,
    pub cross_listed: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass_fail: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TermData {
    pub term: TermMeta,
    pub generated_at: String,
    pub courses: Vec<Course>,
}

impl TermData {
    /// The public version: no instructors or rooms.
    pub fn redacted(&self) -> TermData {
        let mut out = self.clone();
        for c in &mut out.courses {
            c.instructors.clear();
            for s in &mut c.sections {
                s.instructors.clear();
                for m in &mut s.meetings {
                    m.building = None;
                    m.room = None;
                }
            }
        }
        out
    }

    pub fn section_count(&self) -> usize {
        self.courses.iter().map(|c| c.sections.len()).sum()
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TermIndexEntry {
    #[serde(flatten)]
    pub term: TermMeta,
    pub course_count: usize,
    pub section_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Offering {
    pub title: String,
    pub dept: String,
    pub level: String,
    pub terms: Vec<String>,
}

/// Which terms offer each course, newest first.
#[derive(Clone, Debug, Serialize)]
pub struct OfferingsData {
    pub terms: Vec<String>,
    pub courses: Object<Offering>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TeachingTerm {
    pub courses: Vec<String>,
    pub sections: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Teaching {
    /// Departments of the courses they teach, most sections first.
    pub depts: Vec<String>,
    pub terms: Object<TeachingTerm>,
}

/// Who teaches what in each term.
#[derive(Clone, Debug, Serialize)]
pub struct TeachingData {
    pub terms: Vec<String>,
    pub instructors: Object<Teaching>,
}

/// A JSON number that serializes like JavaScript's.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Num(#[serde(serialize_with = "ser_num")] pub f64);

pub fn course_code(id: &str) -> String {
    format!("{}-{}", &id[..2.min(id.len())], &id[2.min(id.len())..])
}

pub fn is_course_id(id: &str) -> bool {
    id.len() == 5 && id.bytes().all(|b| b.is_ascii_digit())
}
