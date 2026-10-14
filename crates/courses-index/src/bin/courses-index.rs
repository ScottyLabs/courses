//! Native CLI for the catalog pipeline. Walks `exported/` and `data/fces.csv`
//! into a `Corpus` and writes the catalog artifact.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber::EnvFilter;

use courses_index::load;

#[derive(Parser, Debug)]
#[command(about = "Build the course catalog artifact")]
struct Args {
    /// Path to the scrape output root. The directory should contain one
    /// subdirectory per student (named by AndrewID), each with
    /// `courses_history/`, `programs/`, and `syllabi/` underneath. The
    /// loader walks every immediate subdirectory and consolidates.
    #[arg(long, env = "EXPORTED_ROOT", default_value = "exported")]
    exported_root: PathBuf,

    /// Path to the FCE CSV.
    #[arg(long, env = "FCE_CSV", default_value = "data/fces.csv")]
    fce_csv: PathBuf,

    /// Print a single course by code after loading (e.g. `--pick 15-122`).
    #[arg(long)]
    pick: Option<String>,

    /// Print one professor whose name contains this substring (case-insensitive).
    #[arg(long)]
    prof: Option<String>,

    /// Write the catalog to this path.
    #[arg(long)]
    write_catalog: Option<PathBuf>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    let corpus = load::run(&args.exported_root, &args.fce_csv)?;
    info!(
        courses = corpus.courses.len(),
        professors = corpus.professors.len(),
        sections = corpus.sections.len(),
        fce_rows = corpus.fce_rows.len(),
        "load complete"
    );

    if args.write_catalog.is_some() {
        unimplemented!();
    }

    if let Some(code) = args.pick {
        match corpus.courses.iter().find(|c| c.code == code) {
            Some(c) => {
                let n_sections = corpus
                    .sections
                    .iter()
                    .filter(|s| s.course_id == c.id)
                    .count();
                println!("{:#?}", c);
                println!("section_rows_for_course = {n_sections}");
            }
            None => eprintln!("no course with code {code}"),
        }
    }

    if let Some(needle) = args.prof {
        let lower = needle.to_lowercase();
        match corpus
            .professors
            .iter()
            .find(|p| p.name.to_lowercase().contains(&lower))
        {
            Some(p) => {
                let n_fce = corpus
                    .fce_rows
                    .iter()
                    .filter(|r| r.instructor_id == Some(p.id))
                    .count();
                println!("{:#?}", p);
                println!("fce_rows_for_prof = {n_fce}");
            }
            None => eprintln!("no professor matching {needle}"),
        }
    }

    let mut top: Vec<(&str, &str, f32)> = corpus
        .courses
        .iter()
        .map(|c| (c.code.as_str(), c.name.as_str(), c.pagerank))
        .collect();
    top.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    println!("\nTop 15 courses by PageRank:");
    for (code, name, score) in top.iter().take(15) {
        println!("  {score:.6}  {code:8}  {name}");
    }

    Ok(())
}
