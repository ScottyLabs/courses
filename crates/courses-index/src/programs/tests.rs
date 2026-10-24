use std::time::{Duration, SystemTime};

use serde_json::{Value, json};

use super::*;

fn course(id: i64, code: &str) -> Value {
    json!({ "id": id, "screen_name": code, "constraints": [
        { "type": "course", "type_string": "", "data": { "course": { "code": code } } }
    ] })
}

fn units(n: f64) -> Value {
    json!({ "type": "minxunits", "type_string": "", "data": { "is_each": false, "units": n } })
}

fn program(
    catalog: i64,
    audit: i64,
    kind: i64,
    name: &str,
    audit_name: &str,
    reqs: Value,
    tree: Value,
) -> Value {
    json!({
        "audit_id": audit,
        "audit_name": audit_name,
        "catalog_id": catalog,
        "program_name": name,
        "program_type": kind,
        "program_reqs": reqs,
        "tree": tree
    })
}

/// A CS major (two versions) with a gen-ed bundle (two versions), a concentration bundle, a minor and a deleted
/// program.
fn exports() -> Vec<(&'static str, Value)> {
    let major_tree = json!({
        "id": 1000,
        "screen_name": "BS in Computer Science",
        "constraints": [units(360.0), {
            "type": "dc", "type_string": "", "data": {
                "for_subrequirement": false, "limit": 0.0, "is_units": false, "scope_program_type": null,
                "scope_programs": [
                    { "id": 20, "name": "SCS Gen Ed" },
                    { "id": 99, "name": "CMU Core Requirements" },
                    { "id": 98, "name": "Some Minor" },
                    { "id": 20, "name": "SCS Gen Ed again" }
                ],
                "specific_subrequirements": []
            }
        }],
        "choices": [course(1001, "15-122")]
    });
    let reqs = json!({ "concentrations": [{ "id": 7, "name": "Machine Learning" }, { "id": 8, "name": "Nothing Matches" }], "department": "Computer Science" });
    let bundle_tree = json!({ "id": 2000, "screen_name": "Gen Ed", "constraints": [units(36.0)], "choices": [course(2001, "76-101")] });
    vec![
        (
            "10/101",
            program(
                10,
                101,
                1,
                "BS in Computer Science",
                "EY2024 Pittsburgh - BS in Computer Science",
                reqs.clone(),
                major_tree.clone(),
            ),
        ),
        (
            "10/100",
            program(
                10,
                100,
                1,
                "BS in Computer Science",
                "EY2020 Pittsburgh - BS in Computer Science",
                reqs,
                major_tree,
            ),
        ),
        (
            "20/200",
            program(
                20,
                200,
                4,
                " SCS General Education ",
                "EY2022 Pittsburgh - SCS Gen Ed",
                json!(null),
                bundle_tree.clone(),
            ),
        ),
        (
            "20/201",
            program(
                20,
                201,
                4,
                " SCS General Education ",
                "EY2025 Pittsburgh - SCS Gen Ed",
                json!(null),
                bundle_tree,
            ),
        ),
        (
            "30/300",
            program(
                30,
                300,
                4,
                "Machine Learning SCS Concentration",
                "Published",
                json!({ "department": "Computer Science" }),
                course(3000, "10-301"),
            ),
        ),
        (
            "40/400",
            program(
                40,
                400,
                1,
                "DELETE Old Major",
                "",
                json!(null),
                course(4000, "15-112"),
            ),
        ),
        (
            "50/500",
            program(
                50,
                500,
                2,
                "Minor in Music (2023)",
                "",
                json!({ "department": "" }),
                course(5000, "57-101"),
            ),
        ),
    ]
}

