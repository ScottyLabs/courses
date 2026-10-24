use serde_json::{Value, json};

use super::*;

fn name_of(id: Js) -> String {
    format!("Program {}", id.string())
}

fn run(raw: &Value, is_root: bool) -> (Value, Vec<DoubleCountRule>) {
    let program_name = name_of;
    let mut ctx = Ctx {
        double_count: Vec::new(),
        program_name: &program_name,
        unknown_patterns: 0,
    };
    let node = normalize(Js::new(raw), &mut ctx, is_root);
    (serde_json::to_value(&node).unwrap(), ctx.double_count)
}

fn node(raw: Value) -> Value {
    run(&raw, false).0
}

fn constraint(ty: &str, data: Value) -> Value {
    json!({ "type": ty, "type_string": "", "data": data })
}

fn course(code: &str) -> Value {
    json!({ "id": 2, "screen_name": code, "constraints": [constraint("course", json!({ "course": { "code": code } }))] })
}

#[test]
fn reads_course_codes_and_patterns() {
    assert_eq!(course_id(" 15-122 ").as_deref(), Some("15122"));
    assert_eq!(course_id("15122").as_deref(), Some("15122"));
    assert_eq!(course_id("15-12"), None);
    assert_eq!(pattern_source("15-3**").as_deref(), Some(r"^153\d\d$"));
    assert_eq!(pattern_source("XX-4").as_deref(), Some(r"^\d\d4\d\d$"));
    assert_eq!(pattern_source("99-x9x").as_deref(), Some(r"^99\d9\d$"));
    assert_eq!(pattern_source("15122"), None);

    let mut set = PoolSet::default();
    assert!(range_to_set("15-200", "15-3**", &mut set));
    assert!(range_to_set("**-100", "**-4", &mut set));
    assert!(!range_to_set("nope", "15-100", &mut set));
    assert_eq!(
        serde_json::to_value(&set).unwrap(),
        json!({ "ranges": [["15200", "15399"]], "anyDeptRanges": [[100, 499]] })
    );
}

#[test]
fn turns_groups_and_course_leaves_into_rules() {
    let raw = json!({
        "id": 1.0,
        "screen_name": "  Core  ",
        "constraints": [
            { "type": "anyxof", "type_string": "Fulfill 2", "data": { "is_and": false, "x": 2 } },
            { "type": "minxunits", "type_string": "", "data": { "is_each": false, "units": 9.0 } },
            { "type": "minxunits", "type_string": "", "data": { "is_each": false, "units": 36.0 } }
        ],
        "choices": [course("15-122"), course("15-150"), course("15-210")]
    });
    let n = node(raw);
    assert_eq!(n["id"], json!(1));
    assert_eq!(n["name"], "Core");
    assert_eq!(n["rule"], json!({ "kind": "atLeast", "n": 2 }));
    assert_eq!(n["minUnits"], json!(36));
    assert_eq!(n["notes"], json!([]));
    assert_eq!(
        n["children"][0],
        json!({ "id": 2, "name": "15-122", "rule": { "kind": "all" }, "children": [], "course": { "id": "15122" }, "notes": [] })
    );

    let any = json!({ "id": 1, "screen_name": "x", "constraints": [constraint("anyxof", json!({ "x": 1 }))], "choices": [course("15-122"), course("15-150")] });
    assert_eq!(node(any)["rule"], json!({ "kind": "any" }));
    let all = json!({ "id": 1, "screen_name": "x", "constraints": [constraint("anyxof", json!({ "x": 5 }))], "choices": [course("15-122")] });
    assert_eq!(node(all)["rule"], json!({ "kind": "all" }));
}

#[test]
fn keeps_a_topics_course_section() {
    let raw = json!({ "id": 3, "screen_name": "Topics", "constraints": [
        constraint("coursesection", json!({ "course": { "code": "15-859" }, "section": "A" })),
        constraint("course", json!({ "course": { "code": "15-859" } }))
    ] });
    assert_eq!(
        node(raw)["course"],
        json!({ "id": "15859", "section": "A" })
    );
}

