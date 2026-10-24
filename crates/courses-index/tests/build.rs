//! Builds a catalog from a tiny synthetic set of inputs: two builds are byte-identical, and each document reaches the
//! right audience.

use std::io::Write as _;
use std::path::Path;

use courses_catalog::{Lookup, Reader, gunzip};
use courses_index::pipeline::{Options, build};
use serde_json::{Value, json};

fn put(root: &Path, path: &str, bytes: &[u8]) {
    let full = root.join(path);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(full, bytes).unwrap();
}

fn gz(text: &str) -> Vec<u8> {
    let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    e.write_all(text.as_bytes()).unwrap();
    e.finish().unwrap()
}

fn search_page(rows: &[[&str; 10]]) -> String {
    let cols = [
        "course",
        "title",
        "units",
        "section",
        "mini",
        "days",
        "begin",
        "end",
        "location",
        "deliverymode",
    ];
    let head: String = cols
        .iter()
        .map(|c| format!(r#"<th class="col-{c}">{c}</th>"#))
        .collect();
    let body: String = rows
        .iter()
        .map(|r| {
            format!(
                "<tr>{}</tr>",
                r.iter()
                    .map(|c| format!("<td>{c}</td>"))
                    .collect::<String>()
            )
        })
        .collect();
    format!(
        r#"<h4 class="department-title">COMPUTER SCIENCE</h4><table><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table>"#
    )
}

const PGH: &str = "Pittsburgh, Pennsylvania";
const IPE: &str = "In-person Expectation";

fn inputs(root: &Path) {
    let soc = search_page(&[
        [
            "15122",
            "Principles of Imperative Computation",
            "12",
            "Lec 1",
            "",
            "TR",
            "09:30AM",
            "10:50AM",
            PGH,
            IPE,
        ],
        ["", "", "", "A", "", "F", "09:00AM", "09:50AM", PGH, IPE],
        [
            "15151",
            "Mathematical Foundations for Computer Science",
            "12",
            "Lec",
            "",
            "MWF",
            "10:00AM",
            "10:50AM",
            PGH,
            IPE,
        ],
    ]);
    put(root, "exported/soc/F26/search-U.html.gz", &gz(&soc));
    put(root, "exported/soc/F26/search-G.html.gz", &gz(""));
    put(
        root,
        "exported/soc/F26/meta.json",
        json!({"term": "F26", "fetchedAt": "2026-10-23T20:37:10.013Z",
               "departments": [{"code": "CS", "name": "Computer Science", "prefix": "15"}]})
        .to_string()
        .as_bytes(),
    );
    put(
        root,
        "exported/soc/F26/extras.json",
        br#"{"15122": {"reservations": {"A": ["Some reservations are for Students in CS"]}}, "15151": {"permission": true}}"#,
    );
    put(
        root,
        "exported/soc/details.json",
        json!({"fetchedAt": "2026-10-21T23:31:21.393Z", "missing": [], "courses": {
            "15122": {"semester": "F26", "title": "Principles of Imperative Computation",
                      "description": "Pointers and invariants.", "prereqText": "15112", "units": 12},
            "02250": {"semester": "F26", "title": "Evolution", "prereqText": "Junior standing", "units": null}
        }})
        .to_string()
        .as_bytes(),
    );
    put(
        root,
        "exported/alice/courses_history/15122/info.json",
        json!({"name": "Principles of Imperative Computation", "units": 12.0, "min_units": 12.0,
               "prereqs": {"req_obj": {"constraints": [{"type": "mingrade", "data": {"grade": "C"}},
                                                       {"type": "course", "data": {"course": {"code": "15-112"}}}]}},
               "co_reqs": [{"code": "21-127"}], "custom_fields": {"goals": "Write correct code"}})
        .to_string()
        .as_bytes(),
    );
    put(
        root,
        "exported/alice/courses_history/15122/ly1_sm1.json",
        json!({"data_list": [{"course": {"code": "15-122", "name": "Principles of Imperative Computation", "units": 12.0},
            "section_groups": [
              {"group_name": "Lecture", "sections": [{"name": "1", "type": {"short_name": "L"},
                "instructors": [{"first_name": "Iliano", "last_name": "Cervesato"}],
                "raw_timings": [{"day": "tuesday", "start": "09:30 AM", "end": "10:50 AM", "building": "GHC", "room": "4401"},
                                {"day": "thursday", "start": "09:30 AM", "end": "10:50 AM", "building": "GHC", "room": "4401"}],
                "max_enrollment": 244, "start_date": "2026-08-24", "end_date": "2026-12-05", "campus": 1}]},
              {"group_name": "Recitation", "sections": [{"name": "A", "parent": {"name": "1"},
                "instructors": [{"first_name": "Ta", "last_name": "One"}],
                "raw_timings": [{"day": "friday", "start": "09:00 AM", "end": "09:50 AM", "building": "PH", "room": "226C"}],
                "max_enrollment": 25, "start_date": "2026-08-24", "end_date": "2026-12-05", "campus": 1}]}]}]})
        .to_string()
        .as_bytes(),
    );
    put(
        root,
        "exported/schedules/S27.csv",
        b"num,title,units,section,lecture,days,begin,end,bldg,room,mode,level,dept,instructors\n\
          15122,Principles of Imperative Computation,12,1,1,TR,09:30AM,10:50AM,GHC,4401,IPE,U,CS,Iliano Cervesato\n\
          15122,Principles of Imperative Computation,12,A3,,F,09:00AM,09:50AM,PH,226C,IPE,U,CS,\n",
    );
    put(root, "exported/syllabi/F25/CS/15122-1.pdf", b"syllabus");
    put(root, "exported/syllabi/F25/CS/15122-2.pdf", b"syllabus");
    put(root, "exported/syllabi/F25/CS/15-151-A.docx", b"other");
    let term = |id: &str, name: &str, season: &str, year: i64, sources: Value| {
        json!({"id": id, "name": name, "season": season, "year": year,
               "minis": [{"number": 3, "name": "Mini 3", "start": "2027-01-19", "end": "2027-03-05"}],
               "classesBegin": "2026-08-24", "classesEnd": "2026-12-04", "noClasses": [], "sources": sources})
    };
    put(
        root,
        "data/terms.json",
        serde_json::to_vec_pretty(&json!([
            term("S27", "Spring 2027", "spring", 2027, json!([{"type": "csv", "path": "schedules/S27.csv"}])),
            term("F26", "Fall 2026", "fall", 2026, json!([
                {"type": "soc", "term": "F26"},
                {"type": "stellic", "role": "supplement", "lyear": 1, "sem": 1, "addMissingSections": false}
            ])),
        ]))
        .unwrap()
        .as_slice(),
    );
}

fn options(root: &Path, output: &str) -> Options {
    Options {
        exported: root.join("exported"),
        data: root.join("data"),
        output: root.join(output),
        terms: Vec::new(),
        stellic_user: None,
        dump: None,
        source_date_epoch: None,
    }
}

fn document(reader: &Reader, name: &str, member: bool) -> Option<Value> {
    match reader.document(name, member).unwrap() {
        Lookup::Found(doc) => Some(serde_json::from_slice(&gunzip(&doc.gzip).unwrap()).unwrap()),
        Lookup::MemberOnly | Lookup::Missing => None,
    }
}

#[test]
fn builds_are_byte_identical() {
    let dir = tempfile::tempdir().unwrap();
    inputs(dir.path());
    let a = build(&options(dir.path(), "a/catalog.sqlite")).unwrap();
    let b = build(&options(dir.path(), "b/catalog.sqlite")).unwrap();
    assert_eq!(a.version, b.version);
    assert_eq!(
        std::fs::read(a.output).unwrap(),
        std::fs::read(b.output).unwrap()
    );
}

#[test]
fn documents_reach_the_right_audience() {
    let dir = tempfile::tempdir().unwrap();
    inputs(dir.path());
    let summary = build(&options(dir.path(), "catalog.sqlite")).unwrap();
    assert_eq!(
        summary.terms,
        [("S27".to_owned(), 1, 2), ("F26".to_owned(), 2, 3)]
    );
    let reader = Reader::open(&summary.output).unwrap();

    let terms = document(&reader, "terms", false).unwrap();
    let ids: Vec<&str> = terms
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["S27", "F26"]);
    assert_eq!(terms[1]["sources"], json!(["soc", "stellic"]));

    let member = document(&reader, "term/F26", true).unwrap();
    assert_eq!(member["generatedAt"], "2026-10-23T20:37:10.013Z");
    let c = &member["courses"][0];
    assert_eq!(c["instructors"], json!(["Iliano Cervesato", "Ta One"]));
    assert_eq!(c["sections"][0]["meetings"][0]["building"], "GHC");
    assert_eq!(c["sections"][0]["capacity"], 244);
    assert_eq!(c["sections"][1]["parent"], "1");
    assert_eq!(
        c["sections"][1]["reservations"],
        json!(["Some reservations are for Students in CS"])
    );
    assert_eq!(member["courses"][1]["permissionRequired"], true);
    let public = document(&reader, "term/F26", false).unwrap();
    let c = &public["courses"][0];
    assert_eq!(c["instructors"], json!([]));
    assert_eq!(c["sections"][0]["instructors"], json!([]));
    assert_eq!(c["sections"][0]["meetings"][0]["building"], Value::Null);
    assert_eq!(c["sections"][0]["meetings"][0]["room"], Value::Null);
    assert_eq!(c["sections"][0]["meetings"][0]["start"], 570);

    let index = document(&reader, "catalog/index", false).unwrap();
    assert_eq!(index["generatedAt"], "2026-10-21T23:31:21.393Z");
    assert_eq!(
        index["courses"]["15122"]["prereqs"],
        json!({"type": "course", "id": "15112", "minGrade": "C"})
    );
    assert_eq!(
        index["courses"]["02250"]["prereqs"],
        json!({"type": "text", "text": "Junior standing"})
    );
    let details = document(&reader, "catalog/details/15", false).unwrap();
    assert_eq!(details["15122"]["description"], "Pointers and invariants.");

    let offerings = document(&reader, "offerings", false).unwrap();
    assert_eq!(
        offerings["courses"]["15122"]["terms"],
        json!(["S27", "F26"])
    );

    // Member-only documents.
    for name in ["teaching", "syllabi/15"] {
        assert!(
            matches!(reader.document(name, false).unwrap(), Lookup::MemberOnly),
            "{name}"
        );
    }
    let teaching = document(&reader, "teaching", true).unwrap();
    assert_eq!(
        teaching["instructors"]["Ta One"]["terms"]["F26"],
        json!({"courses": ["15122"], "sections": 1})
    );
    let syllabi = document(&reader, "syllabi/15", true).unwrap();
    assert_eq!(syllabi["15122"][0]["sections"], json!(["1", "2"]));
    assert_eq!(syllabi["15151"][0]["type"], "docx");

    // Relational tables hold public fields only.
    let conn = reader.connection();
    let count = |sql: &str| conn.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap();
    assert_eq!(count("SELECT count(*) FROM terms"), 2);
    assert_eq!(count("SELECT count(*) FROM courses"), 3);
    assert_eq!(
        count("SELECT count(*) FROM sections WHERE term_id = 'F26'"),
        3
    );
    assert_eq!(
        count("SELECT start FROM meetings WHERE term_id = 'F26' AND section = 'A'"),
        540
    );
    let prereq: String = conn
        .query_row(
            "SELECT prerequisites FROM courses WHERE id = '15122'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(prereq, "15-112 (min C)");
}
