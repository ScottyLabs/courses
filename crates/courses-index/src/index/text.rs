//! Per-field inverted text index. Each field gets an FST term dictionary
//! plus a flat posting-list arena. Scores are baked at build time:
//! `field_weight * idf * (tf * (k1+1)) / (tf + k1 * (1 - b + b * dl/avg))`
//! lands in the arena as a single `f32`, so query-time scoring is just
//! "read 8 bytes per posting, add to accumulator."
//!
//! Each posting list starts with a `u32 df` followed by `df` entries of
//! `(u32 doc_id, f32 score)`, so each posting takes 8 bytes.

use std::collections::BTreeMap;

use fst::automaton::{Levenshtein, Str};
use fst::{Automaton, IntoStreamer, Map, MapBuilder, Streamer};
use serde::{Deserialize, Serialize};

use super::tokenize::{is_partial_code, tokenize_code, tokenize_description, tokenize_general};
use crate::doc::Course;

const FUZZY_PENALTY: f32 = 0.5;
const FUZZY_MIN_TOKEN_LEN: usize = 4;

const K1: f32 = 1.2;
const B: f32 = 0.75;

#[derive(Debug, Clone, Copy)]
pub struct FieldWeights {
    pub code: f32,
    pub name: f32,
    pub description: f32,
    pub instructor_names: f32,
    pub prereqs_text: f32,
}

impl Default for FieldWeights {
    fn default() -> Self {
        Self {
            code: 5.0,
            name: 2.0,
            description: 1.0,
            instructor_names: 1.5,
            prereqs_text: 0.3,
        }
    }
}

/// Wire format of one text field. Just the two byte buffers, since
/// everything else can be reconstructed in microseconds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrebuiltField {
    pub fst_bytes: Vec<u8>,
    pub arena: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrebuiltText {
    pub code: PrebuiltField,
    pub name: PrebuiltField,
    pub description: PrebuiltField,
    pub instructor_names: PrebuiltField,
    pub prereqs_text: PrebuiltField,
}

pub struct TextField {
    fst: Map<Vec<u8>>,
    arena: Vec<u8>,
}

impl TextField {
    pub fn n_terms(&self) -> u64 {
        self.fst.len() as u64
    }

    pub fn arena_bytes(&self) -> usize {
        self.arena.len()
    }

    pub fn from_prebuilt(p: PrebuiltField) -> anyhow::Result<Self> {
        let fst = Map::new(p.fst_bytes).map_err(|e| anyhow::anyhow!("malformed FST bytes: {e}"))?;
        Ok(TextField {
            fst,
            arena: p.arena,
        })
    }

    pub fn to_prebuilt(&self) -> PrebuiltField {
        PrebuiltField {
            fst_bytes: self.fst.as_fst().as_bytes().to_vec(),
            arena: self.arena.clone(),
        }
    }

    fn lookup(&self, term: &str) -> Option<&[u8]> {
        let offset = self.fst.get(term.as_bytes())? as usize;
        self.posting_at(offset)
    }

    fn posting_at(&self, offset: usize) -> Option<&[u8]> {
        let df = u32::from_le_bytes(self.arena[offset..offset + 4].try_into().ok()?);
        let start = offset + 4;
        let end = start + df as usize * 8;
        Some(&self.arena[start..end])
    }

    fn for_each_fuzzy(
        &self,
        term: &str,
        edit_distance: u32,
        skip_exact: bool,
        mut emit: impl FnMut(&[u8], &[u8]),
    ) {
        let Ok(lev) = Levenshtein::new(term, edit_distance) else {
            return;
        };
        let mut stream = self.fst.search(&lev).into_stream();
        let term_bytes = term.as_bytes();
        while let Some((key, offset)) = stream.next() {
            if skip_exact && key == term_bytes {
                continue;
            }
            if let Some(bytes) = self.posting_at(offset as usize) {
                emit(key, bytes);
            }
        }
    }

    fn for_each_prefix(&self, prefix: &str, mut emit: impl FnMut(&[u8], &[u8])) {
        let aut = Str::new(prefix).starts_with();
        let mut stream = self.fst.search(aut).into_stream();
        while let Some((key, offset)) = stream.next() {
            if let Some(bytes) = self.posting_at(offset as usize) {
                emit(key, bytes);
            }
        }
    }
}

pub struct TextIndex {
    pub code: TextField,
    pub name: TextField,
    pub description: TextField,
    pub instructor_names: TextField,
    pub prereqs_text: TextField,
    pub n_docs: u32,
    pub weights: FieldWeights,
}

