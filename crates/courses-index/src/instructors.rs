//! Instructor lists: `instructors/<termId>` for each semester and `instructors/all`, new in CMU Courses.
//!
//! The website's instructor list (`/instructors`) used to build itself from the whole semester document (or, for all
//! semesters, from `teaching`, `offerings`, `catalog/index` and `fce/instructors`): about 8 MB of JSON before the
//! first name showed. These documents hold only what the list shows and searches, built the way the website built
//! it, so the list needs one small document. The instructor's own page still loads the full documents.
//!
//! Both are member-only, like the instructors and FCEs they come from (decision 16):
//!
//! ```json
//! {
//!   "titles": { "15112": "Fundamentals of Programming and Computer Science" },
//!   "instructors": [
//!     {
//!       "name": "David Kosbie",
//!       "depts": ["CS"],
//!       "courses": ["15112", "15151"],
//!       "sections": 4,
//!       "topics": { "15390": ["Special Topic: Robotics"] },
//!       "terms": ["S27", "F26"],
//!       "fce": { "rating": 4.52, "courses": ["15110", "15112"] }
//!     }
//!   ]
//! }
//! ```
//!
//! - `titles`: the title of every course an instructor in the list teaches (or, in `instructors/all`, has FCEs for),
//!   for search. A semester's titles are its own; all semesters use the newest semester's title, then the catalog's.
//! - `instructors`, in the order the website gives out URL slugs, so a list built from this document links to the same
//!   pages as one built from the full documents: a semester's in the order its sections first name them (the website's
//!   `indexTerm`); all semesters' in `teaching` order, then the people only FCEs know (`buildAllInstructors`).
//! - `name`: as the Schedule of Classes prints it; FCE-only people get their FCE name, first name first.
//! - `depts`, `courses` and `sections`: what the website's `Instructor` holds. A semester's departments and courses
//!   are in the order its sections name them, and recitations that only repeat their lecture's instructors don't
//!   count (`teachesSection`); all semesters' courses are sorted and their departments are `teaching`'s, most sections
//!   first.
//! - `topics` (semesters only, left out when empty): titles of sections whose title differs from their course's, by
//!   course, for search.
//! - `terms` (all semesters only): the built semesters they teach in, newest first; empty for FCE-only people.
//! - `fce` (left out when no FCE record matches the name, with `fce::names::NameIndex` as the website matches
//!   names): `rating`, the teaching rating over their last [`RATING_TERMS`] semesters with responses, weighted by
//!   responses and rounded to 0.01 (null when none rated teaching); and `courses`, the courses `fce/instructors`
//!   lists for them, sorted, which say which `fce/courses/<NN>` documents hold their rows.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Serialize;

use crate::fce::InstructorOut;
use crate::fce::js::{Num, opt, round_to};
use crate::fce::names::{NameIndex, display_fce_name};
use crate::model::{OfferingsData, TeachingData, TermData};
use crate::terms::teaches_section;

/// Semesters with responses that an instructor's list rating averages, as the website's `recentTeachingRate`.
pub const RATING_TERMS: usize = 4;

/// The name of the all-semesters list.
pub const ALL: &str = "instructors/all";

/// One `instructors/...` document.
#[derive(Debug, Default, Serialize)]
pub struct List {
    pub titles: BTreeMap<String, String>,
    pub instructors: Vec<Listed>,
}

/// One instructor in a list.
#[derive(Debug, Serialize)]
pub struct Listed {
    pub name: String,
    pub depts: Vec<String>,
    pub courses: Vec<String>,
    pub sections: usize,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub topics: BTreeMap<String, Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fce: Option<Evaluated>,
}

/// What a list shows of an instructor's FCEs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Evaluated {
    pub rating: Option<Num>,
    pub courses: Vec<String>,
}

/// `fce/instructors`, looked up by schedule-style names as the website looks it up.
pub struct Fces<'a> {
    index: NameIndex<usize>,
    people: Vec<&'a InstructorOut>,
}

