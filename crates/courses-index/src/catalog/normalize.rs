use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use chrono::NaiveTime;
use indexmap::IndexMap;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use super::Catalog;

type Rows = BTreeMap<String, Value>;
type SectionRows = IndexMap<String, Value>;
type Sections = IndexMap<(String, String), SectionRows>;

#[derive(Default)]
pub(super) struct Audit {
    pub section_dates: BTreeSet<String>,
    pub offering_years: BTreeSet<i64>,
    pub fce_rows: usize,
    pub fce_years: BTreeSet<String>,
    pub warnings: Vec<String>,
}

fn read(path: &Path) -> Result<Value> {
    serde_json::from_slice(&fs::read(path)?).with_context(|| path.display().to_string())
}

fn decode(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            for key in ["$numberInt", "$numberLong", "$numberDouble"] {
                if let Some(text) = map.get(key).and_then(Value::as_str) {
                    return if key == "$numberDouble" {
                        json!(text.parse::<f64>().ok())
                    } else {
                        json!(text.parse::<i64>().ok())
                    };
                }
            }
            Value::Object(
                map.into_iter()
                    .map(|(key, value)| (key, decode(value)))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.into_iter().map(decode).collect()),
        value => value,
    }
}

fn records(path: &Path) -> Result<Vec<Value>> {
    let raw = fs::read_to_string(path)?;
    let rows = match serde_json::from_str(&raw) {
        Ok(Value::Array(rows)) => rows,
        Ok(Value::Object(rows)) => rows.into_values().collect(),
        Ok(_) => anyhow::bail!("invalid records in {}", path.display()),
        Err(_) => raw
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(serde_json::from_str)
            .collect::<serde_json::Result<_>>()?,
    };
    Ok(rows.into_iter().map(decode).collect())
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

fn number(text: &str) -> f64 {
    text.split(',')
        .next()
        .unwrap_or_default()
        .trim()
        .parse()
        .unwrap_or_default()
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

fn legacy_schedules(source: &Path, courses: &Rows, sections: &mut Sections) -> Result<()> {
    for row in records(&source.join("courses-files/schedules.json"))? {
        let Some(term) = term(&row["year"], &row["semester"]) else {
            continue;
        };
        let code = string(&row["courseID"]);
        if !courses.contains_key(&code) {
            continue;
        }
        for kind in ["lectures", "sections"] {
            for section in array(&row[kind]) {
                let name = string(&section["name"]);
                let id = format!("legacy:{term}:{code}:{name}");
                let meetings: Vec<_> = array(&section["times"])
                    .iter()
                    .map(|row| {
                        let days = array(&row["days"])
                            .iter()
                            .filter_map(|day| {
                                "UMTWRFS".chars().nth(usize::try_from(integer(day)?).ok()?)
                            })
                            .collect();
                        meeting(row, days, clock(&row["begin"]), clock(&row["end"]))
                    })
                    .collect();
                sections
                    .entry((code.clone(), term.clone()))
                    .or_default()
                    .insert(
                        id.clone(),
                        json!({"id": id, "label": name, "kind": kind,
                    "instructors": array(&section["instructors"]), "meetings": meetings}),
                    );
            }
        }
    }
    Ok(())
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

fn minutes(value: &Value) -> Result<String> {
    if value.is_null() {
        return Ok(String::new());
    }
    let time = value
        .as_u64()
        .filter(|time| *time < 1440)
        .context("invalid CourseWeave time")?;
    Ok(format!("{:02}:{:02}", time / 60, time % 60))
}

fn courseweave(path: &Path, courses: &mut Rows, sections: &mut Sections) -> Result<Value> {
    let raw = fs::read(path)?;
    let snapshot: Value = serde_json::from_slice(&raw)?;
    let meta = &snapshot["term"];
    ensure!(
        meta["id"] == "S27" && meta["year"] == 2027 && meta["season"] == "spring",
        "invalid CourseWeave term"
    );
    let rows = array(&snapshot["courses"]);
    ensure!(!rows.is_empty(), "empty CourseWeave snapshot");
    let mut seen = BTreeSet::new();
    let mut section_count = 0;
    let code_pattern = regex::Regex::new(r"^[0-9]{2}-[0-9]{3}$")?;
    for row in rows {
        let code = string(&row["code"]);
        ensure!(
            code_pattern.is_match(&code) && seen.insert(code.clone()),
            "invalid or duplicate CourseWeave course {code}"
        );
        let course = courses.entry(code.clone()).or_insert_with(|| json!({
            "code": code, "description": "", "department": row["dept"], "level": level(&code),
            "offered_terms": [], "gened_tags": [], "prerequisites": "", "corequisites": [], "antirequisites": [],
        }));
        course["name"] = row["title"].clone();
        course["units"] = default(&row["units"], json!(0));
        course["instructors"] = row["instructors"].clone();
        add_term(course, "S27");
        course["offered_terms"]
            .as_array_mut()
            .expect("course terms")
            .sort_by(|left, right| left.as_str().cmp(&right.as_str()));
        let mut converted = SectionRows::new();
        for section in array(&row["sections"]) {
            let id = format!(
                "courseweave:S27:{code}:{}:{}",
                string(&section["kind"]),
                string(&section["name"])
            );
            let mut meetings = Vec::new();
            for row in array(&section["meetings"]) {
                let days = array(&row["days"])
                    .iter()
                    .map(|day| {
                        let index = day
                            .as_u64()
                            .filter(|index| *index < 7)
                            .context("invalid CourseWeave day")?;
                        Ok("MTWRFSU".as_bytes()[index as usize] as char)
                    })
                    .collect::<Result<_>>()?;
                meetings.push(meeting(
                    row,
                    days,
                    minutes(&row["start"])?,
                    minutes(&row["end"])?,
                ));
            }
            let value = json!({"id": id, "label": section["name"], "kind": section["kind"],
                "instructors": section["instructors"], "meetings": meetings, "capacity": section["capacity"],
                "parent": section["parent"], "mini": section["mini"], "mode": section["mode"]});
            ensure!(
                converted.insert(id.clone(), value).is_none(),
                "duplicate section {id}"
            );
        }
        section_count += converted.len();
        sections.insert((code, "S27".into()), converted);
    }
    Ok(json!({
        "source": "https://courseweave.jay3332.tech/data/S27.json",
        "sha256": format!("{:x}", Sha256::digest(&raw)),
        "generated_at": snapshot["generatedAt"],
        "upstream_sources": array(&meta["sources"]),
        "term": "S27", "courses": seen.len(), "sections": section_count,
        "note": "Third-party snapshot; generation time is not independently verified upstream freshness.",
    }))
}

fn add_term(course: &mut Value, term: &str) {
    let terms = course["offered_terms"]
        .as_array_mut()
        .expect("course terms");
    if !terms.iter().any(|value| value.as_str() == Some(term)) {
        terms.push(json!(term));
    }
}

#[derive(Default)]
struct Fce {
    terms: BTreeSet<(String, String)>,
    totals: [(f64, f64); 3],
}

fn add_fces(source: &Path, courses: &mut Rows, audit: &mut Audit) -> Result<()> {
    let mut csv = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(source.join("dalmatian_courses/fce_data.csv"))?;
    let headers = csv.headers()?.clone();
    let mut summaries = BTreeMap::<String, Fce>::new();
    for row in csv.records() {
        let row = row?;
        let get = |key| {
            headers
                .iter()
                .position(|header| header == key)
                .and_then(|index| row.get(index))
                .unwrap_or_default()
        };
        let digits: String = get("Num").chars().filter(char::is_ascii_digit).collect();
        if digits.len() < 5 {
            continue;
        }
        audit.fce_rows += 1;
        audit.fce_years.insert(get("Year").into());
        if get("College") == "Teaching Assistants" {
            continue;
        }
        let suffix = &digits[digits.len() - 5..];
        let code = format!("{}-{}", &suffix[..2], &suffix[2..]);
        if !courses.contains_key(&code) {
            continue;
        }
        let summary = summaries.entry(code).or_default();
        summary
            .terms
            .insert((get("Year").into(), get("Sem").into()));
        let weight = number(get("# Responses"));
        for (column, (sum, count)) in [
            "Hrs Per Week",
            "Overall course rate",
            "Overall teaching rate",
        ]
        .into_iter()
        .zip(&mut summary.totals)
        {
            if let Ok(rating) = get(column).trim().parse::<f64>() {
                if rating.is_finite() && weight.is_finite() && weight > 0.0 {
                    *sum += rating * weight;
                    *count += weight;
                }
            }
        }
    }
    for (code, summary) in summaries {
        let mut payload = json!({"semesters": summary.terms.len()});
        for (key, (sum, count)) in ["hours_per_week", "overall_course", "overall_teaching"]
            .into_iter()
            .zip(summary.totals)
        {
            if count > 0.0 {
                payload[key] = json!(sum / count);
            }
        }
        courses.get_mut(&code).context("missing FCE course")?["fce"] = payload;
    }
    Ok(())
}

pub(super) fn load(
    source: &Path,
    stellic_root: Option<&Path>,
    snapshot: Option<&Path>,
) -> Result<Catalog> {
    let mut paths = files(&source.join("backup/exported/courses_history"))?;
    if let Some(root) = stellic_root {
        let history = root.join("courses_history");
        ensure!(
            history.is_dir(),
            "missing courses_history in {}",
            root.display()
        );
        paths.extend(files(&history)?);
    }
    let mut audit = Audit::default();
    let mut courses = Rows::new();
    let mut instructors = Rows::new();
    for row in records(&source.join("courses-files/courses.json"))? {
        let code = string(&row["courseID"]);
        courses.insert(code.clone(), json!({
            "code": code, "name": row["name"], "description": text_or(&row["desc"], ""),
            "units": number(&string(&row["units"])), "department": text_or(&row["department"], &department(&code)),
            "level": level(&code), "offered_terms": [], "gened_tags": [],
            "prerequisites": text_or(&row["prereqString"], ""), "corequisites": array(&row["coreqs"]),
            "antirequisites": [],
        }));
    }
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
        let old = courses.remove(&code).unwrap_or_default();
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
        courses.insert(code.clone(), json!({
            "code": code, "name": text_or(&row["name"], text_or(&old["name"], "")),
            "description": text_or(&row["long_desc"], text_or(&old["description"], "")),
            "units": default(&row["units"], json!(0)),
            "department": text_or(&old["department"], &department(&code)), "level": level(&code),
            "offered_terms": terms, "gened_tags": array(&old["gened_tags"]),
            "prerequisites": text_or(&row["prereqs"]["text"], ""),
            "corequisites": requirements("co_reqs"), "antirequisites": requirements("anti_reqs"), "instructors": names,
        }));
    }

    let mut sections = Sections::new();
    legacy_schedules(source, &courses, &mut sections)?;
    stellic_sections(
        &paths,
        &courses,
        &mut sections,
        &mut instructors,
        &mut audit,
    )?;
    let default_snapshot = source.join("courseweave/S27.json");
    let snapshot = snapshot.unwrap_or(&default_snapshot);
    let snapshot_report = if snapshot.exists() {
        courseweave(snapshot, &mut courses, &mut sections)?
    } else {
        Value::Null
    };

    let mut geneds = Rows::new();
    for row in records(&source.join("courses-files/geneds.json"))? {
        let code = string(&row["courseID"]);
        for tag in array(&row["tags"]) {
            let id = format!("{}:{}", string(&row["school"]), string(tag));
            let entry = geneds.entry(id.clone()).or_insert_with(|| json!({
                "id": id, "name": id, "courses": [],
                "description": format!("Source updated {}", row["lastUpdated"].as_str().unwrap_or("unknown")),
            }));
            entry["courses"]
                .as_array_mut()
                .expect("gened courses")
                .push(json!(code));
            if let Some(course) = courses.get_mut(&code) {
                course["gened_tags"]
                    .as_array_mut()
                    .expect("gened tags")
                    .push(json!(id));
            }
        }
    }

    add_fces(source, &mut courses, &mut audit)?;
    for row in records(&source.join("dalmatian_courses/courseSyllabi.json"))? {
        let raw = string(&row["number"]).replace('-', "");
        let Some((prefix, suffix)) = raw.split_at_checked(2) else {
            continue;
        };
        let code = format!("{prefix}-{suffix}");
        let url = text_or(&row["url"], "");
        if let Some(course) = courses
            .get_mut(&code)
            .filter(|_| url.starts_with("https://canvas.cmu.edu/"))
        {
            let syllabi = course
                .as_object_mut()
                .context("invalid course")?
                .entry("syllabi")
                .or_insert_with(|| json!([]));
            syllabi.as_array_mut().expect("syllabi").push(json!({
                "label": format!("{}{} {}", string(&row["season"]), string(&row["year"]), string(&row["section"])),
                "url": url,
            }));
        }
    }

    let mut member_programs = Vec::new();
    for path in files(&source.join("backup/exported/programs"))? {
        let row = read(&path)?;
        member_programs.push(json!({
            "id": format!("{}:{}", string(&row["catalog_id"]), string(&row["audit_id"])),
            "name": row["program_name"], "audit_name": row["audit_name"], "program_type": row["program_type"],
            "tree": row["tree"], "source": "Stellic export", "retrieved_at": null,
        }));
    }
    let manifest = Path::new("data/programs-current/manifest.json");
    let public_programs = if manifest.exists() {
        array(&read(manifest)?["programs"]).to_vec()
    } else {
        vec![]
    };

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
        snapshot_report,
        terms: ordered_terms
            .into_iter()
            .rev()
            .map(|(_, _, term)| term)
            .collect(),
        courses: courses.into_values().collect(),
        instructors: instructors.into_values().collect(),
        geneds: geneds.into_values().collect(),
        member_programs,
        public_programs,
    })
}