impl TextIndex {
    pub fn build(courses: &[Course]) -> Self {
        let n = courses.len() as u32;
        let weights = FieldWeights::default();

        let code = build_field(courses, n, weights.code, |c| tokenize_code(&c.code));
        let name = build_field(courses, n, weights.name, |c| tokenize_general(&c.name));
        let description = build_field(courses, n, weights.description, |c| {
            tokenize_description(&c.description)
        });
        let instructor_names = build_field(courses, n, weights.instructor_names, |c| {
            let joined: String = c
                .instructors_recent
                .iter()
                .map(|i| i.name.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            tokenize_general(&joined)
        });
        let prereqs_text = build_field(courses, n, weights.prereqs_text, |c| {
            tokenize_general(c.prereqs_text.as_deref().unwrap_or(""))
        });

        TextIndex {
            code,
            name,
            description,
            instructor_names,
            prereqs_text,
            n_docs: n,
            weights,
        }
    }

    pub fn from_prebuilt(n_docs: u32, p: PrebuiltText) -> anyhow::Result<Self> {
        Ok(TextIndex {
            code: TextField::from_prebuilt(p.code)?,
            name: TextField::from_prebuilt(p.name)?,
            description: TextField::from_prebuilt(p.description)?,
            instructor_names: TextField::from_prebuilt(p.instructor_names)?,
            prereqs_text: TextField::from_prebuilt(p.prereqs_text)?,
            n_docs,
            weights: FieldWeights::default(),
        })
    }

    pub fn to_prebuilt(&self) -> PrebuiltText {
        PrebuiltText {
            code: self.code.to_prebuilt(),
            name: self.name.to_prebuilt(),
            description: self.description.to_prebuilt(),
            instructor_names: self.instructor_names.to_prebuilt(),
            prereqs_text: self.prereqs_text.to_prebuilt(),
        }
    }

    /// Walk every (field, term) posting list for the tokens in `query` and
    /// invoke `emit(doc_id, baked_score)` for each entry. Allocates only the
    /// tokens vec. When a token looks like an incomplete course code (`21-`,
    /// `21-2`, `21-24`) the code field is also walked as a prefix
    /// automaton so the user gets every matching full code while typing.
    pub fn for_each_exact_posting(&self, query: &str, mut emit: impl FnMut(u32, f32)) {
        let tokens = tokenize_general(query);
        let fields: [&TextField; 5] = [
            &self.code,
            &self.name,
            &self.description,
            &self.instructor_names,
            &self.prereqs_text,
        ];
        for token in &tokens {
            for field in fields {
                if let Some(bytes) = field.lookup(token) {
                    walk_postings(bytes, 1.0, &mut emit);
                }
            }
            if is_partial_code(token) {
                self.code.for_each_prefix(token, |_key, bytes| {
                    walk_postings(bytes, 1.0, &mut emit);
                });
            }
        }
    }

    /// Walk Levenshtein-N neighbors of each non-code token across the name
    /// and instructor fields, emitting postings at a reduced score. Caller
    /// should only invoke this if [`Self::for_each_exact_posting`] returned
    /// no hits, since the FST traversal is much more expensive than exact
    /// lookup.
    pub fn for_each_fuzzy_posting(&self, query: &str, mut emit: impl FnMut(u32, f32)) {
        let tokens = tokenize_general(query);
        // Description is too noisy for typo correction and the code and
        // prereqs_text fields aren't structurally meaningful fuzzy targets,
        // so name and instructor_names carry the whole fuzzy expansion.
        let fields: [&TextField; 2] = [&self.name, &self.instructor_names];
        for token in &tokens {
            if token.len() < FUZZY_MIN_TOKEN_LEN || token.contains('-') {
                continue;
            }
            let edit_distance = if token.len() >= 5 { 2 } else { 1 };
            for field in fields {
                field.for_each_fuzzy(token, edit_distance, false, |_key, bytes| {
                    walk_postings(bytes, FUZZY_PENALTY, &mut emit);
                });
            }
        }
    }

    /// Iterate every term across all fields. Diagnostics only.
    pub fn all_terms(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for field in [
            &self.code,
            &self.name,
            &self.description,
            &self.instructor_names,
            &self.prereqs_text,
        ] {
            let mut stream = field.fst.stream().into_stream();
            while let Some((k, _)) = stream.next() {
                if let Ok(s) = std::str::from_utf8(k) {
                    out.push(s.to_string());
                }
            }
        }
        out.sort();
        out.dedup();
        out
    }
}

fn walk_postings(bytes: &[u8], scale: f32, emit: &mut impl FnMut(u32, f32)) {
    for chunk in bytes.chunks_exact(8) {
        let doc_id = u32::from_le_bytes(chunk[0..4].try_into().unwrap());
        let score = f32::from_le_bytes(chunk[4..8].try_into().unwrap());
        emit(doc_id, score * scale);
    }
}

fn build_field<F>(
    courses: &[Course],
    n_docs: u32,
    field_weight: f32,
    mut tokens_for: F,
) -> TextField
where
    F: FnMut(&Course) -> Vec<String>,
{
    let mut doc_lengths: Vec<u32> = vec![0; n_docs as usize];
    // term -> Vec<(doc_id, tf)>
    let mut postings: BTreeMap<String, Vec<(u32, u32)>> = BTreeMap::new();

    for course in courses {
        let toks = tokens_for(course);
        let id = course.id;
        doc_lengths[id as usize] = toks.len() as u32;
        let mut tf_in_doc: BTreeMap<String, u32> = BTreeMap::new();
        for tok in toks {
            *tf_in_doc.entry(tok).or_insert(0) += 1;
        }
        for (term, tf) in tf_in_doc {
            postings.entry(term).or_default().push((id, tf));
        }
    }

    let total_len: u64 = doc_lengths.iter().map(|&v| v as u64).sum();
    let avg_doc_len = if n_docs == 0 {
        1.0
    } else {
        total_len as f32 / n_docs as f32
    };

    let mut arena: Vec<u8> = Vec::new();
    let mut builder = MapBuilder::memory();
    for (term, mut entries) in postings {
        entries.sort_by_key(|(doc_id, _)| *doc_id);
        let df = entries.len() as u32;
        let n = n_docs as f32;
        let idf = ((n - df as f32 + 0.5) / (df as f32 + 0.5) + 1.0).ln();
        let offset = arena.len() as u64;
        arena.extend_from_slice(&df.to_le_bytes());
        for (doc_id, tf) in entries {
            let tf = tf as f32;
            let dl = doc_lengths[doc_id as usize] as f32;
            let len_norm = 1.0 - B + B * dl / avg_doc_len.max(1.0);
            let tf_score = tf * (K1 + 1.0) / (tf + K1 * len_norm);
            let baked = field_weight * idf * tf_score;
            arena.extend_from_slice(&doc_id.to_le_bytes());
            arena.extend_from_slice(&baked.to_le_bytes());
        }
        builder.insert(term.as_bytes(), offset).unwrap();
    }
    let bytes = builder.into_inner().unwrap();
    let fst = Map::new(bytes).unwrap();

    TextField { fst, arena }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::*;

    fn dummy(id: u32, code: &str, name: &str, desc: &str) -> Course {
        Course {
            id,
            code: code.to_string(),
            dept: code
                .split_once('-')
                .map(|(d, _)| d.to_string())
                .unwrap_or_default(),
            course_num: 0,
            name: name.to_string(),
            description: desc.into(),
            units: 9.0,
            units_min: 9.0,
            units_max: 9.0,
            school: None,
            level: None,
            attribute_tags: Vec::new(),
            gened_tags: Vec::new(),
            skills: Vec::new(),
            prereqs_text: None,
            coreq_codes: Vec::new(),
            antireq_codes: Vec::new(),
            equiv_codes: Vec::new(),
            website: None,
            campuses: Vec::new(),
            semesters_offered: Vec::new(),
            instructors_recent: Vec::new(),
            programs: Vec::new(),
            fce_aggregates: None,
            has_syllabus_terms: Vec::new(),
            pagerank: 0.0,
        }
    }

    #[test]
    fn matches_by_name() {
        let courses = vec![
            dummy(0, "21-241", "Matrix Algebra", "Vectors and matrices"),
            dummy(1, "21-269", "Vector Analysis", "Multivariable calculus"),
            dummy(2, "15-122", "Imperative Computation", "Pointers and arrays"),
        ];
        let idx = TextIndex::build(&courses);
        let mut acc: Vec<(u32, f32)> = Vec::new();
        idx.for_each_exact_posting("matrix", |d, s| acc.push((d, s)));
        let mut sums: std::collections::HashMap<u32, f32> = std::collections::HashMap::new();
        for (d, s) in acc {
            *sums.entry(d).or_insert(0.0) += s;
        }
        let mut hits: Vec<_> = sums.into_iter().collect();
        hits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        assert_eq!(hits[0].0, 0);
    }

    #[test]
    fn matches_by_code() {
        let courses = vec![
            dummy(0, "15-122", "PIC", "intro"),
            dummy(1, "15-150", "Functional", "intro"),
        ];
        let idx = TextIndex::build(&courses);
        let mut hit_ids: Vec<u32> = Vec::new();
        idx.for_each_exact_posting("15-122", |d, _| hit_ids.push(d));
        assert!(hit_ids.contains(&0));
    }
}
