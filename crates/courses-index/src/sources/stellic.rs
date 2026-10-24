//! Stellic `getcoursesections` responses, as the scraper saves them: `<dir>/<5 digits>/ly<lyear>_sm<sem>.json`
//! (Courseweave's `scripts/sources/stellic.ts` and `scripts/lib/stellic-parse.ts`).
//!
//! Options: `dir` (relative to the exported root; default the one account's `courses_history`), `lyear` (Stellic's
//! academic-year offset from the exporting account's first term) and `sem` (1 fall, 2 spring, 3 summer), both
//! optional when `<dir>/meta.json` records them, `campuses` (Stellic campus ids to keep; 1 is Pittsburgh) and
//! `optional` (skip a missing dir with a warning).
//!
//! Stellic has recitations and labs with parent links, rooms, instructors and enrollment caps, but no delivery mode
//! or SOC department code (derived from the course number).

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use tracing::warn;

use super::{Inputs, Loaded, fetched_at, read_json, stellic_export_dir};
use crate::config::SourceConfig;
use crate::js;
use crate::model::{Course, Kind, Meeting, Section, TermMeta, course_code, is_course_id};
use crate::text::{clock, day_name};

#[derive(Deserialize)]
pub struct SectionsFile {
    #[serde(default)]
    pub data_list: Option<Vec<Entry>>,
}

#[derive(Deserialize)]
pub struct Entry {
    pub course: CourseRef,
    #[serde(default)]
    pub section_groups: Option<Vec<Group>>,
}

#[derive(Deserialize)]
pub struct CourseRef {
    pub name: String,
    #[serde(default)]
    pub units: Option<Value>,
}

#[derive(Deserialize)]
pub struct Group {
    #[serde(default)]
    pub group_name: Option<String>,
    #[serde(default)]
    pub sections: Option<Vec<StellicSection>>,
}

#[derive(Deserialize)]
pub struct Named {
    pub name: String,
}

#[derive(Deserialize)]
pub struct SectionType {
    pub short_name: Option<String>,
}

#[derive(Deserialize)]
pub struct Instructor {
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
}

#[derive(Deserialize)]
pub struct Timing {
    #[serde(default)]
    pub building: Option<String>,
    #[serde(default)]
    pub day: Option<String>,
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub end: Option<String>,
    #[serde(default)]
    pub room: Option<String>,
}

#[derive(Deserialize)]
pub struct StellicSection {
    pub name: String,
    #[serde(default)]
    pub parent: Option<Named>,
    #[serde(rename = "type", default)]
    pub kind: Option<SectionType>,
    #[serde(default)]
    pub instructors: Option<Vec<Instructor>>,
    #[serde(default)]
    pub raw_timings: Option<Vec<Timing>>,
    #[serde(default)]
    pub max_enrollment: Option<Value>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub topic: Option<Value>,
    #[serde(default)]
    pub hidden: Option<Value>,
    #[serde(default)]
    pub campus: Option<Value>,
}

/// Course-number prefix to SOC department code (Stellic doesn't carry it).
pub fn prefix_dept(prefix: &str) -> Option<&'static str> {
    Some(match prefix {
        "00" => "IED",
        "01" => "MIS",
        "02" => "CB",
        "03" => "BSC",
        "05" => "HCI",
        "06" => "CHE",
        "07" => "SCS",
        "09" => "CMY",
        "10" => "MLG",
        "11" => "LTI",
        "12" => "CEE",
        "14" => "INI",
        "15" => "CS",
        "16" => "ROB",
        "17" => "S3D",
        "18" => "ECE",
        "19" => "EPP",
        "21" => "MSC",
        "24" => "MEG",
        "27" => "MSE",
        "32" => "NVS",
        "33" => "PHY",
        "36" => "STA",
        "38" => "MCS",
        "39" => "CIT",
        "42" => "BMD",
        "45" | "46" | "47" | "70" => "BUS",
        "48" => "ARC",
        "49" => "III",
        "51" => "DES",
        "52" => "BXA",
        "53" => "ETC",
        "54" => "DRA",
        "57" => "MUS",
        "60" => "ART",
        "62" => "CFA",
        "65" => "H00",
        "66" => "HSS",
        "67" => "ISP",
        "69" => "PE",
        "73" => "ECO",
        "76" => "ENG",
        "79" => "HIS",
        "80" => "PHI",
        "82" => "LCL",
        "84" => "CST",
        "85" => "PSY",
        "86" => "NSI",
        "88" => "SDS",
        "90" => "PPP",
        "91" => "PMP",
        "92" => "MED",
        "93" => "AEM",
        "94" => "HC",
        "95" => "ISM",
        "98" => "STU",
        "99" => "CMU",
        _ => return None,
    })
}

