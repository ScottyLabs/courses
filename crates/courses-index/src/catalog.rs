use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{self, Read},
    path::Path,
};

use anyhow::{Context, Result, ensure};
use flate2::{Compression, write::GzEncoder};
use rusqlite::{Connection, params, params_from_iter};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

mod normalize;
struct Catalog {
    audit: normalize::Audit,
    terms: Vec<String>,
    courses: Vec<Value>,
    instructors: Vec<Value>,
    member_programs: Vec<Value>,
    geneds: Vec<Value>,
}

pub fn build(source: &Path, output: &Path) -> Result<()> {
    let catalog = normalize::load(source)?;
    fs::create_dir_all(output)?;
    for audience in ["public", "member"] {
        write_audience(&catalog, output, audience)
            .with_context(|| format!("building {audience} catalog"))?;
    }
    write_report(&catalog, source, output)
}

fn write_report(catalog: &Catalog, source: &Path, output: &Path) -> Result<()> {
    let inventory_path = output.join("inventory.json");
    let prior: Vec<Value> = if inventory_path.exists() {
        serde_json::from_slice(&fs::read(&inventory_path)?)?
    } else {
        vec![]
    };
    let cache: std::collections::BTreeMap<_, _> = prior
        .iter()
        .filter_map(|row| Some((row["path"].as_str()?, row)))
        .collect();
    let mut inventory = Vec::new();
    let mut bytes = 0;
    for entry in WalkDir::new(source).sort_by_file_name() {
        let entry = entry?;
        if !entry.path().is_file() || entry.file_name() == ".DS_Store" {
            continue;
        }
        let path = entry.path().strip_prefix(source)?.to_string_lossy();
        let metadata = entry.path().metadata()?;
        let size = metadata.len();
        let modified = u64::try_from(
            metadata
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos(),
        )?;
        bytes += size;
        if let Some(previous) = cache.get(path.as_ref()).filter(|row| {
            row["bytes"].as_u64() == Some(size) && row["mtime_ns"].as_u64() == Some(modified)
        }) {
            inventory.push((**previous).clone());
            continue;
        }
        let mut hasher = Sha256::new();
        let mut file = File::open(entry.path())?;
        let mut buffer = [0; 32 * 1024];
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        inventory.push(json!({
            "path": path, "bytes": size, "mtime_ns": modified,
            "sha256": format!("{:x}", hasher.finalize()),
        }));
    }
    let section_count: usize = catalog
        .courses
        .iter()
        .filter_map(|course| course["sections_by_term"].as_object())
        .flat_map(|terms| terms.values().filter_map(Value::as_array))
        .map(Vec::len)
        .sum();
    let audit = &catalog.audit;
    let programs: BTreeSet<_> = catalog
        .member_programs
        .iter()
        .filter_map(|program| program["id"].as_str()?.split(':').next())
        .collect();
    let report = json!({
        "imported_at": chrono::Utc::now().to_rfc3339(),
        "source_files": inventory.len(), "source_bytes": bytes,
        "courses": catalog.courses.len(), "section_records": section_count,
        "instructors": catalog.instructors.len(), "gened_categories": catalog.geneds.len(),
        "program_audits": catalog.member_programs.len(),
        "programs": programs.len(),
        "section_dates": [audit.section_dates.first(), audit.section_dates.last()],
        "offering_years": [audit.offering_years.first(), audit.offering_years.last()],
        "terms": catalog.terms,
        "warnings": audit.warnings,
        "notes": [
            "File modification times are copy dates, not verified scrape dates.",
            "Stellic student_context/alerts and all unrecognized fields are excluded.",
            "Program trees require CMU authentication and are not a validated degree audit engine.",
        ],
    });
    fs::write(inventory_path, serde_json::to_vec_pretty(&inventory)?)?;
    fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

fn write_audience(catalog: &Catalog, output: &Path, audience: &str) -> Result<()> {
    let member = audience == "member";
    let path = output.join(format!("{audience}.sqlite"));
    let temporary = tempfile::NamedTempFile::new_in(output)?;
    let mut db = Connection::open(temporary.path())?;
    let tx = db.transaction()?;
    tx.execute_batch(include_str!("../catalog/schema.sql"))?;
    let coverage = "INSERT INTO catalog_coverage VALUES(?,?)";
    for dataset in ["terms", "courses", "geneds"] {
        tx.execute(coverage, params![dataset, ""])?;
    }
    if member {
        tx.execute(coverage, params!["instructors", ""])?;
    }
    for (index, term) in catalog.terms.iter().enumerate() {
        let (season, year) = term.split_at(1);
        let season = match season {
            "F" => "Fall",
            "S" => "Spring",
            "M" => "Summer",
            _ => anyhow::bail!("invalid term {term}"),
        };
        let payload = json!({"id": term, "label": format!("{season} 20{year}")});
        tx.execute(
            "INSERT INTO catalog_terms VALUES(?,?,?)",
            params![term, index as i64, payload.to_string()],
        )?;
        tx.execute(coverage, params!["courses", term])?;
    }

    {
        let mut insert = tx.prepare("INSERT INTO catalog_courses VALUES(?,?,?,?,?,?,?,?,?,?)")?;
        for course in &catalog.courses {
            let terms: BTreeSet<&str> = course["offered_terms"]
                .as_array()
                .context("missing offered terms")?
                .iter()
                .filter_map(Value::as_str)
                .collect();
            let seasons: BTreeSet<_> = terms
                .iter()
                .map(|term| match term.as_bytes().first() {
                    Some(b'F') => Ok("fall"),
                    Some(b'S') => Ok("spring"),
                    Some(b'M') => Ok("summer"),
                    _ => anyhow::bail!("invalid term {term}"),
                })
                .collect::<Result<_>>()?;
            let seasons = seasons.into_iter().collect::<Vec<_>>().join("|");
            let mut row = course.as_object().context("invalid course")?.clone();
            let sections = row
                .remove("sections_by_term")
                .context("missing section data")?;
            if !member {
                for key in ["fce", "instructors", "syllabi"] {
                    row.remove(key);
                }
            }
            for term in std::iter::once("").chain(terms) {
                let mut selected = sections[term].as_array().cloned().unwrap_or_default();
                if !member {
                    for section in &mut selected {
                        section["instructors"] = json!([]);
                        for meeting in section["meetings"]
                            .as_array_mut()
                            .context("missing meetings")?
                        {
                            meeting
                                .as_object_mut()
                                .context("invalid meeting")?
                                .remove("location");
                        }
                    }
                }
                row.insert("sections".into(), Value::Array(selected));
                insert.execute(params![
                    term,
                    course["code"].as_str(),
                    course["name"].as_str(),
                    course["description"].as_str(),
                    course["department"].as_str(),
                    course["level"]
                        .as_str()
                        .context("missing course level")?
                        .parse::<i64>()?,
                    course["units"]
                        .as_f64()
                        .or_else(|| course["units"].as_str()?.parse().ok()),
                    seasons,
                    row.get("fce")
                        .and_then(|fce| fce["overall_course"].as_f64()),
                    serde_json::to_string(&row)?,
                ])?;
            }
        }
    }
    if member {
        for instructor in &catalog.instructors {
            tx.execute(
                "INSERT INTO catalog_instructors VALUES(?,?,?,?)",
                params![
                    instructor["id"].as_str(),
                    instructor["name"].as_str(),
                    instructor.get("department").map_or(Some(""), Value::as_str),
                    instructor.to_string(),
                ],
            )?;
        }
    }
    let restricted = if member {
        catalog.member_programs.as_slice()
    } else {
        &[]
    };
    for program in restricted {
        tx.execute(
            "INSERT INTO programs VALUES(?,?,?)",
            params![
                program["id"].as_str(),
                program["name"].as_str(),
                program.to_string(),
            ],
        )?;
    }
    for gened in &catalog.geneds {
        tx.execute(
            "INSERT INTO catalog_geneds VALUES(?,?)",
            params![gened["id"].as_str(), gened.to_string()],
        )?;
    }
    tx.commit()?;
    let integrity: String = db.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    ensure!(
        integrity == "ok",
        "catalog integrity check failed: {integrity}"
    );
    drop(db);
    temporary.persist(&path)?;

    let browser_file = tempfile::NamedTempFile::new_in(output)?;
    fs::copy(&path, browser_file.path())?;
    let browser = Connection::open(browser_file.path())?;
    let keep: Vec<_> = std::iter::once("")
        .chain(catalog.terms.iter().take(3).map(String::as_str))
        .collect();
    let marks = vec!["?"; keep.len()].join(",");
    browser.execute(
        &format!("DELETE FROM catalog_courses WHERE term_id NOT IN ({marks})"),
        params_from_iter(&keep),
    )?;
    browser.execute(
        &format!(
            "DELETE FROM catalog_coverage WHERE dataset='courses' AND term_id NOT IN ({marks})"
        ),
        params_from_iter(&keep),
    )?;
    browser.execute_batch("DELETE FROM programs; VACUUM;")?;
    drop(browser);
    let browser_path = output.join(format!("{audience}-browser.sqlite"));
    browser_file.persist(&browser_path)?;

    let mut compressed = tempfile::NamedTempFile::new_in(output)?;
    let mut gzip = GzEncoder::new(compressed.as_file_mut(), Compression::new(6));
    io::copy(&mut File::open(browser_path)?, &mut gzip)?;
    gzip.finish()?;
    compressed.persist(output.join(format!("{audience}-browser.sqlite.gz")))?;
    Ok(())
}