#[test]
fn folds_course_set_constraints_into_pools_caps_and_exclusions() {
    let sets = json!([
        { "courses": ["15-122", "15-122", "15-3**"], "code_ranges": [["15-400", "15-499"]], "code_patterns": ["21-***", "**-***"] },
        { "schools": [{ "name": "Mellon College of Science" }], "timeframe": { "start_term": 1, "start_year": 2022, "end_term": 2, "end_year": 2025 } },
        { "courses": ["76-101"], "is_excluded": true },
        { "courses": [] }
    ]);
    let raw = json!({ "id": 4, "screen_name": "Electives", "constraints": [
        constraint("xfromcourseset", json!({ "conditional_course_sets": sets, "x": 2.0, "is_units": false, "is_exact": true })),
        constraint("xfromcourseset", json!({ "conditional_course_sets": [{ "courses": ["15-112"] }], "x": 1, "is_atmost": true, "inverse": true })),
        constraint("xfromcourseset", json!({ "conditional_course_sets": [{ "courses": ["99-101"] }], "x": 0, "inverse": true })),
        constraint("notcountcourseset", json!({ "courses": ["15-104"], "code_ranges": [], "code_patterns": ["70-***"] })),
        { "type": "xfromcourseset", "type_string": "Take  2 courses\n", "data": { "conditional_course_sets": [], "x": 2 } }
    ] });
    let n = node(raw);
    assert_eq!(
        n["pool"],
        json!([{
            "sets": [
                { "courses": ["15122"], "patterns": [r"^153\d\d$"], "ranges": [["15400", "15499"]], "depts": ["21"], "any": true },
                { "depts": ["03", "09", "21", "33", "38"], "from": "F22", "to": "S25" }
            ],
            "units": false, "min": 2, "max": 2
        }])
    );
    assert_eq!(
        n["caps"],
        json!([{ "sets": [{ "courses": ["15112"] }], "units": false, "max": 1, "inverse": true }])
    );
    assert_eq!(
        n["exclude"],
        json!([{ "courses": ["99101"] }, { "courses": ["15104"], "depts": ["70"] }])
    );
    // A rule with nothing to evaluate is kept as a note.
    assert_eq!(n["notes"], json!(["Take 2 courses"]));
}

#[test]
fn reads_department_ranges_and_all_course_pools() {
    let raw = json!({ "id": 5, "screen_name": "Depts", "constraints": [
        constraint("xfromdepts", json!({ "depts": [{ "code": "15" }, { "code": 21 }, { "code": "1" }], "schools": [], "additional_courses": ["02-251"], "x": 2.0, "is_units": false, "all_from_same_choice": true })),
        constraint("coderange", json!({ "code_start": "80-100", "code_end": "80-199", "additional_courses": [], "x": 1 })),
        constraint("xfromallcourses", json!({ "x": null }))
    ] });
    assert_eq!(
        node(raw)["pool"],
        json!([
            { "sets": [{ "depts": ["15", "21"], "courses": ["02251"] }], "units": false, "min": 2, "sameDept": true },
            { "sets": [{ "ranges": [["80100", "80199"]] }], "units": false, "min": 1 },
            { "sets": [{ "any": true }], "units": false }
        ])
    );
}

#[test]
fn reads_grades_windows_repeats_and_visibility() {
    let raw = json!({ "id": 6, "screen_name": "Seminar", "constraints": [
        constraint("mingrade", json!({ "grade": "C" })),
        constraint("cancountungraded", json!({ "usems": [] })),
        constraint("notcountbeforeusem", json!({ "usem": { "semester": 3, "year": 2005 }, "end_usem": null })),
        constraint("repeatxcourses", json!({ "courses": [{ "code": "99-101" }, { "code": "bad" }], "x": 3, "same_course": false })),
        constraint("alwayshide", json!({ "hide_unless_attempted": true }))
    ] });
    let n = node(raw);
    assert_eq!(n["minGrade"], "C");
    assert_eq!(n["allowPassFail"], true);
    assert_eq!(n["window"], json!({ "from": "M05" }));
    assert_eq!(
        n["repeat"],
        json!({ "courses": ["99101"], "times": 3, "sameCourse": false })
    );
    assert_eq!(n["onlyIfAttempted"], true);
    // A repeat leaf has no pool and isn't a manual check.
    assert_eq!(n.get("pool"), None);
    assert_eq!(n.get("manual"), None);
}

