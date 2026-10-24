//! Compares the FCE and program documents with the files Courseweave's build scripts wrote from the same inputs.
//! Ignored by default: the inputs are CMU-internal and one student's Stellic scrape, which aren't committed.
//!
//! ```sh
//! COURSEWEAVE_DATA=~/Projects/cmu-course-planner/public/data \
//! EXPORTED_ROOT=~/Projects/scottylabs/courses/exported \
//!   cargo test --release -p courses-index --test courseweave_parity -- --ignored --nocapture
//! ```
//!
//! `FCE_DATA_DIR` overrides the directory holding `fces.csv` (the repository's `data/`). Documents are compared as
//! parsed JSON, so key order and number spelling don't matter. Courseweave's wall-clock `generatedAt` is ignored.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::Value;

fn env_dir(name: &str) -> PathBuf {
    let value =
        std::env::var_os(name).unwrap_or_else(|| panic!("set {name}; see this file's docs"));
    PathBuf::from(value)
}

fn read_json(path: &Path) -> Value {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    let mut value: Value = serde_json::from_slice(&bytes).unwrap();
    if let Some(obj) = value.as_object_mut() {
        obj.remove("generatedAt");
    }
    value
}

/// The first path at which two JSON values differ.
fn first_difference(path: &str, ours: &Value, theirs: &Value) -> Option<String> {
    match (ours, theirs) {
        (Value::Object(a), Value::Object(b)) => {
            let keys: BTreeSet<&String> = a.keys().chain(b.keys()).collect();
            keys.into_iter().find_map(|k| match (a.get(k), b.get(k)) {
                (Some(x), Some(y)) => first_difference(&format!("{path}.{k}"), x, y),
                (x, y) => Some(format!("{path}.{k}: ours {x:?}, Courseweave's {y:?}")),
            })
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => a
            .iter()
            .zip(b)
            .enumerate()
            .find_map(|(i, (x, y))| first_difference(&format!("{path}[{i}]"), x, y)),
        _ if ours == theirs => None,
        _ => Some(format!(
            "{path}: ours {}, Courseweave's {}",
            truncate(ours),
            truncate(theirs)
        )),
    }
}

fn truncate(v: &Value) -> String {
    let s = v.to_string();
    if s.len() > 300 {
        format!("{}...", &s[..300])
    } else {
        s
    }
}

/// Compares each document with Courseweave's file for it, returning the differences.
fn compare(
    docs: &[(String, Vec<u8>)],
    courseweave_file: impl Fn(&str) -> Option<PathBuf>,
) -> Vec<String> {
    let mut differences = Vec::new();
    for (name, json) in docs {
        let Some(path) = courseweave_file(name) else {
            continue;
        };
        let ours: Value = serde_json::from_slice(json).unwrap();
        if let Some(diff) = first_difference(name, &ours, &read_json(&path)) {
            differences.push(diff);
        }
    }
    differences
}

fn file_stems(dir: &Path) -> BTreeSet<String> {
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| {
            let name = e.unwrap().file_name().into_string().unwrap();
            name.strip_suffix(".json").map(str::to_owned)
        })
        .collect()
}

fn report(kind: &str, docs: &[(String, Vec<u8>)], differences: &[String]) {
    let bytes: usize = docs.iter().map(|(_, j)| j.len()).sum();
    let gzip: usize = docs
        .iter()
        .map(|(_, j)| courses_catalog::gzip(j).unwrap().len())
        .sum();
    println!(
        "{kind}: {} documents, {:.1} MB JSON, {:.1} MB gzipped, {} differ from Courseweave",
        docs.len(),
        bytes as f64 / 1e6,
        gzip as f64 / 1e6,
        differences.len()
    );
    for d in differences.iter().take(20) {
        println!("  {d}");
    }
}

#[test]
#[ignore = "needs Courseweave's built files and the CMU-internal inputs"]
fn fce_documents_match_courseweave() {
    let courseweave = env_dir("COURSEWEAVE_DATA").join("fce");
    let data_dir = std::env::var_os("FCE_DATA_DIR").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data"),
        PathBuf::from,
    );
    let csv = std::fs::read_to_string(data_dir.join("fces.csv")).unwrap();

    let start = Instant::now();
    let (docs, summary) = courses_index::fce::documents(&csv).unwrap();
    println!("fce: built in {:.2?}: {summary:?}", start.elapsed());

    // Every chunk Courseweave wrote, and no others.
    let chunks: BTreeSet<String> = docs
        .iter()
        .filter_map(|(n, _)| n.strip_prefix("fce/courses/"))
        .filter(|n| *n != "index")
        .map(str::to_owned)
        .collect();
    assert_eq!(chunks, file_stems(&courseweave.join("courses")));
    let index = &docs
        .iter()
        .find(|(n, _)| n == "fce/courses/index")
        .unwrap()
        .1;
    assert_eq!(
        serde_json::from_slice::<Vec<String>>(index).unwrap(),
        chunks.iter().cloned().collect::<Vec<_>>()
    );

    let differences = compare(&docs, |name| match name {
        "fce/summary" => Some(courseweave.join("summary.json")),
        "fce/instructors" => Some(courseweave.join("instructors.json")),
        "fce/courses/index" => None,
        _ => Some(courseweave.join(format!(
            "courses/{}.json",
            name.strip_prefix("fce/courses/")?
        ))),
    });
    report("fce", &docs, &differences);
    assert!(differences.is_empty());
}
