//! One-shot diagnostics over the loaded corpus. Currently used to size the
//! interning opportunity for `Course.description`. Walks every course's
//! description, counts duplicates and word-level n-grams, then ranks
//! candidate phrases by estimated byte savings under a hypothetical
//! "intern table + 4-byte references" encoding.

use std::collections::HashMap;

use crate::index::Index;

/// Bytes per intern reference at runtime (a `u32` id).
const REF_OVERHEAD: i64 = 4;

/// Per-entry overhead in the intern table beyond the literal bytes
/// (one length prefix and a few bytes of housekeeping).
const TABLE_ENTRY_OVERHEAD: i64 = 4;

const MIN_NGRAM_WORDS: usize = 2;
const MAX_NGRAM_WORDS: usize = 12;
const MIN_FREQUENCY: u32 = 5;
const TOP_PRINT: usize = 60;
const TOP_AGGREGATE: usize = 1000;

pub fn run(index: &Index) {
    let descriptions: Vec<&str> = index
        .courses
        .iter()
        .map(|c| &*c.description)
        .filter(|d| !d.is_empty())
        .collect();
    let total_courses = descriptions.len();
    let total_bytes: usize = descriptions.iter().map(|s| s.len()).sum();
    let avg_len = total_bytes as f64 / total_courses.max(1) as f64;

    println!("\n=== description corpus stats ===");
    println!("courses with non-empty description: {total_courses}");
    println!("total description bytes:            {total_bytes}");
    println!("avg length:                         {avg_len:.0} bytes");

    println!("\n--- duplicate full strings ---");
    let mut full: HashMap<&str, u32> = HashMap::new();
    for d in &descriptions {
        *full.entry(*d).or_insert(0) += 1;
    }
    let mut dup_candidates: Vec<(&&str, &u32)> = full.iter().filter(|(_, c)| **c > 1).collect();
    dup_candidates.sort_by_key(|(s, c)| -((**c as i64 - 1) * s.len() as i64));
    let dup_savings: i64 = dup_candidates
        .iter()
        .map(|(s, c)| (**c as i64 - 1) * s.len() as i64)
        .sum();
    println!(
        "{} unique strings appear >1 time (max savings if dedup'd: {} bytes, {:.1}% of corpus)",
        dup_candidates.len(),
        dup_savings,
        (dup_savings as f64 / total_bytes as f64) * 100.0
    );
    for (s, c) in dup_candidates.iter().take(10) {
        let p = preview(s, 80);
        println!("  freq={:>4} len={:>5} {p:?}", c, s.len());
    }

    println!("\n--- n-gram intern candidates (word-level) ---");
    let mut counts: HashMap<String, u32> = HashMap::with_capacity(2_000_000);
    for d in &descriptions {
        let words: Vec<&str> = d.split_whitespace().collect();
        for n in MIN_NGRAM_WORDS..=MAX_NGRAM_WORDS {
            if words.len() < n {
                break;
            }
            for window in words.windows(n) {
                let phrase = window.join(" ");
                *counts.entry(phrase).or_insert(0) += 1;
            }
        }
    }
    println!(
        "distinct n-grams (n={}..={}): {}",
        MIN_NGRAM_WORDS,
        MAX_NGRAM_WORDS,
        counts.len()
    );

    let mut candidates: Vec<(String, u32, i64)> = counts
        .into_iter()
        .filter(|(_, freq)| *freq >= MIN_FREQUENCY)
        .map(|(s, freq)| {
            let len = s.len() as i64;
            let saving = (freq as i64) * (len - REF_OVERHEAD) - (len + TABLE_ENTRY_OVERHEAD);
            (s, freq, saving)
        })
        .filter(|(_, _, saving)| *saving > 0)
        .collect();
    candidates.sort_by(|a, b| b.2.cmp(&a.2));

    println!("\n{:>9}  {:>6}  {:>5}  phrase", "savings", "freq", "len");
    for (phrase, freq, saving) in candidates.iter().take(TOP_PRINT) {
        let preview = preview(phrase, 100);
        println!("{saving:>9}  {freq:>6}  {:>5}  {preview:?}", phrase.len());
    }

    let topk_saving: i64 = candidates
        .iter()
        .take(TOP_AGGREGATE)
        .map(|(_, _, s)| *s)
        .sum();
    println!(
        "\ntop-{} candidates total: {} bytes ({:.1}% of {} corpus bytes)",
        TOP_AGGREGATE,
        topk_saving,
        (topk_saving as f64 / total_bytes as f64) * 100.0,
        total_bytes
    );
    println!(
        "(savings overlap when phrases nest, so the realistic win is lower; treat this as an upper bound for greedy interning)"
    );

    println!("\n--- n-gram length distribution among candidates ---");
    let mut by_len: HashMap<usize, (u32, i64)> = HashMap::new();
    for (phrase, _, saving) in &candidates {
        let words = phrase.split_whitespace().count();
        let entry = by_len.entry(words).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += saving;
    }
    let mut by_len_v: Vec<_> = by_len.into_iter().collect();
    by_len_v.sort_by_key(|(n, _)| *n);
    println!(
        "{:>5}  {:>10}  {:>12}",
        "words", "n_phrases", "total_saving"
    );
    for (n, (count, saving)) in &by_len_v {
        println!("{n:>5}  {count:>10}  {saving:>12}");
    }
}

fn preview(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{truncated}...")
    }
}
