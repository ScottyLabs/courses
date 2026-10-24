use serde_json::{Value, json};

use super::*;

const HEADER: &str = "Year,Sem,College,Dept,Num,Section,Instructor,Course Name,Course Level,Total # Students,\
# Responses,Response Rate,Hrs Per Week,Interest in student learning,Clearly explain course requirements,\
Clear learning objectives & goals,Instructor provides feedback to students to improve,\
Demonstrate importance of subject matter,Explains subject matter of course,Show respect for all students,\
Overall teaching rate,Overall course rate";

/// A row: year, season, course number, section, instructor, enrolled, responses, hours, teaching, course rating.
fn row(fields: [&str; 10]) -> String {
    let [
        year,
        sem,
        num,
        section,
        instructor,
        enrolled,
        responses,
        hours,
        teaching,
        course,
    ] = fields;
    let q1 = if teaching.is_empty() { "" } else { "4.5" };
    format!(
        "{year},{sem},College,DEP,{num},{section},\"{instructor}\",TITLE,Undergraduate,{enrolled},{responses},50,\
         {hours},{q1},,,,,,,{teaching},{course}"
    )
}

/// 15-122 has responses in five terms (S26 co-taught, a mini in S25); 06-222 only a row without responses.
fn fixture() -> String {
    let rows = [
        row([
            "2026",
            "Spring",
            "15122",
            "A",
            "SMITH, JANE",
            "40",
            "20",
            "10",
            "4",
            "4",
        ]),
        row([
            "2026",
            "Spring",
            "15122",
            "A",
            "DOE,  JOHN",
            "40",
            "10",
            "13",
            "3",
            "3.5",
        ]),
        row([
            "2025",
            "Fall",
            "15122",
            "B",
            "Smith, Jane",
            "30",
            "15",
            "8",
            "5",
            "4.5",
        ]),
        row([
            "2025",
            "Fall",
            "15122",
            "C",
            "SMITH, JANE",
            "10",
            "0",
            "",
            "",
            "",
        ]),
        row([
            "2025",
            "Spring",
            "15122",
            "A3",
            "DOE, JOHN",
            "20",
            "10",
            "12",
            "4",
            "4",
        ]),
        row([
            "2024",
            "Fall",
            "15122",
            "A",
            "DOE, JOHN",
            "20",
            "10",
            "9",
            "4",
            "3",
        ]),
        row([
            "2024",
            "Spring",
            "15-122",
            "A",
            "DOE, JOHN",
            "20",
            "10",
            "6",
            "2",
            "2",
        ]),
        row([
            "",
            "Spring",
            "15122",
            "A",
            "DOE, JOHN",
            "20",
            "10",
            "6",
            "2",
            "2",
        ]),
        row([
            "2024",
            "Spring",
            "Email",
            "A",
            "DOE, JOHN",
            "20",
            "10",
            "6",
            "2",
            "2",
        ]),
        row([
            "2025", "Fall", "6222", "A", "LEE, ANN", "5", "0", "", "", "",
        ]),
    ];
    format!("{HEADER}\r\n{}\r\n", rows.join("\r\n"))
}

fn docs() -> BTreeMap<String, Value> {
    let (docs, _) = documents(&fixture()).unwrap();
    docs.into_iter()
        .map(|(name, json)| (name, serde_json::from_slice(&json).unwrap()))
        .collect()
}

#[test]
fn parses_rows_and_drops_unusable_ones() {
    let (rows, dropped) = parse_rows(&fixture());
    assert_eq!(dropped, 2);
    assert_eq!(rows.len(), 8);
    assert_eq!(rows[1].instructor, "DOE, JOHN");
    assert_eq!(rows[4].mini, Some(3));
    assert_eq!(rows[6].course, "15122");
    assert_eq!(rows[7].course, "06222");
    assert_eq!(rows[3].hours, None);
    assert_eq!(rows[0].questions[0], Some(4.5));
}

#[test]
fn summarizes_recent_terms_weighted_by_responses() {
    let docs = docs();
    // Sections first: S26 A averages its two instructors (hours 11), then four recent terms weighted by each
    // section's largest response count. S24 is the previous window (hours 6).
    assert_eq!(
        docs["fce/summary"],
        json!({
            "latest": "S26",
            "recentTerms": 4,
            "courses": {
                "15122": [10, 3.89, 4.08, 55, 4, 5, "S26", 4, 28, 0.25, 1]
            }
        })
    );
}