impl<'a> Fces<'a> {
    /// `instructors` is `fce/instructors`' map, keyed by name key; it's added in key order, as the website reads it.
    pub fn new(instructors: &'a BTreeMap<String, InstructorOut>) -> Self {
        let mut index = NameIndex::new();
        let mut people = Vec::with_capacity(instructors.len());
        for (key, out) in instructors {
            index.add_key(key, people.len());
            people.push(out);
        }
        Self { index, people }
    }

    /// The FCE record for a schedule-style name, and its position.
    fn get(&self, name: &str) -> Option<(usize, &'a InstructorOut)> {
        self.index.get(name).map(|&i| (i, self.people[i]))
    }
}

/// The list's view of an FCE record.
pub fn evaluated(out: &InstructorOut) -> Evaluated {
    let recent = &out.terms[out.terms.len().saturating_sub(RATING_TERMS)..];
    let (mut sum, mut weight) = (0.0, 0.0);
    for (_, rate, responses) in recent {
        let Some(rate) = rate else { continue };
        if responses.0 <= 0.0 {
            continue;
        }
        sum += rate.0 * responses.0;
        weight += responses.0;
    }
    let rating = (weight != 0.0).then(|| sum / weight);
    Evaluated {
        rating: opt(round_to(rating, 2)),
        courses: out.courses.keys().cloned().collect(),
    }
}

fn push_new(list: &mut Vec<String>, value: &str) {
    if !list.iter().any(|v| v == value) {
        list.push(value.to_owned());
    }
}

/// `instructors/<termId>`: who teaches in one semester.
pub fn term_list(term: &TermData, fces: Option<&Fces>) -> List {
    let mut order: Vec<Listed> = Vec::new();
    let mut position: HashMap<&str, usize> = HashMap::new();
    for course in &term.courses {
        for section in &course.sections {
            for name in &section.instructors {
                if !teaches_section(course, section, name) {
                    continue;
                }
                let i = *position.entry(name.as_str()).or_insert_with(|| {
                    order.push(Listed {
                        name: name.clone(),
                        depts: Vec::new(),
                        courses: Vec::new(),
                        sections: 0,
                        topics: BTreeMap::new(),
                        terms: None,
                        fce: None,
                    });
                    order.len() - 1
                });
                let listed = &mut order[i];
                listed.sections += 1;
                push_new(&mut listed.courses, &course.id);
                push_new(&mut listed.depts, &course.dept);
            }
        }
    }

    let by_id: HashMap<&str, _> = term.courses.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut titles = BTreeMap::new();
    for listed in &mut order {
        for id in &listed.courses {
            let Some(course) = by_id.get(id.as_str()) else {
                continue;
            };
            titles.insert(id.clone(), course.title.clone());
            // Any section naming them, as the website's search does (not only the ones they teach).
            let mut topics: Vec<String> = Vec::new();
            for section in &course.sections {
                if let Some(title) = &section.title
                    && section.instructors.contains(&listed.name)
                {
                    push_new(&mut topics, title);
                }
            }
            if !topics.is_empty() {
                listed.topics.insert(id.clone(), topics);
            }
        }
        listed.fce = fces
            .and_then(|f| f.get(&listed.name))
            .map(|(_, out)| evaluated(out));
    }
    List {
        titles,
        instructors: order,
    }
}

/// A catalog title worth showing: not one that only repeats the course number ("15-101"), as the website's
/// `catalogTitle`.
fn catalog_title(title: &str) -> Option<&str> {
    // `/^\d{2}-?\d{3}$/` on the trimmed title.
    let t = crate::js::trim(title).as_bytes();
    let digits = |b: &[u8]| b.iter().all(u8::is_ascii_digit);
    let number = match t.len() {
        5 => digits(t),
        6 => t[2] == b'-' && digits(&t[..2]) && digits(&t[3..]),
        _ => false,
    };
    (!title.is_empty() && !number).then_some(title)
}

/// Titles and departments for any course: the newest semester offering it, then the catalog (the website's
/// `courseLookup`).
struct Lookup<'a, F> {
    offered: HashMap<&'a str, (&'a str, &'a str)>,
    dept_by_prefix: HashMap<&'a str, &'a str>,
    catalog: F,
}

