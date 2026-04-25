mod discovery;
mod stellic;

use anyhow::{Context, Result};
use clap::Parser;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;

use discovery::{Sem, Soc, Task, course_tasks, fetch_soc, parse_fce};
use stellic::Stellic;

const SEASONS: &[&str] = &["fall", "spring", "summer_1", "summer_2"];

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(long, default_value = "data/fces.csv", env = "FCE_PATH")]
    fce_path: PathBuf,

    #[arg(long, default_value_t = 64, env = "CONCURRENCY")]
    concurrency: usize,

    #[arg(long, default_value = "data/courses_history", env = "OUT_DIR")]
    out_dir: PathBuf,

    #[arg(long, env = "COOKIE_HEADER")]
    cookie_header: Option<String>,

    #[arg(long, env = "ANDREW_ID")]
    andrew_id: String,

    #[arg(long, env = "LIMIT")]
    limit: Option<usize>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let args = Args::parse();

    let (stellic, term_joined) = Stellic::login(args.cookie_header, &args.andrew_id, args.out_dir)?;
    let joined_sem = Sem::from_id(term_joined.semester).context("unknown joined semester id")?;
    let anchor = joined_sem.ay_start(term_joined.year);
    info!(
        plan_id = %stellic.plan_id,
        joined_year = term_joined.year,
        joined_sem = ?joined_sem,
        anchor,
        "authed"
    );

    let (fce, soc_results) = rayon::join(
        || parse_fce(&args.fce_path).context("fce"),
        || {
            SEASONS
                .par_iter()
                .map(|s| (*s, fetch_soc(s)))
                .collect::<Vec<_>>()
        },
    );
    let fce = fce?;
    let soc: HashMap<&str, Soc> = soc_results
        .into_iter()
        .filter_map(|(season, r)| match r {
            Ok(Some(s)) => Some((season, s)),
            Ok(None) => {
                debug!(season, "soc unpublished, skipping");
                None
            }
            Err(e) => {
                warn!(season, error = %e, "soc fetch failed");
                None
            }
        })
        .collect();

    let courses: HashSet<String> = fce
        .keys()
        .chain(soc.values().flat_map(|s| s.codes.iter()))
        .cloned()
        .collect();

    let mut tasks: Vec<Task> = courses
        .into_par_iter()
        .flat_map_iter(|c| course_tasks(&c, &fce, &soc, anchor))
        .collect();
    if let Some(n) = args.limit {
        tasks.truncate(n);
    }
    let total = tasks.len();
    info!(total, concurrency = args.concurrency, "scraping");

    let done = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(args.concurrency)
        .build()?;
    pool.install(|| {
        tasks.into_par_iter().for_each(|task| {
            let result = match &task {
                Task::Info(course) => stellic.save_info(course),
                Task::Sections {
                    course,
                    lyear,
                    sem_id,
                } => stellic.save_sections(course, *lyear, *sem_id),
            };
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            if let Err(e) = result {
                failed.fetch_add(1, Ordering::Relaxed);
                debug!(error = %e, task = ?task, "task failed");
            }
            if n.is_multiple_of(500) {
                info!(
                    done = n,
                    total,
                    failed = failed.load(Ordering::Relaxed),
                    "progress"
                );
            }
        });
    });

    info!(
        done = done.load(Ordering::Relaxed),
        failed = failed.load(Ordering::Relaxed),
        "complete"
    );
    Ok(())
}
