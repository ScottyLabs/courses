use serde_json::json;

use super::*;
use crate::js::Object;
use crate::model::{Course, Kind, Offering, Section, Teaching, TeachingTerm, TermMeta};

fn section(name: &str, parent: Option<&str>, instructors: &[&str], title: Option<&str>) -> Section {
    let kind = if parent.is_some() {
        Kind::Section
    } else {
        Kind::Lecture
    };
    let mut s = Section::new(
        name.into(),
        kind,
        parent.map(str::to_owned),
        None,
        "IPE".into(),
    );
    s.instructors = instructors.iter().map(|n| (*n).to_owned()).collect();
    s.title = title.map(str::to_owned);
    s
}

fn course(id: &str, dept: &str, title: &str, sections: Vec<Section>) -> Course {
    Course {
        id: id.into(),
        code: crate::model::course_code(id),
        title: title.into(),
        units: Some(12.0),
        dept: dept.into(),
        level: "U".into(),
        instructors: vec![],
        sections,
        cross_listed: vec![],
        permission_required: None,
        pass_fail: None,
    }
}

fn term(id: &str, courses: Vec<Course>) -> TermData {
    TermData {
        term: TermMeta {
            id: id.into(),
            name: id.into(),
            season: "Spring".into(),
            year: 2027,
            minis: vec![],
            classes_begin: "2027-01-11".into(),
            classes_end: "2027-04-30".into(),
            no_classes: vec![],
            notes: None,
            sources: vec![],
        },
        generated_at: String::new(),
        courses,
    }
}

/// An FCE record with the given courses and `(term, teaching rate, responses)` history, oldest first.
fn fce(name: &str, courses: &[&str], terms: &[(&str, Option<f64>, f64)]) -> InstructorOut {
    InstructorOut {
        name: name.into(),
        courses: courses
            .iter()
            .map(|id| {
                (
                    (*id).to_owned(),
                    (
                        1,
                        Num(10.0),
                        None,
                        None,
                        None,
                        "S26".to_owned(),
                        "S26".to_owned(),
                    ),
                )
            })
            .collect(),
        terms: terms
            .iter()
            .map(|(t, rate, responses)| ((*t).to_owned(), rate.map(Num), Num(*responses)))
            .collect(),
    }
}

fn fce_index(people: Vec<(&str, InstructorOut)>) -> BTreeMap<String, InstructorOut> {
    people
        .into_iter()
        .map(|(key, out)| (key.to_owned(), out))
        .collect()
}

/// 15-122: a lecture Kosbie teaches with a recitation that repeats him, and one a TA teaches; 15-390, a topics
/// course; 21-127 in another department, taught by Kosbie and Mackey.
fn spring() -> TermData {
    term(
        "S27",
        vec![
            course(
                "15122",
                "CS",
                "Principles of Imperative Computation",
                vec![
                    section("1", None, &["David Kosbie"], None),
                    section("A", Some("1"), &["David Kosbie"], None),
                    section("B", Some("1"), &["Ann Tee"], None),
                ],
            ),
            course(
                "15390",
                "CS",
                "Special Topics",
                vec![
                    section("A", None, &["Ann Tee"], Some("Robotics")),
                    section("B", None, &["Ann Tee"], Some("Robotics")),
                ],
            ),
            course(
                "21127",
                "MSC",
                "Concepts of Mathematics",
                vec![section("1", None, &["Mackey John", "David Kosbie"], None)],
            ),
        ],
    )
}

#[test]
fn lists_a_semesters_instructors_as_the_website_indexes_them() {
    let list = term_list(&spring(), None);
    let names: Vec<_> = list.instructors.iter().map(|i| i.name.as_str()).collect();
    // First named first, as the website gives out slugs.
    assert_eq!(names, ["David Kosbie", "Ann Tee", "Mackey John"]);

    let kosbie = &list.instructors[0];
    // The recitation that repeats his lecture doesn't count.
    assert_eq!(kosbie.sections, 2);
    assert_eq!(kosbie.courses, ["15122", "21127"]);
    assert_eq!(kosbie.depts, ["CS", "MSC"]);
    assert!(kosbie.fce.is_none());

    let tee = &list.instructors[1];
    assert_eq!(tee.sections, 3);
    assert_eq!(tee.courses, ["15122", "15390"]);
    assert_eq!(
        tee.topics,
        BTreeMap::from([("15390".to_owned(), vec!["Robotics".to_owned()])])
    );

    assert_eq!(
        list.titles.get("21127").map(String::as_str),
        Some("Concepts of Mathematics")
    );
    assert_eq!(list.titles.len(), 3);
}

#[test]
fn matches_fce_records_by_name_and_rates_the_last_four_semesters() {
    let records = fce_index(vec![
        (
            "david|kosbie",
            fce(
                "David Kosbie",
                &["15112", "15122"],
                &[
                    // Older than the last four: left out.
                    ("F24", Some(1.0), 100.0),
                    ("S25", Some(4.0), 10.0),
                    ("F25", None, 50.0),
                    ("S26", Some(5.0), 30.0),
                    ("F26", Some(4.5), 20.0),
                ],
            ),
        ),
        ("ann|tee", fce("Ann Tee", &["15390"], &[("F26", None, 4.0)])),
    ]);
    let fces = Fces::new(&records);
    let list = term_list(&spring(), Some(&fces));

    let kosbie = list.instructors[0].fce.as_ref().unwrap();
    // (4 * 10 + 5 * 30 + 4.5 * 20) / 60 = 4.666..., to 0.01.
    assert_eq!(kosbie.rating, Some(Num(4.67)));
    assert_eq!(kosbie.courses, ["15112", "15122"]);
    // Matched, though nobody rated their teaching.
    assert_eq!(
        list.instructors[1].fce,
        Some(Evaluated {
            rating: None,
            courses: vec!["15390".into()]
        })
    );
    // "Mackey John" has no record.
    assert!(list.instructors[2].fce.is_none());
}

