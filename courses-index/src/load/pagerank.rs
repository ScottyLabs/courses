//! Build the prereq graph from each course's `prereqs_text`, run PageRank
//! over it, and write the resulting score back to `Course.pagerank`. The
//! text is parsed with a simple `\d{2}-\d{3}` scan rather than walking the
//! Stellic `req_obj` tree because the text already contains every code the
//! tree does, in a form we treat purely as edges (logical structure does not
//! affect rank flow).
//!
//! Each prereq reference inside course C's text adds an edge `C -> A` so
//! rank flows from C toward its prereqs. A course referenced as a prereq
//! by many downstream courses accumulates rank, which is the popularity
//! and centrality signal we want as a tiebreaker for relevance.

use std::collections::HashMap;
use std::sync::OnceLock;

use regex::Regex;
use tracing::debug;

use crate::doc::Course;

const DAMPING: f32 = 0.85;
const ITERATIONS: usize = 50;
const CONVERGENCE: f32 = 1e-7;

#[derive(Debug, Default)]
pub struct PageRankResult {
    pub n_nodes: usize,
    pub n_edges: usize,
    pub iterations_run: usize,
    pub final_delta: f32,
}

pub fn compute_pagerank(courses: &mut [Course]) -> PageRankResult {
    let code_to_idx: HashMap<&str, usize> = courses
        .iter()
        .enumerate()
        .map(|(i, c)| (c.code.as_str(), i))
        .collect();

    let mut out_edges: Vec<Vec<usize>> = vec![Vec::new(); courses.len()];
    let mut n_edges = 0usize;
    let re = code_regex();
    for (i, course) in courses.iter().enumerate() {
        let Some(text) = course.prereqs_text.as_deref() else {
            continue;
        };
        let mut seen: Vec<usize> = Vec::new();
        for cap in re.find_iter(text) {
            let code = cap.as_str();
            let Some(&j) = code_to_idx.get(code) else {
                continue;
            };
            if i == j {
                continue;
            }
            if !seen.contains(&j) {
                seen.push(j);
            }
        }
        n_edges += seen.len();
        out_edges[i] = seen;
    }

    let n = courses.len();
    let mut rank: Vec<f32> = vec![1.0 / n as f32; n];
    let mut iterations_run = 0;
    let mut final_delta = 0.0;
    let teleport = (1.0 - DAMPING) / n as f32;

    for it in 0..ITERATIONS {
        iterations_run = it + 1;

        let mut next: Vec<f32> = vec![teleport; n];

        let mut dangling_sum = 0.0;
        for i in 0..n {
            if out_edges[i].is_empty() {
                dangling_sum += rank[i];
            } else {
                let share = DAMPING * rank[i] / out_edges[i].len() as f32;
                for &j in &out_edges[i] {
                    next[j] += share;
                }
            }
        }
        let dangling_share = DAMPING * dangling_sum / n as f32;
        for v in next.iter_mut() {
            *v += dangling_share;
        }

        let delta: f32 = next
            .iter()
            .zip(rank.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        final_delta = delta;
        rank = next;
        if delta < CONVERGENCE {
            break;
        }
    }

    for (course, score) in courses.iter_mut().zip(rank.iter()) {
        course.pagerank = *score;
    }

    let result = PageRankResult {
        n_nodes: n,
        n_edges,
        iterations_run,
        final_delta,
    };
    debug!(
        n_nodes = result.n_nodes,
        n_edges = result.n_edges,
        iterations = result.iterations_run,
        delta = result.final_delta,
        "pagerank complete"
    );
    result
}

fn code_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\b\d{2}-\d{3}\b").unwrap())
}