#[test]
fn marks_manual_checks_and_placeholders() {
    let gpa = json!({ "id": 7, "screen_name": "GPA", "constraints": [
        { "type": "requirementgpa", "type_string": "Minimum GPA of 2.0", "data": { "min": 2.0 } },
        { "type": "customgpa", "type_string": "silent", "data": {} },
        { "type": "noncourse", "type_string": "Advisor approval", "data": {} }
    ] });
    let n = node(gpa);
    assert_eq!(n["manual"], true);
    assert_eq!(
        n["notes"],
        json!(["Minimum GPA of 2.0", "Advisor approval"])
    );

    let empty = json!({ "id": 8, "screen_name": "Placeholder" });
    assert_eq!(node(empty)["manual"], true);

    let units = json!({ "id": 9, "screen_name": "Free electives", "constraints": [constraint("minxcourses", json!({ "x": 3 }))] });
    let n = node(units);
    assert_eq!(n["minCourses"], 3);
    assert_eq!(
        n["pool"],
        json!([{ "sets": [{ "any": true }], "units": false }])
    );

    let others = json!({ "id": 10, "screen_name": "Others", "constraints": [
        { "type": "staticonly", "type_string": "Others may count", "data": { "name": "OTHERS_MAY_COUNT" } }
    ], "choices": [course("15-122")] });
    let n = node(others);
    assert_eq!(n.get("manual"), None);
    assert_eq!(n["notes"], json!(["Others may count"]));
}

#[test]
fn reads_double_counting_at_the_root_and_sharing_below_it() {
    let dc = |for_sub: bool| {
        constraint(
            "dc",
            json!({
                "for_subrequirement": for_sub,
                "limit": 2.0,
                "is_units": false,
                "scope_programs": [{ "id": 37, "name": "SCS Gen Ed" }, { "id": 40 }],
                "scope_program_type": 2,
                "specific_subrequirements": [{ "requirements": [11, 12], "limit": 1, "is_units": true }]
            }),
        )
    };
    let raw = json!({ "id": 1, "screen_name": "Root", "constraints": [dc(false), dc(true)] });
    let (n, double_count) = run(&raw, true);
    assert_eq!(n["shareWithin"], json!({ "limit": 2, "units": false }));
    assert_eq!(
        serde_json::to_value(&double_count).unwrap(),
        json!([{
            "programs": [37, 40],
            "kinds": ["minor"],
            "programNames": ["SCS Gen Ed", "Program 40"],
            "limit": 2,
            "units": false,
            "exceptions": [{ "nodes": [11, 12], "limit": 1, "units": true }]
        }])
    );
    // Below the root, every double-count limit is about sharing within the program.
    let (_, double_count) = run(&raw, false);
    assert!(double_count.is_empty());
}

#[test]
fn reads_program_dependent_choices() {
    let raw = json!({ "id": 1, "screen_name": "Root", "constraints": [
        { "type": "programdependentanyxof", "type_string": "Depends", "data": { "scope_programs": [{ "id": 5 }], "subrequirements": [3], "is_include": true, "is_inverse": false, "concentrations": [], "student_tags": [] } },
        { "type": "programdependentanyxof", "type_string": "Depends", "data": { "scope_programs": [{ "id": 6 }], "subrequirements": [4], "concentrations": [{ "id": 1 }] } }
    ], "choices": [course("15-122")] });
    let n = node(raw);
    assert_eq!(
        n["programDependent"],
        json!([{ "programs": [5], "children": [3], "include": true }])
    );
    assert_eq!(n["notes"], json!(["Depends"]));
}
