use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use chrono::NaiveTime;
use indexmap::IndexMap;
use serde_json::{Value, json};
use walkdir::WalkDir;

use super::Catalog;

type Rows = BTreeMap<String, Value>;
type SectionRows = IndexMap<String, Value>;
type Sections = IndexMap<(String, String), SectionRows>;

#[derive(Default)]
pub(super) struct Audit {
    pub section_dates: BTreeSet<String>,
    pub offering_years: BTreeSet<i64>,
    pub warnings: Vec<String>,
}

fn read(path: &Path) -> Result<Value> {
    serde_json::from_slice(&fs::read(path)?).with_context(|| path.display().to_string())
}

fn files(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(vec![]);
    }
    WalkDir::new(root)
        .min_depth(2)
        .max_depth(2)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
        .filter_map(|entry| match entry {
            Ok(entry)
                if entry.file_type().is_file()
                    && entry.path().extension().is_some_and(|ext| ext == "json") =>
            {
                Some(Ok(entry.into_path()))
            }
            Ok(_) => None,
            Err(error) => Some(Err(error.into())),
        })
        .collect()
}

fn array(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

fn string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        value => value.to_string(),
    }
}

fn integer(value: &Value) -> Option<i64> {
    value.as_i64().or_else(|| value.as_str()?.parse().ok())
}

fn text_or<'a>(value: &'a Value, fallback: &'a str) -> &'a str {
    value
        .as_str()
        .filter(|text| !text.is_empty())
        .unwrap_or(fallback)
}

fn default(value: &Value, fallback: Value) -> Value {
    if value.is_null() {
        fallback
    } else {
        value.clone()
    }
}

fn level(code: &str) -> String {
    code.get(code.len().saturating_sub(3)..)
        .and_then(|suffix| suffix.parse::<u32>().ok())
        .map_or(0, |number| number / 100 * 100)
        .to_string()
}

fn department(code: &str) -> String {
    code.chars().take(2).collect()
}

fn term(year: &Value, season: &Value) -> Option<String> {
    let prefix = match (season.as_i64(), season.as_str()) {
        (Some(1), _) | (_, Some("fall")) => 'F',
        (Some(2), _) | (_, Some("spring")) => 'S',
        (Some(3), _) | (_, Some("summer")) => 'M',
        _ => return None,
    };
    Some(format!("{prefix}{:02}", integer(year)?.rem_euclid(100)))
}

fn instructor(row: &Value, code: &str, instructors: &mut Rows) -> String {
    let id = string(&row["id"]);
    let name = join_fields(row, ["first_name", "last_name"]);
    let entry = instructors
        .entry(id.clone())
        .or_insert_with(|| json!({"id": id, "name": name, "courses": []}));
    let courses = entry["courses"].as_array_mut().expect("instructor courses");
    if !courses.iter().any(|course| course.as_str() == Some(code)) {
        courses.push(json!(code));
    }
    name
}