impl<'a, F: Fn(&str) -> Option<String>> Lookup<'a, F> {
    fn new(offerings: &'a OfferingsData, catalog: F) -> Self {
        let mut dept_by_prefix = HashMap::new();
        // In JavaScript's key order, as the website walks the offerings.
        for (id, o) in offerings.courses.ordered() {
            dept_by_prefix
                .entry(id.get(..2).unwrap_or(id))
                .or_insert(o.dept.as_str());
        }
        let offered = offerings
            .courses
            .0
            .iter()
            .map(|(id, o)| (id.as_str(), (o.title.as_str(), o.dept.as_str())))
            .collect();
        Self {
            offered,
            dept_by_prefix,
            catalog,
        }
    }

    fn title(&self, id: &str) -> Option<String> {
        match self.offered.get(id) {
            Some((title, _)) => Some((*title).to_owned()),
            None => (self.catalog)(id).filter(|t| catalog_title(t).is_some()),
        }
    }

    fn dept(&self, id: &str) -> Option<&'a str> {
        self.offered
            .get(id)
            .map(|(_, dept)| *dept)
            .or_else(|| self.dept_by_prefix.get(id.get(..2).unwrap_or(id)).copied())
    }
}

/// `instructors/all`: everyone teaching in a built semester, then everyone else FCEs know. `catalog` gives a course's
/// title in the course catalog.
pub fn all_list(
    teaching: &TeachingData,
    offerings: &OfferingsData,
    catalog: impl Fn(&str) -> Option<String>,
    fces: Option<&Fces>,
) -> List {
    let lookup = Lookup::new(offerings, catalog);
    let mut list = Vec::new();
    let mut matched = BTreeSet::new();
    for (name, taught) in &teaching.instructors.0 {
        let terms: Vec<String> = teaching
            .terms
            .iter()
            .filter(|id| taught.terms.0.iter().any(|(t, _)| t == *id))
            .cloned()
            .collect();
        let mut courses = BTreeSet::new();
        let mut sections = 0;
        for (_, in_term) in &taught.terms.0 {
            courses.extend(in_term.courses.iter().cloned());
            sections += in_term.sections;
        }
        let fce = fces.and_then(|f| f.get(name)).map(|(i, out)| {
            matched.insert(i);
            evaluated(out)
        });
        list.push(Listed {
            name: name.clone(),
            depts: taught.depts.clone(),
            courses: courses.into_iter().collect(),
            sections,
            topics: BTreeMap::new(),
            terms: Some(terms),
            fce,
        });
    }
    if let Some(fces) = fces {
        for (i, out) in fces.people.iter().enumerate() {
            if matched.contains(&i) {
                continue;
            }
            let courses: Vec<String> = out.courses.keys().cloned().collect();
            let mut depts = Vec::new();
            for id in &courses {
                if let Some(dept) = lookup.dept(id) {
                    push_new(&mut depts, dept);
                }
            }
            list.push(Listed {
                name: display_fce_name(&out.name),
                depts,
                courses,
                sections: 0,
                topics: BTreeMap::new(),
                terms: Some(Vec::new()),
                fce: Some(evaluated(out)),
            });
        }
    }

    let mut titles = BTreeMap::new();
    for listed in &list {
        for id in &listed.courses {
            if let Some(title) = lookup.title(id) {
                titles.insert(id.clone(), title);
            }
        }
    }
    List {
        titles,
        instructors: list,
    }
}

/// Every instructor list: one per semester, then `instructors/all`. `catalog` gives a course's title in the course
/// catalog, and `fce` is `fce/instructors`' map when the catalog has FCEs.
pub fn documents(
    terms: &[TermData],
    teaching: &TeachingData,
    offerings: &OfferingsData,
    catalog: impl Fn(&str) -> Option<String>,
    fce: Option<&BTreeMap<String, InstructorOut>>,
) -> Vec<(String, List)> {
    let fces = fce.map(Fces::new);
    let mut out: Vec<(String, List)> = terms
        .iter()
        .map(|t| {
            (
                format!("instructors/{}", t.term.id),
                term_list(t, fces.as_ref()),
            )
        })
        .collect();
    out.push((
        ALL.to_owned(),
        all_list(teaching, offerings, catalog, fces.as_ref()),
    ));
    out
}

#[cfg(test)]
mod tests;