fn truthy(v: &Option<Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|f| f != 0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}

const DAY_MS: f64 = 86_400_000.0;

/// Mini number for a section. Dates win: a section spanning most of the term is full-semester, otherwise it's the
/// mini whose midpoint is closest. Falls back to the name's last digit (A3 is mini 3) when dates are missing or
/// aren't in this term.
pub fn detect_mini(
    name: &str,
    start: Option<&str>,
    end: Option<&str>,
    term: &TermMeta,
) -> Option<u8> {
    let by_name = crate::text::mini_of(name, term);
    let (Some(s), Some(e)) = (start.and_then(js::parse_date), end.and_then(js::parse_date)) else {
        return by_name;
    };
    let (Some(term_start), Some(term_end)) = (
        js::parse_date(&term.classes_begin),
        js::parse_date(&term.classes_end),
    ) else {
        return by_name;
    };
    let in_term = s >= term_start - 30.0 * DAY_MS && e <= term_end + 30.0 * DAY_MS;
    if !in_term || term.minis.is_empty() {
        return by_name;
    }
    if e - s >= (term_end - term_start) * 0.7 {
        return None;
    }
    let mid = (s + e) / 2.0;
    let mut best = None;
    let mut best_dist = f64::INFINITY;
    for m in &term.minis {
        let (Some(ms), Some(me)) = (js::parse_date(&m.start), js::parse_date(&m.end)) else {
            continue;
        };
        let d = ((ms + me) / 2.0 - mid).abs();
        if d < best_dist {
            best_dist = d;
            best = Some(m.number);
        }
    }
    best
}

fn place(s: Option<&str>) -> Option<String> {
    let t = js::trim(s.unwrap_or(""));
    if t.is_empty()
        || ["TBA", "TBD", "DNM"]
            .iter()
            .any(|p| p.eq_ignore_ascii_case(t))
    {
        None
    } else {
        Some(t.to_owned())
    }
}

/// Groups per-day timings with the same time and room into one meeting. No timings is one TBA meeting.
pub fn timings_to_meetings(timings: &[Timing]) -> Vec<Meeting> {
    let mut meetings: Vec<Meeting> = Vec::new();
    for t in timings {
        let day = t.day.as_deref().and_then(|d| day_name(&d.to_lowercase()));
        let start = clock(js::trim(t.start.as_deref().unwrap_or("")), false);
        let end = clock(js::trim(t.end.as_deref().unwrap_or("")), false);
        let building = place(t.building.as_deref());
        let room = if building.is_some() {
            place(t.room.as_deref())
        } else {
            None
        };
        let i = match meetings.iter().position(|m| {
            m.start == start && m.end == end && m.building == building && m.room == room
        }) {
            Some(i) => i,
            None => {
                meetings.push(Meeting {
                    days: Vec::new(),
                    start,
                    end,
                    building,
                    room,
                });
                meetings.len() - 1
            }
        };
        if let Some(d) = day
            && !meetings[i].days.contains(&d)
        {
            meetings[i].days.push(d);
        }
    }
    for m in &mut meetings {
        m.days.sort();
    }
    if meetings.is_empty() {
        meetings.push(Meeting::tba());
    }
    meetings
}

fn topic_title(topic: &Option<Value>) -> Option<String> {
    let s = match topic {
        Some(Value::String(s)) => s,
        Some(Value::Object(o)) => o.get("name")?.as_str()?,
        _ => return None,
    };
    Some(js::trim(s).to_owned()).filter(|t| !t.is_empty())
}

