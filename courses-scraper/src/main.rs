//! CMU course catalog scraper. Pulls course info, sections, program audits
//! and syllabi from Stellic and Canvas into `exported/`, driven by FCE and
//! Schedule of Classes discovery feeds. The companion `courses-index` crate
//! reads this output to build the client-side search index.
//!
//! `docs/` at the repo root carries per-source reference material covering
//! API shapes, data conventions, and operator guidance.

mod canvas;
mod discovery;
mod requirements;
mod stellic;
mod syllabi;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;

use canvas::Canvas;
use discovery::{Sem, Soc, Task, course_tasks, fetch_soc, parse_fce};
use stellic::Stellic;

const SEASONS: &[&str] = &["fall", "spring", "summer_1", "summer_2"];

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
enum Mode {
    Courses,
    Programs,
    Syllabi,
    All,
}

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(long, default_value = "data/fces.csv", env = "FCE_PATH")]
    fce_path: PathBuf,

    #[arg(long, default_value_t = 32, env = "CONCURRENCY")]
    concurrency: usize,

    /// Root directory for scrape output. The scraper writes under
    /// `<export_root>/<username>/{courses_history,programs,syllabi}/`,
    /// where `<username>` is the Stellic-authed AndrewID. Multiple
    /// students' scrapes coexist as sibling subdirectories.
    #[arg(long, default_value = "exported", env = "EXPORT_ROOT")]
    export_root: PathBuf,

    #[arg(long, env = "COOKIE_HEADER")]
    cookie_header: Option<String>,

    #[arg(long, env = "CANVAS_TOKEN")]
    canvas_token: Option<String>,

    #[arg(long, env = "LIMIT")]
    limit: Option<usize>,

    #[arg(long, value_enum, default_value_t = Mode::Courses, env = "MODE")]
    mode: Mode,

    #[arg(long, env = "INCREMENTAL")]
    incremental: bool,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let args = Args::parse();

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(args.concurrency)
        .build()?;

    let needs_canvas = matches!(args.mode, Mode::Syllabi | Mode::All);

    // Always authenticate against Stellic, even for syllabi-only runs:
    // the AndrewID it returns names the per-student subdir under
    // `export_root` for all output kinds.
    let (stellic, term_joined) = Stellic::login(args.cookie_header.clone(), &args.export_root)?;

    let joined_sem = Sem::from_id(term_joined.semester).context("unknown joined semester id")?;
    let anchor = joined_sem.ay_start(term_joined.year);

    let student_root = args.export_root.join(&stellic.username);
    let programs_dir = student_root.join("programs");
    let syllabi_dir = args.export_root.join("syllabi");

    info!(
        username = %stellic.username,
        plan_id = %stellic.plan_id,
        joined_year = term_joined.year,
        joined_sem = ?joined_sem,
        anchor,
        student_root = %student_root.display(),
        "authed"
    );

    let canvas = if needs_canvas {
        let token = args
            .canvas_token
            .clone()
            .context("--canvas-token (or CANVAS_TOKEN) required for syllabi mode")?;
        Some(Canvas::new(token))
    } else {
        None
    };

    match args.mode {
        Mode::Courses => {
            scrape_courses(&stellic, &args, anchor, &pool)?;
        }
        Mode::Programs => {
            scrape_programs(&stellic, &args, &programs_dir, &pool)?;
        }
        Mode::Syllabi => {
            scrape_syllabi(canvas.as_ref().unwrap(), &args, &syllabi_dir, &pool)?;
        }
        Mode::All => {
            let c = canvas.as_ref().unwrap();
            let ((cr, pr), sr) = pool.install(|| {
                rayon::join(
                    || {
                        rayon::join(
                            || scrape_courses(&stellic, &args, anchor, &pool),
                            || scrape_programs(&stellic, &args, &programs_dir, &pool),
                        )
                    },
                    || scrape_syllabi(c, &args, &syllabi_dir, &pool),
                )
            });
            cr?;
            pr?;
            sr?;
        }
    }
    Ok(())
}

fn scrape_syllabi(
    canvas: &Canvas,
    args: &Args,
    syllabi_dir: &std::path::Path,
    pool: &rayon::ThreadPool,
) -> Result<()> {
    let mut tasks = pool.install(|| syllabi::build_tasks(canvas))?;
    if let Some(n) = args.limit {
        tasks.truncate(n);
    }
    let total = tasks.len();
    info!(total, concurrency = args.concurrency, "scraping syllabi");

    let done = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    pool.install(|| {
        tasks.into_par_iter().for_each(|task| {
            let result = syllabi::save_task(canvas, syllabi_dir, &task, args.incremental);
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            if let Err(e) = result {
                failed.fetch_add(1, Ordering::Relaxed);
                debug!(error = %e, task = ?task, "syllabus save failed");
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
        "syllabi complete"
    );
    Ok(())
}

fn scrape_courses(
    stellic: &Stellic,
    args: &Args,
    anchor: i32,
    pool: &rayon::ThreadPool,
) -> Result<()> {
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
    info!(total, concurrency = args.concurrency, "scraping courses");

    let done = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    pool.install(|| {
        tasks.into_par_iter().for_each(|task| {
            let result = match &task {
                Task::Info(course) => stellic.save_info(course, args.incremental),
                Task::Sections {
                    course,
                    lyear,
                    sem_id,
                } => stellic.save_sections(course, *lyear, *sem_id, args.incremental),
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
        "courses complete"
    );
    Ok(())
}

fn scrape_programs(
    stellic: &Stellic,
    args: &Args,
    programs_dir: &std::path::Path,
    pool: &rayon::ThreadPool,
) -> Result<()> {
    let mut programs = stellic.get_programs()?;
    info!(count = programs.len(), "fetched program catalog");
    if let Some(n) = args.limit {
        programs.truncate(n);
    }
    let mut tasks = pool.install(|| requirements::build_tasks(stellic, &programs));
    if let Some(n) = args.limit {
        tasks.truncate(n);
    }
    let total = tasks.len();
    info!(
        total,
        concurrency = args.concurrency,
        "scraping requirements"
    );

    let done = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    pool.install(|| {
        tasks.into_par_iter().for_each(|task| {
            let result = requirements::save_audit(stellic, programs_dir, &task, args.incremental);
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            if let Err(e) = result {
                failed.fetch_add(1, Ordering::Relaxed);
                debug!(error = %e, task = ?task, "audit save failed");
            }
            if n.is_multiple_of(200) {
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
        "requirements complete"
    );
    Ok(())
}