fn write_exports(root: &Path, user: &str, files: &[(&str, Value)]) {
    for (path, value) in files {
        let path = root
            .join(user)
            .join("programs")
            .join(format!("{path}.json"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_vec(value).unwrap()).unwrap();
    }
}

fn set_mtime(path: &Path, time: SystemTime) {
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(time)
        .unwrap();
}

fn docs(root: &Path) -> BTreeMap<String, Value> {
    let (docs, _) = documents(root).unwrap().unwrap();
    docs.into_iter()
        .map(|(name, json)| (name, serde_json::from_slice(&json).unwrap()))
        .collect()
}

#[test]
fn names_versions_from_stellic_names() {
    assert_eq!(
        version_name("EY2025 Pittsburgh - BS in Computer Science", "BS in CS"),
        "EY2025 Pittsburgh"
    );
    assert_eq!(
        version_name(" Published ", "Minor in Music (2023)"),
        "EY2023"
    );
    assert_eq!(version_name("", "Minor in Music"), "Current");
    assert_eq!(version_name("  - x", "y"), "- x");
    assert_eq!(entry_year_of(&["EY 2019 Doha", "x"]), Some(2019));
    assert_eq!(entry_year_of(&["Current", "BS (2021)"]), Some(2021));
    assert_eq!(campus_of(&["EY2020 Qatar - BS", "x"]), Some("Doha"));
    assert_eq!(campus_of(&["Current", "BS"]), None);
    assert!(
        starts_with_delete("  delete me")
            && starts_with_delete("DELETE")
            && starts_with_delete("Delete-x")
    );
    assert!(!starts_with_delete("DELETED") && !starts_with_delete("Undelete"));
}

#[test]
fn indexes_programs_and_versions() {
    let dir = tempfile::tempdir().unwrap();
    write_exports(dir.path(), "student", &exports());
    let docs = docs(dir.path());
    assert_eq!(
        docs["programs/index"],
        json!({ "programs": [
            { "catalogId": 10, "name": "BS in Computer Science", "kind": "major", "versions": [
                { "auditId": 101, "name": "EY2024 Pittsburgh", "entryYear": 2024, "campus": "Pittsburgh" },
                { "auditId": 100, "name": "EY2020 Pittsburgh", "entryYear": 2020, "campus": "Pittsburgh" }
            ] },
            { "catalogId": 30, "name": "Machine Learning SCS Concentration", "kind": "requirements", "versions": [
                { "auditId": 300, "name": "Current", "entryYear": null, "campus": null }
            ] },
            { "catalogId": 50, "name": "Minor in Music (2023)", "kind": "minor", "versions": [
                { "auditId": 500, "name": "EY2023", "entryYear": 2023, "campus": null }
            ] },
            { "catalogId": 20, "name": "SCS General Education", "kind": "requirements", "versions": [
                { "auditId": 201, "name": "EY2025 Pittsburgh", "entryYear": 2025, "campus": "Pittsburgh" },
                { "auditId": 200, "name": "EY2022 Pittsburgh", "entryYear": 2022, "campus": "Pittsburgh" }
            ] }
        ] })
    );
    let names: Vec<&String> = docs.keys().collect();
    assert_eq!(
        names,
        [
            "programs/100",
            "programs/101",
            "programs/200",
            "programs/201",
            "programs/300",
            "programs/500",
            "programs/index"
        ]
    );
}

#[test]
fn attaches_bundles_and_concentrations_at_the_majors_entry_year() {
    let dir = tempfile::tempdir().unwrap();
    write_exports(dir.path(), "student", &exports());
    let docs = docs(dir.path());
    let major = &docs["programs/101"];
    assert_eq!(
        (
            &major["programName"],
            &major["name"],
            &major["kind"],
            &major["totalUnits"]
        ),
        (
            &json!("BS in Computer Science"),
            &json!("EY2024 Pittsburgh"),
            &json!("major"),
            &json!(360)
        )
    );
    // A major's root unit minimum is its total, not a requirement of the root.
    assert_eq!(major["root"].get("minUnits"), None);
    assert_eq!(
        major["attached"],
        json!([
            { "catalogId": 20, "name": "SCS General Education", "auditId": 200 },
            { "catalogId": 99, "name": "CMU Core Requirements", "auditId": null }
        ])
    );
    assert_eq!(major["doubleCount"][0]["programs"], json!([20, 99, 98, 20]));
    assert_eq!(
        major["concentrations"],
        json!([
            { "id": 7, "name": "Machine Learning", "programs": [
                { "catalogId": 30, "auditId": 300, "name": "Machine Learning SCS Concentration" }
            ] },
            { "id": 8, "name": "Nothing Matches", "programs": [] }
        ])
    );
    // The 2020 version predates every gen-ed version, so it gets the newest.
    assert_eq!(docs["programs/100"]["attached"][0]["auditId"], 201);
    // Bundles keep their root minimum and report it as their total.
    assert_eq!(docs["programs/200"]["root"]["minUnits"], 36);
    assert_eq!(docs["programs/200"]["totalUnits"], 36);
    assert_eq!(docs["programs/300"]["totalUnits"], Value::Null);
}

#[test]
fn reads_the_newest_export_of_each_version() {
    let dir = tempfile::tempdir().unwrap();
    let mut newer = exports().remove(4);
    newer.1["program_name"] = json!("Machine Learning SCS Concentration (new)");
    write_exports(dir.path(), "a", &exports());
    write_exports(dir.path(), "b", &[newer]);
    let old = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    set_mtime(&dir.path().join("a/programs/30/300.json"), old);
    set_mtime(
        &dir.path().join("b/programs/30/300.json"),
        old + Duration::from_secs(60),
    );

    let raws = read_exports(dir.path()).unwrap();
    let ids: Vec<i64> = raws
        .iter()
        .map(|r| r["audit_id"].as_i64().unwrap())
        .collect();
    // Sorted by directory and file name, without the deleted program; the newer export keeps the older's place.
    assert_eq!(ids, [100, 101, 200, 201, 300, 500]);
    assert_eq!(
        raws[4]["program_name"],
        "Machine Learning SCS Concentration (new)"
    );

    // An older duplicate doesn't replace a newer one.
    set_mtime(
        &dir.path().join("b/programs/30/300.json"),
        old - Duration::from_secs(60),
    );
    let raws = read_exports(dir.path()).unwrap();
    assert_eq!(
        raws[4]["program_name"],
        "Machine Learning SCS Concentration"
    );
}

#[test]
fn writes_member_only_documents() {
    use courses_catalog::{FILE_NAME, Lookup, Reader};

    let dir = tempfile::tempdir().unwrap();
    write_exports(dir.path(), "student", &exports());
    let writer = Writer::create(&dir.path().join(FILE_NAME)).unwrap();
    let summary = write(&writer, dir.path()).unwrap();
    assert_eq!(
        (summary.programs, summary.versions, summary.documents),
        (4, 6, 7)
    );
    assert_eq!(summary.kinds["requirements"], 2);
    let reader = Reader::open(&writer.finish().unwrap()).unwrap();
    for name in ["programs/index", "programs/101"] {
        assert!(matches!(
            reader.document(name, false).unwrap(),
            Lookup::MemberOnly
        ));
        assert!(matches!(
            reader.document(name, true).unwrap(),
            Lookup::Found(_)
        ));
    }

    // Without exports there's nothing to write.
    let empty = tempfile::tempdir().unwrap();
    let writer = Writer::create(&empty.path().join(FILE_NAME)).unwrap();
    assert_eq!(
        write(&writer, &empty.path().join("missing")).unwrap(),
        Summary::default()
    );
}

#[test]
fn builds_are_deterministic() {
    let raws: Vec<Value> = exports()
        .into_iter()
        .map(|(_, v)| v)
        .filter(|v| !is_deleted(Js::new(v)))
        .collect();
    let a = build(&raws).unwrap().documents().unwrap();
    let b = build(&raws).unwrap().documents().unwrap();
    assert_eq!(a, b);
}