/// One course from a sections file; `None` when it has no usable sections.
pub fn parse_course(
    id: &str,
    file: &SectionsFile,
    term: &TermMeta,
    campuses: Option<&[f64]>,
) -> Option<Course> {
    let entries = file.data_list.as_deref().unwrap_or_default();
    let first = &entries.first()?.course;
    let title = js::squish(&first.name);

    let mut sections: Vec<Section> = Vec::new();
    for entry in entries {
        for group in entry.section_groups.as_deref().unwrap_or_default() {
            for s in group.sections.as_deref().unwrap_or_default() {
                if truthy(&s.hidden) {
                    continue;
                }
                if let (Some(campuses), Some(campus)) =
                    (campuses, s.campus.as_ref().filter(|c| !c.is_null()))
                    && !campus.as_f64().is_some_and(|c| campuses.contains(&c))
                {
                    continue;
                }
                if sections.iter().any(|x| x.name == s.name) {
                    continue;
                }
                // The SOC only calls numbered sections lectures (Lec 1); Stellic also types lettered standalone
                // sections as L.
                let typed_lecture = match &s.kind {
                    Some(t) => t.short_name.as_deref() == Some("L"),
                    None => group.group_name.as_deref() == Some("Lecture"),
                };
                let lecture = typed_lecture && s.name.starts_with(|c: char| c.is_ascii_digit());
                let mut instructors: Vec<String> = Vec::new();
                for i in s.instructors.as_deref().unwrap_or_default() {
                    let name = js::squish(&format!(
                        "{} {}",
                        i.first_name.as_deref().unwrap_or(""),
                        i.last_name.as_deref().unwrap_or("")
                    ));
                    // Stellic uses "[] []" for unassigned instructors.
                    if name.chars().any(|c| c.is_ascii_alphabetic()) && !instructors.contains(&name)
                    {
                        instructors.push(name);
                    }
                }
                let mut section = Section::new(
                    s.name.clone(),
                    if lecture {
                        Kind::Lecture
                    } else {
                        Kind::Section
                    },
                    s.parent.as_ref().map(|p| p.name.clone()),
                    detect_mini(
                        &s.name,
                        s.start_date.as_deref(),
                        s.end_date.as_deref(),
                        term,
                    ),
                    String::new(),
                );
                section.instructors = instructors;
                section.meetings =
                    timings_to_meetings(s.raw_timings.as_deref().unwrap_or_default());
                if let Some(topic) = topic_title(&s.topic).filter(|t| *t != title) {
                    section.title = Some(topic);
                }
                section.capacity = s.max_enrollment.as_ref().and_then(Value::as_f64);
                sections.push(section);
            }
        }
    }
    if sections.is_empty() {
        return None;
    }

    let units = first
        .units
        .as_ref()
        .and_then(Value::as_f64)
        .filter(|u| *u >= 0.0);
    let prefix = &id[..2];
    Some(Course {
        id: id.to_owned(),
        code: course_code(id),
        title,
        units,
        dept: prefix_dept(prefix).unwrap_or(prefix).to_owned(),
        level: if id[2..].parse::<u32>().unwrap_or(0) >= 600 {
            "G"
        } else {
            "U"
        }
        .to_owned(),
        instructors: Vec::new(),
        sections,
        cross_listed: Vec::new(),
        permission_required: None,
        pass_fail: None,
    })
}

/// `Number(x)` of an option.
fn number(v: Option<&Value>) -> Option<f64> {
    match v? {
        Value::Null => None,
        Value::Number(n) => n.as_f64(),
        Value::String(s) => Some(js::number(s)),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => Some(f64::NAN),
    }
}

