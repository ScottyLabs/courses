//! Programs pipeline that fans out from the catalog's flat program list into
//! every published audit version per program, then fetches each audit and
//! writes a trimmed `<catalog_id>/<audit_id>.json` per (program, audit).
//! The test-apply audit path sidesteps Stellic's plan and major caps and
//! returns the full requirement tree for any reachable audit in one call.
//!
//! The build-tasks pass parallelizes over programs because
//! `getauditversions` is cheap and mostly IO-bound; the save pass parallelizes
//! over audits because `getauditinfo` is heavy server-side and benefits from
//! pipelining.

use anyhow::Result;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{debug, info, warn};

use crate::stellic::{Program, Stellic};

#[derive(Debug)]
pub struct Task {
    pub program_id: u32,
    pub program_name: String,
    pub program_type: u8,
    pub audit_id: u32,
    pub audit_name: String,
    pub requirement: u64,
}

pub fn build_tasks(stellic: &Stellic, programs: &[Program]) -> Vec<Task> {
    let done = AtomicUsize::new(0);
    let total = programs.len();
    programs
        .par_iter()
        .flat_map_iter(|p| {
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            if n.is_multiple_of(200) {
                info!(done = n, total, "audit version discovery");
            }
            match stellic.get_audit_versions(p.id) {
                Ok(avs) => avs
                    .into_iter()
                    .map(|av| Task {
                        program_id: p.id,
                        program_name: p.name.clone(),
                        program_type: p.program_type,
                        audit_id: av.id,
                        audit_name: av.name,
                        requirement: av.requirement,
                    })
                    .collect::<Vec<_>>(),
                Err(e) => {
                    warn!(program_id = p.id, name = %p.name, error = %e, "audit versions fetch failed");
                    vec![]
                }
            }
        })
        .collect()
}

pub fn save_audit(stellic: &Stellic, dir: &Path, task: &Task) -> Result<()> {
    let resp = stellic.get_audit_data(task.audit_id)?;
    if resp.get("success").and_then(|s| s.as_bool()) == Some(false) {
        return Ok(());
    }
    let Some(programs) = resp
        .get("req_tree")
        .and_then(|t| t.get("programs"))
        .and_then(|p| p.as_array())
    else {
        return Ok(());
    };
    let Some(matching) = programs
        .iter()
        .find(|p| p.get("id").and_then(|i| i.as_u64()) == Some(task.requirement))
    else {
        debug!(
            program_id = task.program_id,
            program = %task.program_name,
            audit_id = task.audit_id,
            audit = %task.audit_name,
            "audit response missing requested audit_version"
        );
        return Ok(());
    };
    let mut matching = matching.clone();
    if let Some(obj) = matching.as_object_mut() {
        obj.remove("uid");
    }
    let mut filtered_ucpm = serde_json::Map::new();
    if let Some(ucpm) = resp
        .get("unique_course_parents_mapping")
        .and_then(|v| v.as_object())
    {
        let target = task.requirement.to_string();
        for (course_req_id, mapping) in ucpm {
            if let Some(padded) = mapping.get(&target) {
                filtered_ucpm.insert(
                    course_req_id.clone(),
                    serde_json::json!({ &target: padded }),
                );
            }
        }
    }
    let program_reqs = resp
        .get("program_reqs")
        .and_then(|v| v.as_object())
        .and_then(|m| m.get(&task.requirement.to_string()).cloned());

    let wrapped = serde_json::json!({
        "catalog_id": task.program_id,
        "program_name": task.program_name,
        "program_type": task.program_type,
        "audit_id": task.audit_id,
        "audit_name": task.audit_name,
        "requirement_id": task.requirement,
        "is_combination": resp.get("is_combination"),
        "free_electives_req": resp.get("free_electives_req"),
        "program_reqs": program_reqs,
        "unique_course_parents_mapping": filtered_ucpm,
        "tree": matching,
    });
    let prog_dir = dir.join(task.program_id.to_string());
    fs::create_dir_all(&prog_dir)?;
    fs::write(
        prog_dir.join(format!("{}.json", task.audit_id)),
        serde_json::to_string(&wrapped)?,
    )?;
    Ok(())
}
