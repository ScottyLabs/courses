//! CMU course catalog scraper. Pulls course info, sections, program audits
//! and syllabi from Stellic and Canvas into `exported/`, driven by FCE and
//! Schedule of Classes discovery feeds. The companion `courses-index` crate
//! reads this output to build the catalog.
//!
//! `sites/docs-migrating-wip` carries per-source reference material covering
//! API shapes, data conventions, and operator guidance.

mod canvas;
mod discovery;
mod ldap;
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
    Ldap,
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

    #[arg(long, value_enum, default_value_t = Mode::Courses, env = "MODE")]
    mode: Mode,

    /// Refresh just one absolute term, e.g. S27. Does not use the historical plan-window cap.
    #[arg(long)]
    term: Option<String>,

    /// Read the Cookie request header from a private local file, never command-line text.
    #[arg(long)]
    cookie_file: Option<PathBuf>,

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

    // Personal credentials come from plain env: workflow env in CI, the
    // devenv shell's secretspec dev profile (.env via local provider) in dev
    let cookie_header = match &args.cookie_file {
        Some(path) => Some(
            std::fs::read_to_string(path)
                .context("reading cookie file")?
                .trim()
                .to_owned(),
        ),
        None => std::env::var("COOKIE_HEADER")
            .ok()
            .filter(|s| !s.is_empty()),
    };
    if let Some(term) = &args.term {
        anyhow::ensure!(
            args.mode == Mode::Courses,
            "--term is supported only with --mode courses"
        );
        let (year, sem) = parse_target_term(term)?;
        let season = match sem {
            Sem::Fall => "fall",
            Sem::Spring => "spring",
            Sem::Summer => "summer_1",
        };
        let published = fetch_soc(season)?.is_some_and(|soc| soc.matches_term(year, sem));
        anyhow::ensure!(
            published,
            "The public Schedule of Classes does not yet publish {term}; no authenticated requests were made"
        );
    }
    if args.term.is_some() && args.incremental {
        anyhow::bail!("A term refresh must not skip existing records; omit --incremental");
    }
    let canvas_token = std::env::var("CANVAS_TOKEN").ok().filter(|s| !s.is_empty());
    let ldap_password = std::env::var("CMU_LDAP_PASSWORD")
        .ok()
        .filter(|s| !s.is_empty());

    // Directory enrichment reads existing scrape output and talks only
    // to LDAP, no Stellic session needed
    if args.mode == Mode::Ldap {
        let password = ldap_password.context("CMU_LDAP_PASSWORD required for ldap mode")?;

        return ldap::scrape_ldap(&args.export_root, &password);
    }

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(args.concurrency)
        .build()?;

    let needs_canvas = matches!(args.mode, Mode::Syllabi | Mode::All);

    // Always authenticate against Stellic, even for syllabi-only runs:
    // the AndrewID it returns names the per-student subdir under
    // `export_root` for all output kinds.
    let (stellic, term_joined) =
        Stellic::login(cookie_header, &args.export_root, args.concurrency)?;

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
        let token = canvas_token.context("CANVAS_TOKEN required for syllabi mode")?;
        Some(Canvas::new(token, args.concurrency))
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
        Mode::Ldap => unreachable!("dispatched before Stellic login"),
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

            match ldap_password.as_deref() {
                Some(pw) => ldap::scrape_ldap(&args.export_root, pw)?,
                None => warn!("CMU_LDAP_PASSWORD not set; skipping directory enrichment"),
            }
        }
    }
    Ok(())
}

fn parse_target_term(term: &str) -> Result<(u32, Sem)> {
    let bytes = term.as_bytes();
    if bytes.len() != 3 || !bytes[1..].iter().all(u8::is_ascii_digit) {
        anyhow::bail!("Use a term such as S27, F26, or M27");
    }
    let sem = match bytes[0] {
        b'S' => Sem::Spring,
        b'F' => Sem::Fall,
        b'M' => Sem::Summer,
        _ => anyhow::bail!("Unknown term season"),
    };
    Ok((2000 + term[1..].parse::<u32>()?, sem))
}