#[test]
fn keeps_every_row_newest_first_by_section_and_instructor() {
    let docs = docs();
    assert_eq!(docs["fce/courses/index"], json!(["06", "15"]));
    let nulls: [Value; 7] = std::array::from_fn(|_| Value::Null);
    let mut q = nulls.clone();
    q[0] = json!(4.5);
    assert_eq!(
        docs["fce/courses/15"]["15122"],
        json!([
            ["S26", "A", "DOE, JOHN", 40, 10, 13, 3, 3.5, q, null],
            ["S26", "A", "SMITH, JANE", 40, 20, 10, 4, 4, q, null],
            ["F25", "B", "Smith, Jane", 30, 15, 8, 5, 4.5, q, null],
            [
                "F25",
                "C",
                "SMITH, JANE",
                10,
                0,
                null,
                null,
                null,
                nulls,
                null
            ],
            ["S25", "A3", "DOE, JOHN", 20, 10, 12, 4, 4, q, 3],
            ["F24", "A", "DOE, JOHN", 20, 10, 9, 4, 3, q, null],
            ["S24", "A", "DOE, JOHN", 20, 10, 6, 2, 2, q, null],
        ])
    );
    assert_eq!(
        docs["fce/courses/06"],
        json!({ "06222": [["F25", "A", "LEE, ANN", 5, 0, null, null, null, nulls, null]] })
    );
}

#[test]
fn builds_instructor_histories() {
    let docs = docs();
    assert_eq!(
        docs["fce/instructors"],
        json!({
            "instructors": {
                // The mixed-case spelling names her; the row without responses counts as a term taught.
                "jane|smith": {
                    "name": "Jane Smith",
                    "courses": { "15122": [2, 35, 4.43, 4.21, 9.1, "S26", "F25"] },
                    "terms": [["F25", 5, 15], ["S26", 4, 20]]
                },
                // 3.125 rounds half up to 3.13, as Math.round does.
                "john|doe": {
                    "name": "John Doe",
                    "courses": { "15122": [4, 40, 3.25, 3.13, 10, "S26", "S24"] },
                    "terms": [["S24", 2, 10], ["F24", 4, 10], ["S25", 4, 10], ["S26", 3, 10]]
                },
                "ann|lee": {
                    "name": "Ann Lee",
                    "courses": { "06222": [1, 0, null, null, null, "F25", "F25"] },
                    "terms": []
                }
            }
        })
    );
}

#[test]
fn rejects_an_empty_export() {
    assert!(documents(HEADER).is_err());
}

#[test]
fn writes_member_only_documents() {
    use courses_catalog::{FILE_NAME, Lookup, Reader};

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(CSV_FILE), fixture()).unwrap();
    let writer = Writer::create(&dir.path().join(FILE_NAME)).unwrap();
    let summary = write(&writer, dir.path()).unwrap();
    assert_eq!(
        (
            summary.rows,
            summary.dropped,
            summary.courses,
            summary.instructors,
            summary.documents
        ),
        (8, 2, 1, 3, 5)
    );
    assert_eq!(summary.earliest.as_deref(), Some("S24"));
    let reader = Reader::open(&writer.finish().unwrap()).unwrap();
    for name in [
        "fce/summary",
        "fce/courses/index",
        "fce/courses/15",
        "fce/courses/06",
        "fce/instructors",
    ] {
        assert!(
            matches!(reader.document(name, false).unwrap(), Lookup::MemberOnly),
            "{name}"
        );
        assert!(
            matches!(reader.document(name, true).unwrap(), Lookup::Found(_)),
            "{name}"
        );
    }

    // Without an export there's nothing to write.
    let empty = tempfile::tempdir().unwrap();
    let writer = Writer::create(&empty.path().join(FILE_NAME)).unwrap();
    assert_eq!(write(&writer, empty.path()).unwrap(), Summary::default());
}

#[test]
fn builds_are_deterministic() {
    assert_eq!(
        documents(&fixture()).unwrap().0,
        documents(&fixture()).unwrap().0
    );
}
