//! Native CLI for the catalog pipeline. Walks `exported/` and `data/fces.csv`
//! into a `Corpus`, builds the in-memory index, and exposes flags for the
//! various development workflows attached to it: catalog read/write, patch
//! generation, ad-hoc queries, schedule helpers, and the bench harness.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber::EnvFilter;

use courses_index::{
    binary,
    index::{FacetAxis, FacetFilters, Index, Query, Searcher, SortOrder},
    load,
};

#[derive(Parser, Debug)]
#[command(about = "Build the courses-index binary artifact")]
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

    /// Run a text query against the index and print the top hits.
    #[arg(long)]
    search: Option<String>,

    /// Allow Levenshtein-1 fuzzy matches in `--search`.
    #[arg(long)]
    fuzzy: bool,

    /// Restrict `--search` results to this department (e.g. `--dept 15`).
    #[arg(long)]
    dept: Option<String>,

    /// Run a benchmark of typical queries and print timings.
    #[arg(long)]
    bench: bool,

    /// Smoke-test the schedule helpers and print a sample window result.
    #[arg(long)]
    schedule_demo: bool,

    /// Write the loaded corpus to this path as a `.bin` file.
    #[arg(long)]
    write_catalog: Option<PathBuf>,

    /// Read the corpus from this path instead of walking `exported/`.
    #[arg(long)]
    read_catalog: Option<PathBuf>,

    /// Don't compress the written catalog (useful for diff inspection).
    #[arg(long)]
    no_compress: bool,

    /// Generate a zstd patch from `--old-catalog` to `--new-catalog`, written
    /// to this path. Server-side step before publishing a weekly delta.
    #[arg(long, requires_all = ["old_catalog", "new_catalog"])]
    write_patch: Option<PathBuf>,

    /// Apply `--patch` against `--old-catalog`, writing the reconstructed
    /// catalog to this path. Sanity check for the patch round-trip.
    #[arg(long, requires_all = ["old_catalog", "patch"])]
    apply_patch: Option<PathBuf>,

    #[arg(long)]
    old_catalog: Option<PathBuf>,

    #[arg(long)]
    new_catalog: Option<PathBuf>,

    #[arg(long)]
    patch: Option<PathBuf>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    if let Some(out_path) = &args.write_patch {
        let old = args.old_catalog.as_ref().expect("clap requires");
        let new = args.new_catalog.as_ref().expect("clap requires");
        let t0 = std::time::Instant::now();
        binary::write_patch(old, new, out_path)?;
        let bytes = std::fs::metadata(out_path)?.len();
        info!(
            ms = t0.elapsed().as_secs_f64() * 1e3,
            bytes,
            from = %old.display(),
            to = %new.display(),
            patch = %out_path.display(),
            "patch written"
        );
        return Ok(());
    }

    if let Some(out_path) = &args.apply_patch {
        let old = args.old_catalog.as_ref().expect("clap requires");
        let patch = args.patch.as_ref().expect("clap requires");
        let t0 = std::time::Instant::now();
        let bytes = binary::apply_patch(old, patch)?;
        std::fs::write(out_path, &bytes)?;
        info!(
            ms = t0.elapsed().as_secs_f64() * 1e3,
            bytes = bytes.len(),
            old = %old.display(),
            patch = %patch.display(),
            out = %out_path.display(),
            "patch applied"
        );
        return Ok(());
    }

    let (corpus, prebuilt_text) = if let Some(path) = &args.read_catalog {
        let t0 = std::time::Instant::now();
        let payload = binary::read_catalog(path)?;
        info!(
            ms = t0.elapsed().as_secs_f64() * 1e3,
            path = %path.display(),
            has_prebuilt_text = payload.prebuilt_text.is_some(),
            "catalog loaded from binary"
        );
        (payload.corpus, payload.prebuilt_text)
    } else {
        let corpus = load::run(&args.exported_root, &args.fce_csv)?;
        info!(
            courses = corpus.courses.len(),
            professors = corpus.professors.len(),
            sections = corpus.sections.len(),
            fce_rows = corpus.fce_rows.len(),
            "load complete"
        );
        (corpus, None)
    };

    let build_t0 = std::time::Instant::now();
    let index = match prebuilt_text {
        Some(p) => Index::build_with_prebuilt_text(corpus, p)?,
        None => Index::build(corpus),
    };
    let build_elapsed = build_t0.elapsed();

    if let Some(path) = &args.write_catalog {
        let corpus_view = courses_index::load::Corpus {
            courses: index.courses.clone(),
            professors: index.professors.clone(),
            sections: index.schedule.sections.clone(),
            fce_rows: index.fce_rows.clone(),
        };
        let prebuilt = index.text.to_prebuilt();
        let t0 = std::time::Instant::now();
        binary::write_catalog(path, &corpus_view, Some(&prebuilt), !args.no_compress)?;
        let bytes = std::fs::metadata(path)?.len();
        info!(
            ms = t0.elapsed().as_secs_f64() * 1e3,
            bytes,
            path = %path.display(),
            "catalog written"
        );
    }
    info!(
        ms = build_elapsed.as_secs_f64() * 1e3,
        terms_code = index.text.code.n_terms(),
        terms_name = index.text.name.n_terms(),
        terms_description = index.text.description.n_terms(),
        terms_instr = index.text.instructor_names.n_terms(),
        facet_pairs = index.facets.cardinality(),
        "index built"
    );

    if let Some(code) = args.pick {
        match index.courses.iter().find(|c| c.code == code) {
            Some(c) => {
                let n_sections = index.schedule.sections_for_course(c.id).len();
                println!("{:#?}", c);
                println!("section_rows_for_course = {n_sections}");
            }
            None => eprintln!("no course with code {code}"),
        }
    }

    if let Some(needle) = args.prof {
        let lower = needle.to_lowercase();
        match index
            .professors
            .iter()
            .find(|p| p.name.to_lowercase().contains(&lower))
        {
            Some(p) => {
                let n_fce = index
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

    if let Some(text) = &args.search {
        let mut q = Query {
            text: Some(text.clone()),
            fuzzy: args.fuzzy,
            limit: 10,
            sort: SortOrder::Relevance,
            ..Query::default()
        };
        if let Some(d) = args.dept {
            q.facets = FacetFilters {
                dept: vec![d],
                ..FacetFilters::default()
            };
        }
        let t0 = std::time::Instant::now();
        let res = index.query(&q);
        let elapsed_us = t0.elapsed().as_micros();
        println!(
            "\nQuery {:?} yielded {} hits in {} µs",
            text, res.total_matched, elapsed_us
        );
        for h in res.hits {
            let c = &index.courses[h.course_id as usize];
            println!("  {:.4}  {:8}  {}", h.score, c.code, c.name);
        }
        if !res.did_you_mean_codes.is_empty() {
            println!("did you mean: {}", res.did_you_mean_codes.join(", "));
        }
    }

    if args.bench {
        run_bench(&index);
    }

    if args.schedule_demo {
        run_schedule_demo(&index);
    }

    let mut top: Vec<(&str, &str, f32)> = index
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

fn run_schedule_demo(index: &Index) {
    use courses_index::doc::{Sem, Term};

    use std::collections::BTreeMap;
    let mut counts: BTreeMap<(u16, u8), usize> = BTreeMap::new();
    for s in &index.schedule.sections {
        *counts.entry((s.term.year, s.term.sem as u8)).or_insert(0) += 1;
    }
    let term_label = |sem_u8: u8| match sem_u8 {
        1 => "Fall",
        2 => "Spring",
        3 => "Summer",
        _ => "?",
    };
    println!("\nsections per term:");
    for ((y, s), n) in counts.iter().rev().take(8) {
        println!("  {y} {} : {n}", term_label(*s));
    }

    let term = Term {
        year: 2025,
        sem: Sem::Spring,
    };
    // Mon-Wed-Fri from 9:00 to 12:30. Day bitmask uses bit 0=Mon, 2=Wed, 4=Fri.
    let days = (1u8 << 0) | (1 << 2) | (1 << 4);
    let start = 9 * 60;
    let end = 12 * 60 + 30;

    let t0 = std::time::Instant::now();
    let hits = index.schedule.schedule_fit(term, days, start, end);
    let elapsed = t0.elapsed();
    println!(
        "\nschedule_fit(S25, MWF 9:00-12:30) yielded {} sections in {} µs",
        hits.len(),
        elapsed.as_micros()
    );
    for sid in hits.iter().take(8) {
        let s = index
            .schedule
            .sections
            .iter()
            .find(|s| s.section_id == *sid)
            .unwrap();
        let cm = s.course_id;
        let code = &index.courses[cm as usize].code;
        println!(
            "  {code}  start={:02}:{:02}  end={:02}:{:02}  days_mask={:07b}",
            s.start_minutes / 60,
            s.start_minutes % 60,
            s.end_minutes / 60,
            s.end_minutes % 60,
            s.days
        );
    }

    if let (Some(a), Some(b), Some(c)) = (
        index.code_to_id.get("15-122"),
        index.code_to_id.get("21-241"),
        index.code_to_id.get("21-242"),
    ) {
        let t0 = std::time::Instant::now();
        let collide_ab = index.schedule.courses_overlap(*a, *b);
        let collide_ac = index.schedule.courses_overlap(*a, *c);
        let collide_bc = index.schedule.courses_overlap(*b, *c);
        let elapsed = t0.elapsed();
        println!(
            "\ncourses_overlap (3 pairs) computed in {} µs",
            elapsed.as_micros()
        );
        println!("  15-122 vs 21-241: {collide_ab}");
        println!("  15-122 vs 21-242: {collide_ac}");
        println!("  21-241 vs 21-242: {collide_bc}");
    }
}

fn run_bench(index: &Index) {
    let queries: &[(&str, Query)] = &[
        (
            "text:linear algebra",
            Query {
                text: Some("linear algebra".into()),
                limit: 10,
                ..Query::default()
            },
        ),
        (
            "text:machine learning",
            Query {
                text: Some("machine learning".into()),
                limit: 10,
                ..Query::default()
            },
        ),
        (
            "text:15-122",
            Query {
                text: Some("15-122".into()),
                limit: 10,
                ..Query::default()
            },
        ),
        (
            "text:imperative",
            Query {
                text: Some("imperative".into()),
                limit: 10,
                ..Query::default()
            },
        ),
        (
            "filter:dept=15",
            Query {
                facets: FacetFilters {
                    dept: vec!["15".into()],
                    ..Default::default()
                },
                limit: 10,
                sort: SortOrder::PageRankDesc,
                ..Query::default()
            },
        ),
        (
            "text+filter:algorithms dept=15",
            Query {
                text: Some("algorithms".into()),
                facets: FacetFilters {
                    dept: vec!["15".into()],
                    ..Default::default()
                },
                limit: 10,
                ..Query::default()
            },
        ),
        (
            "browse:pagerank top 50",
            Query {
                limit: 50,
                sort: SortOrder::PageRankDesc,
                ..Query::default()
            },
        ),
        (
            "text+counts:ml + dept,level",
            Query {
                text: Some("machine learning".into()),
                limit: 10,
                count_facets: vec![FacetAxis::Dept, FacetAxis::Level, FacetAxis::AttributeTags],
                ..Query::default()
            },
        ),
        (
            "browse+counts:dept,level,programs",
            Query {
                limit: 10,
                sort: SortOrder::PageRankDesc,
                count_facets: vec![FacetAxis::Dept, FacetAxis::Level, FacetAxis::ProgramId],
                ..Query::default()
            },
        ),
    ];

    let mut uncached = Searcher::with_cache_capacity(index.n_docs as usize, 0);
    let mut cached = Searcher::new(index.n_docs as usize);
    println!(
        "\n{:<35}  {:>10}  {:>10}  {:>10}  {:>10}",
        "query", "hits", "raw p50", "cached p50", "p99"
    );
    println!("{}", "-".repeat(84));
    for (label, q) in queries {
        for _ in 0..50 {
            let _ = uncached.query(index, q);
        }
        let mut raw_samples: Vec<f64> = Vec::with_capacity(2000);
        for _ in 0..2000 {
            let t0 = std::time::Instant::now();
            let _ = uncached.query(index, q);
            raw_samples.push(t0.elapsed().as_nanos() as f64 / 1000.0);
        }
        raw_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let raw_p50 = raw_samples[raw_samples.len() / 2];
        let raw_p99 = raw_samples[(raw_samples.len() as f64 * 0.99) as usize];

        // Warm cache and measure cached repeats.
        let _ = cached.query(index, q);
        let mut cached_samples: Vec<f64> = Vec::with_capacity(2000);
        for _ in 0..2000 {
            let t0 = std::time::Instant::now();
            let _ = cached.query(index, q);
            cached_samples.push(t0.elapsed().as_nanos() as f64 / 1000.0);
        }
        cached_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let cached_p50 = cached_samples[cached_samples.len() / 2];

        let res = uncached.query(index, q);
        println!(
            "{:<35}  {:>10}  {:>10.2}  {:>10.2}  {:>10.2}",
            label, res.total_matched, raw_p50, cached_p50, raw_p99
        );
    }
}