fn scrape_syllabi(
    canvas: &Canvas,
    args: &Args,
    syllabi_dir: &std::path::Path,
    pool: &rayon::ThreadPool,
) -> Result<()> {
    let existing = syllabi::load_existing_keys()?;
    info!(existing = existing.len(), "listed CDN syllabi");

    let tasks = pool.install(|| syllabi::build_tasks(canvas))?;

    let total = tasks.len();
    info!(total, concurrency = args.concurrency, "scraping syllabi");

    let done = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);

    pool.install(|| {
        tasks.into_par_iter().for_each(|task| {
            let result =
                syllabi::save_task(canvas, syllabi_dir, &task, args.incremental, &existing);
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

    let target = args.term.as_deref().map(parse_target_term).transpose()?;
    let courses: HashSet<String> = if let Some((year, sem)) = target {
        soc.values()
            .filter(|s| s.matches_term(year, sem))
            .flat_map(|s| s.codes.iter())
            .cloned()
            .collect()
    } else {
        fce.keys()
            .chain(soc.values().flat_map(|s| s.codes.iter()))
            .cloned()
            .collect()
    };
    anyhow::ensure!(
        !courses.is_empty(),
        "No published courses found; refusing an empty refresh"
    );
    let tasks: Vec<Task> = courses
        .into_par_iter()
        .flat_map_iter(|c| {
            if let Some((year, sem)) = target {
                vec![
                    Task::Info(c.clone()),
                    Task::Sections {
                        course: c,
                        lyear: sem.ay_start(year) - anchor + 1,
                        sem_id: match sem {
                            Sem::Fall => 1,
                            Sem::Spring => 2,
                            Sem::Summer => 3,
                        },
                    },
                ]
            } else {
                course_tasks(&c, &fce, &soc, anchor)
            }
        })
        .collect();
    let total = tasks.len();
    info!(total, concurrency = args.concurrency, "scraping courses");

    let done = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);

    let run_task = |task: &Task| -> Result<()> {
        match task {
            Task::Info(course) => stellic.save_info(course, args.incremental),
            Task::Sections {
                course,
                lyear,
                sem_id,
            } => stellic.save_sections(
                course,
                *lyear,
                *sem_id,
                args.incremental,
                target.map(|(year, _)| year),
            ),
        }
    };
    if target.is_some() {
        // Explicit refreshes are serial regardless of the pool setting. Stop
        // the whole run on the first failed request, including 401/403/429.
        for task in &tasks {
            std::thread::sleep(std::time::Duration::from_secs(2));
            done.fetch_add(1, Ordering::Relaxed);
            if let Err(error) = run_task(task) {
                failed.fetch_add(1, Ordering::Relaxed);
                warn!(%error, "targeted refresh stopped on first failure");
                break;
            }
        }
    } else {
        pool.install(|| {
            tasks.par_iter().for_each(|task| {
                done.fetch_add(1, Ordering::Relaxed);
                if let Err(error) = run_task(task) {
                    failed.fetch_add(1, Ordering::Relaxed);
                    debug!(%error, "course request failed");
                }
            })
        });
    }

    info!(
        done = done.load(Ordering::Relaxed),
        failed = failed.load(Ordering::Relaxed),
        "courses complete"
    );
    let failures = failed.load(Ordering::Relaxed);
    let root = args.export_root.join(&stellic.username);
    std::fs::create_dir_all(&root)?;
    std::fs::write(
        root.join("scrape-manifest.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "target_term": args.term, "tasks": total, "completed": done.load(Ordering::Relaxed), "failed": failures,
            "completed_unix_seconds": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs()
        }))?,
    )?;
    if failures > 0 {
        anyhow::bail!("{failures} course requests failed; this scrape is incomplete")
    }
    Ok(())
}

fn scrape_programs(
    stellic: &Stellic,
    args: &Args,
    programs_dir: &std::path::Path,
    pool: &rayon::ThreadPool,
) -> Result<()> {
    let programs = stellic.get_programs()?;
    info!(count = programs.len(), "fetched program catalog");

    let tasks = pool.install(|| requirements::build_tasks(stellic, &programs));
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

#[cfg(test)]
mod term_tests {
    use super::*;
    #[test]
    fn spring_2027_uses_2026_academic_year_and_spring_semester() {
        let (year, sem) = parse_target_term("S27").unwrap();
        assert_eq!(year, 2027);
        assert_eq!(sem, Sem::Spring);
        assert_eq!(sem.ay_start(year), 2026);
        assert!(parse_target_term("S2027").is_err());
        assert!(parse_target_term("X27").is_err());
    }
}