pub fn load(source: &SourceConfig, term: &TermMeta, inputs: &Inputs) -> Result<Loaded> {
    let dir = match source.str("dir") {
        Some(rel) => Some(inputs.exported.join(rel)),
        None => stellic_export_dir(inputs)?,
    };
    let Some(dir) = dir.filter(|d| d.is_dir()) else {
        if source.truthy("optional") {
            warn!(term = %term.id, "stellic export not found; skipping");
            return Ok(Loaded {
                courses: Vec::new(),
                fetched_at: None,
            });
        }
        bail!("{}: stellic export not found", term.id);
    };
    let meta_path = dir.join("meta.json");
    let meta: HashMap<String, Value> = if meta_path.exists() {
        read_json(&meta_path)?
    } else {
        HashMap::new()
    };
    let lyear = number(
        source
            .options
            .get("lyear")
            .filter(|v| !v.is_null())
            .or(meta.get("lyear")),
    );
    let sem = number(
        source
            .options
            .get("sem")
            .filter(|v| !v.is_null())
            .or(meta.get("sem")),
    );
    let (Some(lyear), Some(sem)) = (
        lyear.filter(|l| l.fract() == 0.0),
        sem.filter(|s| [1.0, 2.0, 3.0].contains(s)),
    ) else {
        bail!(
            "{}: stellic source needs integer lyear and sem (1 fall, 2 spring, 3 summer), or a meta.json that records them",
            term.id
        );
    };
    let campuses: Option<Vec<f64>> = match source.options.get("campuses") {
        Some(Value::Array(list)) => Some(
            list.iter()
                .map(|v| number(Some(v)).unwrap_or(0.0))
                .collect(),
        ),
        _ => None,
    };

    let file_name = format!("ly{}_sm{}.json", lyear as i64, sem as i64);
    let ids = course_dirs(&dir)?;
    let courses = ids
        .par_iter()
        .map(|id| -> Result<Option<Course>> {
            let path = dir.join(id).join(&file_name);
            if !path.exists() {
                return Ok(None);
            }
            let file: SectionsFile = read_json(&path)?;
            Ok(parse_course(id, &file, term, campuses.as_deref()))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();
    Ok(Loaded {
        courses,
        fetched_at: fetched_at(&meta_path),
    })
}

/// Course-id subdirectories, sorted.
pub fn course_dirs(dir: &Path) -> Result<Vec<String>> {
    let mut ids: Vec<String> = std::fs::read_dir(dir)
        .with_context(|| format!("listing {}", dir.display()))?
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|id| is_course_id(id))
        .collect();
    ids.sort();
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MiniMeta;

    fn fall() -> TermMeta {
        TermMeta {
            id: "F26".into(),
            name: "Fall 2026".into(),
            season: "fall".into(),
            year: 2026,
            minis: vec![
                MiniMeta {
                    number: 1,
                    name: "Mini 1".into(),
                    start: "2026-08-24".into(),
                    end: "2026-10-09".into(),
                },
                MiniMeta {
                    number: 2,
                    name: "Mini 2".into(),
                    start: "2026-10-19".into(),
                    end: "2026-12-04".into(),
                },
            ],
            classes_begin: "2026-08-24".into(),
            classes_end: "2026-12-04".into(),
            no_classes: vec![],
            notes: None,
            sources: vec![],
        }
    }

    fn timing(day: &str, start: &str, end: &str, building: &str, room: &str) -> Timing {
        Timing {
            building: Some(building.into()),
            day: Some(day.into()),
            start: Some(start.into()),
            end: Some(end.into()),
            room: Some(room.into()),
        }
    }

    const FILE: &str = r#"{"data_list":[
      {"course":{"code":"15-122","name":"Principles of  Imperative Computation","units":12.0},"section_groups":[
        {"group_name":"Lecture","sections":[{"name":"1","parent":null,"type":{"name":"Lecture","short_name":"L"},
          "instructors":[{"first_name":"Iliano","last_name":"Cervesato"},{"first_name":"[]","last_name":"[]"}],
          "raw_timings":[{"day":"tuesday","start":"07:00 PM","end":"08:20 PM","building":"GHC","room":"4401"},
            {"day":"tuesday","start":"09:30 AM","end":"10:50 AM","building":"GHC","room":"4401"},
            {"day":"thursday","start":"09:30 AM","end":"10:50 AM","building":"GHC","room":"4401"}],
          "max_enrollment":244,"start_date":"2026-08-24","end_date":"2026-12-05","campus":1}]},
        {"group_name":"Recitation","sections":[
          {"name":"A","parent":{"name":"1"},"type":{"name":"Recitation","short_name":"R"},"instructors":[],
           "raw_timings":[{"day":"friday","start":"09:00 AM","end":"09:50 AM","building":"PH","room":"226C"}],
           "max_enrollment":25,"start_date":"2026-08-24","end_date":"2026-12-05","topic":{"name":"Recitation topic"}},
          {"name":"Z","parent":{"name":"1"},"hidden":true,"raw_timings":[]},
          {"name":"Q","campus":2}]}]},
      {"course":{"code":"15-122","name":"Principles of Imperative Computation","units":12},
       "section_groups":[{"group_name":"Lecture","sections":[{"name":"1","parent":null,"raw_timings":[]}]}]}]}"#;

    #[test]
    fn clock_times() {
        assert_eq!(clock("07:00 PM", false), Some(1140));
        assert_eq!(clock("09:30AM", false), Some(570));
        assert_eq!(clock("12:00 PM", false), Some(720));
        assert_eq!(clock("", false), None);
        assert_eq!(clock("TBA", false), None);
    }

    #[test]
    fn merges_per_day_timings() {
        let m = timings_to_meetings(&[
            timing("thursday", "09:30 AM", "10:50 AM", "GHC", "4401"),
            timing("tuesday", "09:30 AM", "10:50 AM", "GHC", "4401"),
        ]);
        assert_eq!(
            m,
            [Meeting {
                days: vec![1, 3],
                start: Some(570),
                end: Some(650),
                building: Some("GHC".into()),
                room: Some("4401".into())
            }]
        );
        assert_eq!(timings_to_meetings(&[]), [Meeting::tba()]);
        assert_eq!(
            timings_to_meetings(&[timing("monday", "10:00 AM", "10:50 AM", "TBA", "TBA")])[0]
                .building,
            None
        );
    }

    #[test]
    fn detects_minis_from_dates() {
        let t = fall();
        assert_eq!(
            detect_mini("A", Some("2026-08-24"), Some("2026-12-05"), &t),
            None
        );
        assert_eq!(
            detect_mini("A", Some("2026-08-24"), Some("2026-10-10"), &t),
            Some(1)
        );
        assert_eq!(
            detect_mini("12", Some("2026-10-19"), Some("2026-12-05"), &t),
            Some(2)
        );
        assert_eq!(detect_mini("A2", None, None, &t), Some(2));
        // Dates from another year don't count.
        assert_eq!(
            detect_mini("A1", Some("2025-08-25"), Some("2025-12-06"), &t),
            Some(1)
        );
    }

    #[test]
    fn builds_lectures_with_children() {
        let file: SectionsFile = serde_json::from_str(FILE).unwrap();
        let c = parse_course("15122", &file, &fall(), Some(&[1.0])).unwrap();
        assert_eq!(
            (c.code.as_str(), c.dept.as_str(), c.level.as_str()),
            ("15-122", "CS", "U")
        );
        assert_eq!(c.title, "Principles of Imperative Computation");
        assert_eq!(c.units, Some(12.0));
        let shape: Vec<_> = c
            .sections
            .iter()
            .map(|s| (s.name.as_str(), s.kind, s.parent.as_deref()))
            .collect();
        assert_eq!(
            shape,
            [("1", Kind::Lecture, None), ("A", Kind::Section, Some("1"))]
        );
        assert_eq!(c.sections[0].meetings.len(), 2);
        assert_eq!(c.sections[0].instructors, ["Iliano Cervesato"]);
        assert_eq!(c.sections[0].capacity, Some(244.0));
        assert_eq!(c.sections[1].title.as_deref(), Some("Recitation topic"));
        let m = &c.sections[1].meetings[0];
        assert_eq!(
            (m.days.as_slice(), m.start, m.building.as_deref()),
            (&[4u8][..], Some(540), Some("PH"))
        );

        let all = parse_course("15122", &file, &fall(), None).unwrap();
        assert_eq!(all.sections.len(), 3);
    }

    #[test]
    fn lettered_lectures_are_sections_and_600_level_is_graduate() {
        let file: SectionsFile = serde_json::from_str(
            r#"{"data_list":[{"course":{"code":"76-101","name":"Interpretation and Argument","units":9},
            "section_groups":[{"group_name":"Lecture","sections":[{"name":"A","parent":null,"type":{"name":"Lecture","short_name":"L"}}]}]}]}"#,
        )
        .unwrap();
        assert_eq!(
            parse_course("76101", &file, &fall(), None)
                .unwrap()
                .sections[0]
                .kind,
            Kind::Section
        );
        let grad = parse_course("15641", &file, &fall(), None).unwrap();
        assert_eq!(grad.level, "G");
        let empty: SectionsFile = serde_json::from_str(r#"{"data_list":[]}"#).unwrap();
        assert!(parse_course("15122", &empty, &fall(), None).is_none());
    }
}
