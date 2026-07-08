//! Walk every `exported/programs/<catalog_id>/<audit_id>.json`, collect the
//! set of course codes referenced by the audit's `tree`, and attach a
//! `ProgramMembership` to each matching course. The audit JSON's `tree` is a
//! deeply nested choice/constraint structure, so the walk just looks for
//! `{"type": "course", "data": {"course": {"code": ...}}}` nodes regardless
//! of nesting depth.
//!
//! A single course can appear multiple times within one audit's tree (listed
//! under several requirement branches); the per-audit set is deduplicated
//! before emission. Across audits, the same course typically belongs to many
//! programs, so the resulting `Course.programs` is an unordered list of
//! `(audit, program)` tuples.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, warn};
use walkdir::WalkDir;

use crate::doc::{Course, CourseId, ProgramMembership};

#[derive(Debug, Default)]
pub struct ProgramLoad {
    pub files_scanned: usize,
    pub files_skipped: usize,
    pub memberships_attached: usize,
    pub orphan_codes: HashSet<String>,
}

/// Walk `programs_root` and populate each course's `programs` field with one
/// `ProgramMembership` per audit it appears in.
pub fn load_programs(programs_root: &Path, courses: &mut [Course]) -> Result<ProgramLoad> {
    let code_to_id: HashMap<String, CourseId> =
        courses.iter().map(|c| (c.code.clone(), c.id)).collect();

    let files: Vec<_> = WalkDir::new(programs_root)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.file_name()
                    .to_str()
                    .map(|n| n.ends_with(".json"))
                    .unwrap_or(false)
        })
        .collect();

    let parsed: Vec<ParsedAudit> = files
        .par_iter()
        .filter_map(|e| parse_one(e.path()))
        .collect();

    let mut out = ProgramLoad {
        files_scanned: files.len(),
        files_skipped: files.len() - parsed.len(),
        ..Default::default()
    };

    let mut by_course: HashMap<CourseId, Vec<ProgramMembership>> = HashMap::new();
    for audit in parsed {
        for code in &audit.course_codes {
            let Some(&course_id) = code_to_id.get(code) else {
                out.orphan_codes.insert(code.clone());
                continue;
            };
            by_course
                .entry(course_id)
                .or_default()
                .push(ProgramMembership {
                    program_id: audit.program_id,
                    program_name: audit.program_name.clone(),
                    audit_id: audit.audit_id,
                    catalog_id: audit.catalog_id,
                    program_type: audit.program_type,
                });
        }
    }

    for course in courses.iter_mut() {
        if let Some(mut memberships) = by_course.remove(&course.id) {
            memberships.sort_by(|a, b| {
                (a.catalog_id, a.audit_id, a.program_id).cmp(&(
                    b.catalog_id,
                    b.audit_id,
                    b.program_id,
                ))
            });
            out.memberships_attached += memberships.len();
            course.programs = memberships;
        }
    }

    debug!(
        scanned = out.files_scanned,
        skipped = out.files_skipped,
        attached = out.memberships_attached,
        orphans = out.orphan_codes.len(),
        "programs load complete"
    );
    Ok(out)
}

struct ParsedAudit {
    audit_id: u32,
    catalog_id: u32,
    program_id: u32,
    program_name: String,
    program_type: u8,
    course_codes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AuditEnvelope {
    audit_id: u32,
    catalog_id: u32,
    program_name: String,
    program_type: u8,
    program_reqs: ProgramReqs,
    tree: Value,
}

#[derive(Debug, Deserialize)]
struct ProgramReqs {
    id: u32,
}

fn parse_one(path: &Path) -> Option<ParsedAudit> {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(err) => {
            warn!(?err, path = %path.display(), "audit read failed");
            return None;
        }
    };
    let env: AuditEnvelope = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(err) => {
            warn!(?err, path = %path.display(), "audit parse failed");
            return None;
        }
    };

    let mut codes: HashSet<String> = HashSet::new();
    collect_course_codes(&env.tree, &mut codes);

    Some(ParsedAudit {
        audit_id: env.audit_id,
        catalog_id: env.catalog_id,
        program_id: env.program_reqs.id,
        program_name: env.program_name,
        program_type: env.program_type,
        course_codes: codes.into_iter().collect(),
    })
}

fn collect_course_codes(node: &Value, out: &mut HashSet<String>) {
    match node {
        Value::Object(map) => {
            if map.get("type").and_then(|v| v.as_str()) == Some("course")
                && let Some(course) = map.get("data").and_then(|d| d.get("course"))
                && let Some(code) = course.get("code").and_then(|v| v.as_str())
                && !code.is_empty()
            {
                out.insert(code.to_string());
            }
            for v in map.values() {
                collect_course_codes(v, out);
            }
        }
        Value::Array(items) => {
            for v in items {
                collect_course_codes(v, out);
            }
        }
        _ => {}
    }
}