#[test]
fn leaves_empty_fields_out_of_the_json() {
    let list = term_list(&spring(), None);
    let value = serde_json::to_value(&list).unwrap();
    assert_eq!(
        value["instructors"][0],
        json!({
            "name": "David Kosbie",
            "depts": ["CS", "MSC"],
            "courses": ["15122", "21127"],
            "sections": 2,
        })
    );
    assert_eq!(
        value["instructors"][1]["topics"],
        json!({ "15390": ["Robotics"] })
    );
}

fn teaching() -> TeachingData {
    let taught = |terms: &[(&str, &[&str], usize)], depts: &[&str]| Teaching {
        depts: depts.iter().map(|d| (*d).to_owned()).collect(),
        terms: terms
            .iter()
            .map(|(id, courses, sections)| {
                (
                    (*id).to_owned(),
                    TeachingTerm {
                        courses: courses.iter().map(|c| (*c).to_owned()).collect(),
                        sections: *sections,
                    },
                )
            })
            .collect::<Object<_>>(),
    };
    TeachingData {
        terms: vec!["S27".into(), "F26".into()],
        instructors: [
            (
                "Ann Tee".to_owned(),
                taught(&[("S27", &["15390"], 2)], &["CS"]),
            ),
            (
                "David Kosbie".to_owned(),
                taught(
                    &[("F26", &["15112"], 2), ("S27", &["15122", "15112"], 3)],
                    &["CS"],
                ),
            ),
        ]
        .into_iter()
        .collect(),
    }
}

fn offerings() -> OfferingsData {
    let offering = |title: &str, dept: &str| Offering {
        title: title.into(),
        dept: dept.into(),
        level: "U".into(),
        terms: vec!["S27".into()],
    };
    OfferingsData {
        terms: vec!["S27".into(), "F26".into()],
        courses: [
            (
                "15112".to_owned(),
                offering("Fundamentals of Programming", "CS"),
            ),
            ("15122".to_owned(), offering("Imperative Computation", "CS")),
            ("15390".to_owned(), offering("Special Topics", "CS")),
            (
                "80100".to_owned(),
                offering("Introduction to Philosophy", "PHI"),
            ),
            ("80180".to_owned(), offering("Nature of Language", "PHI")),
        ]
        .into_iter()
        .collect(),
    }
}

fn catalog(id: &str) -> Option<String> {
    match id {
        "80250" => Some("Ancient Philosophy".into()),
        // Says nothing: left out.
        "80999" => Some("80-999".into()),
        _ => None,
    }
}

#[test]
fn lists_everyone_teaching_then_people_only_fces_know() {
    let records = fce_index(vec![
        (
            "david|kosbie",
            fce("KOSBIE, DAVID", &["15112"], &[("F26", Some(4.0), 10.0)]),
        ),
        (
            "mary|shelley",
            fce(
                "SHELLEY, MARY",
                &["80250", "80999"],
                &[("S20", Some(3.5), 8.0)],
            ),
        ),
    ]);
    let fces = Fces::new(&records);
    let list = all_list(&teaching(), &offerings(), catalog, Some(&fces));
    let names: Vec<_> = list.instructors.iter().map(|i| i.name.as_str()).collect();
    assert_eq!(names, ["Ann Tee", "David Kosbie", "Mary Shelley"]);

    let kosbie = &list.instructors[1];
    assert_eq!(kosbie.courses, ["15112", "15122"]);
    assert_eq!(kosbie.sections, 5);
    // Built semesters, newest first.
    assert_eq!(
        kosbie.terms.as_deref(),
        Some(&["S27".to_owned(), "F26".to_owned()][..])
    );
    assert_eq!(kosbie.fce.as_ref().unwrap().rating, Some(Num(4.0)));

    let shelley = &list.instructors[2];
    assert_eq!(shelley.sections, 0);
    assert_eq!(shelley.terms.as_deref(), Some(&[][..]));
    // Departments of courses no semester offers come from courses with the same number prefix.
    assert_eq!(shelley.depts, ["PHI"]);
    assert_eq!(shelley.fce.as_ref().unwrap().rating, Some(Num(3.5)));

    assert_eq!(
        list.titles.get("15122").map(String::as_str),
        Some("Imperative Computation")
    );
    assert_eq!(
        list.titles.get("80250").map(String::as_str),
        Some("Ancient Philosophy")
    );
    assert!(!list.titles.contains_key("80999"));
}

#[test]
fn lists_only_teaching_without_fces() {
    let list = all_list(&teaching(), &offerings(), catalog, None);
    assert_eq!(list.instructors.len(), 2);
    assert!(list.instructors.iter().all(|i| i.fce.is_none()));
}

#[test]
fn names_a_document_per_semester_and_one_for_all() {
    let records = BTreeMap::new();
    let docs = documents(
        &[spring()],
        &teaching(),
        &offerings(),
        catalog,
        Some(&records),
    );
    let names: Vec<_> = docs.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["instructors/S27", "instructors/all"]);
}

#[test]
fn recognizes_titles_that_only_repeat_the_number() {
    assert_eq!(catalog_title("15-122"), None);
    assert_eq!(catalog_title(" 15122 "), None);
    assert_eq!(catalog_title("15-1222"), Some("15-1222"));
    assert_eq!(catalog_title(""), None);
    assert_eq!(catalog_title("Calculus"), Some("Calculus"));
}
