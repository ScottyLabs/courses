//! `courses-index build`: builds `catalog.sqlite` from the raw inputs.

use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use courses_index::pipeline::{self, Options};

#[derive(Parser, Debug)]
#[command(about = "Build the course catalog")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Build catalog.sqlite from the scraper's output and data/.
    Build {
        /// The scraper's output root: Stellic exports, syllabi, SOC captures and schedule CSVs.
        #[arg(long, env = "EXPORTED_ROOT", default_value = "exported")]
        exported: PathBuf,

        /// The repository's data directory: terms.json, fces.csv and finals/.
        #[arg(long, default_value = "data")]
        data: PathBuf,

        /// Where to write the catalog.
        #[arg(long, short, default_value = "data/catalog/catalog.sqlite")]
        output: PathBuf,

        /// Build only these terms (repeat or separate with commas), for faster local builds. The term index,
        /// offerings and teaching documents then cover only these terms.
        #[arg(long = "term", value_delimiter = ',')]
        terms: Vec<String>,

        /// Whose Stellic export the lyear values in data/terms.json are for, when exported/ has several.
        #[arg(long, env = "STELLIC_EXPORT_USER")]
        stellic_user: Option<String>,

        /// Also write every document as JSON under this directory (public/ and member/), for inspection and diffs.
        #[arg(long)]
        dump: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    match Cli::parse().command {
        Command::Build {
            exported,
            data,
            output,
            terms,
            stellic_user,
            dump,
        } => {
            let source_date_epoch = std::env::var("SOURCE_DATE_EPOCH")
                .ok()
                .map(|s| s.trim().parse::<i64>())
                .transpose()
                .context("SOURCE_DATE_EPOCH must be whole seconds")?;
            let started = Instant::now();
            let summary = pipeline::build(&Options {
                exported,
                data,
                output,
                terms,
                stellic_user,
                dump,
                source_date_epoch,
            })?;
            for (id, courses, sections) in &summary.terms {
                println!("{id}: {courses} courses, {sections} sections");
            }
            for (id, exams) in &summary.finals {
                println!("finals/{id}: {exams} exams");
            }
            println!(
                "catalog: {} courses; syllabi: {} entries",
                summary.catalog_courses, summary.syllabi
            );
            println!(
                "{}: {} documents, {:.1} MB of JSON, {:.1} MB file, version {}, in {:.1?}",
                summary.output.display(),
                summary.documents,
                summary.json_bytes as f64 / 1e6,
                summary.file_bytes as f64 / 1e6,
                summary.version,
                started.elapsed()
            );
            Ok(())
        }
    }
}