fn join_fields(row: &Value, keys: [&str; 2]) -> String {
    keys.into_iter()
        .map(|key| string(&row[key]))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn clock(value: &Value) -> String {
    NaiveTime::parse_from_str(
        &string(value).replace(' ', "").to_ascii_uppercase(),
        "%I:%M%p",
    )
    .map(|time| time.format("%H:%M").to_string())
    .unwrap_or_default()
}

fn meeting(row: &Value, days: Vec<char>, start: String, end: String) -> Value {
    json!({"days": days, "starts_at": start, "ends_at": end, "location": join_fields(row, ["building", "room"])})
}

fn stellic_sections(
    paths: &[PathBuf],
    courses: &Rows,
    sections: &mut Sections,
    instructors: &mut Rows,
    audit: &mut Audit,
) -> Result<()> {
    let mut replaced = BTreeSet::new();
    for path in paths.iter().filter(|path| {
        path.file_name()
            .is_some_and(|name| name.to_string_lossy().contains("sm"))
    }) {
        let export = read(path)?;
        if export["success"] == false {
            audit.warnings.push(path.display().to_string());
            tracing::warn!(file = %path.display(), "unsuccessful Stellic export");
            continue;
        }
        for row in array(&export["data_list"]) {
            let code = string(&row["course"]["code"]);
            if !courses.contains_key(&code) {
                continue;
            }
            for group in array(&row["section_groups"]) {
                for section in array(&group["sections"]) {
                    let Some(date) = section["start_date"]
                        .as_str()
                        .filter(|date| !date.is_empty())
                    else {
                        audit
                            .warnings
                            .push(format!("{}: undated section", path.display()));
                        tracing::warn!(file = %path.display(), "undated section");
                        continue;
                    };
                    audit.section_dates.insert(date.into());
                    let mut parts = date.split('-');
                    let year: i64 = parts.next().context("missing year")?.parse()?;
                    let month: u8 = parts.next().context("missing month")?.parse()?;
                    let season = match month {
                        1..=4 => 'S',
                        5..=7 => 'M',
                        _ => 'F',
                    };
                    let term = format!("{season}{:02}", year.rem_euclid(100));
                    let target = sections.entry((code.clone(), term.clone())).or_default();
                    if replaced.insert((path.parent().and_then(Path::parent), code.clone(), term)) {
                        target.clear();
                    }
                    let meetings: Vec<_> = array(&section["timings"])
                        .iter()
                        .map(|row| {
                            let days = array(&row["days"])
                                .iter()
                                .filter_map(|day| match day.as_str()? {
                                    "monday" => Some('M'),
                                    "tuesday" => Some('T'),
                                    "wednesday" => Some('W'),
                                    "thursday" => Some('R'),
                                    "friday" => Some('F'),
                                    "saturday" => Some('S'),
                                    "sunday" => Some('U'),
                                    _ => None,
                                })
                                .collect();
                            meeting(
                                row,
                                days,
                                clock(&row["time"]["start"]),
                                clock(&row["time"]["end"]),
                            )
                        })
                        .collect();
                    let names: Vec<_> = array(&section["instructors"])
                        .iter()
                        .map(|row| instructor(row, &code, instructors))
                        .collect();
                    let id = string(&section["id"]);
                    target.insert(id.clone(), json!({"id": id, "label": text_or(&section["name"], ""),
                        "kind": text_or(&group["group_name"], ""), "instructors": names, "meetings": meetings,
                        "capacity": section["max_enrollment"], "waitlist": section["waitlist"]}));
                }
            }
        }
    }
    Ok(())
}

fn add_term(course: &mut Value, term: &str) {
    let terms = course["offered_terms"]
        .as_array_mut()
        .expect("course terms");
    if !terms.iter().any(|value| value.as_str() == Some(term)) {
        terms.push(json!(term));
    }
}

pub(super) fn load(source: &Path) -> Result<Catalog> {
    let mut roots = fs::read_dir(source)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    roots.sort();
    let mut paths = Vec::new();
    for root in &roots {
        paths.extend(files(&root.join("courses_history"))?);
    }

    let mut audit = Audit::default();
    let mut courses = Rows::new();
    let mut instructors = Rows::new();
    let mut geneds = BTreeMap::<String, BTreeSet<String>>::new();
    for path in paths
        .iter()
        .filter(|path| path.file_name().is_some_and(|name| name == "info.json"))
    {
        let row = read(path)?;
        let code = string(&row["code"]);
        if code.is_empty() {
            audit.warnings.push(path.display().to_string());
            tracing::warn!(file = %path.display(), "missing course code");
            continue;
        }
        if courses.contains_key(&code) {
            continue;
        }
        let mut terms = BTreeSet::new();
        for semester in array(&row["offerings"])
            .iter()
            .flat_map(|offering| array(&offering["semesters"]))
        {
            if let Some(year) = integer(&semester["year"]) {
                audit.offering_years.insert(year);
            }
            terms.extend(term(&semester["year"], &semester["semester"]));
        }
        let names: Vec<_> = array(&row["instructors"])
            .iter()
            .map(|row| instructor(row, &code, &mut instructors))
            .collect();
        let requirements = |key: &str| {
            array(&row[key])
                .iter()
                .filter_map(|value| value["code"].as_str())
                .filter(|code| !code.is_empty())
                .collect::<Vec<_>>()
        };
        let tags: BTreeSet<_> = array(&row["offering_tags"])
            .iter()
            .filter(|tag| !tag["sem"].is_null())
            .filter_map(|tag| tag["name"].as_str())
            .filter(|name| !name.is_empty())
            .collect();
        for tag in &tags {
            geneds
                .entry((*tag).to_owned())
                .or_default()
                .insert(code.clone());
        }
        courses.insert(code.clone(), json!({
            "code": code, "name": text_or(&row["name"], ""),
            "description": text_or(&row["long_desc"], ""), "units": default(&row["units"], json!(0)),
            "department": department(&code), "level": level(&code),
            "offered_terms": terms, "gened_tags": tags,
            "prerequisites": text_or(&row["prereqs"]["text"], ""),
            "corequisites": requirements("co_reqs"), "antirequisites": requirements("anti_reqs"), "instructors": names,
        }));
    }
    ensure!(
        !courses.is_empty(),
        "no Stellic course exports found under {}",
        source.display()
    );

    let mut sections = Sections::new();
    stellic_sections(
        &paths,
        &courses,
        &mut sections,
        &mut instructors,
        &mut audit,
    )?;

    let mut member_programs = Rows::new();
    for root in roots {
        for path in files(&root.join("programs"))? {
            let row = read(&path)?;
            let id = format!(
                "{}:{}",
                string(&row["catalog_id"]),
                string(&row["audit_id"])
            );
            member_programs.entry(id.clone()).or_insert_with(|| json!({
                "id": id,
                "name": row["program_name"], "audit_name": row["audit_name"], "program_type": row["program_type"],
                "tree": row["tree"], "source": "Stellic export", "retrieved_at": null,
            }));
        }
    }

    for ((code, term), entries) in sections {
        let course = courses
            .get_mut(&code)
            .context("section references unknown course")?;
        add_term(course, &term);
        let terms = course
            .as_object_mut()
            .context("invalid course")?
            .entry("sections_by_term")
            .or_insert_with(|| json!({}));
        terms[term] = Value::Array(entries.into_values().collect());
    }
    let mut ordered_terms = BTreeSet::new();
    for course in courses.values_mut() {
        if course["sections_by_term"].is_null() {
            course["sections_by_term"] = json!({});
        }
        for term in array(&course["offered_terms"])
            .iter()
            .filter_map(Value::as_str)
        {
            let (season, year) = term.split_at(1);
            let order = match season {
                "S" => 0,
                "M" => 1,
                "F" => 2,
                _ => anyhow::bail!("invalid term {term}"),
            };
            ordered_terms.insert((year.parse::<u8>()?, order, term.to_owned()));
        }
    }
    Ok(Catalog {
        audit,
        terms: ordered_terms
            .into_iter()
            .rev()
            .map(|(_, _, term)| term)
            .collect(),
        courses: courses.into_values().collect(),
        instructors: instructors.into_values().collect(),
        geneds: geneds
            .into_iter()
            .map(|(id, courses)| json!({"id": id, "name": id, "courses": courses}))
            .collect(),
        member_programs: member_programs.into_values().collect(),
    })
}
