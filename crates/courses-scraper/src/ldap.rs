//! CMU directory (LDAP) enrichment for instructors. One authenticated
//! bind against `ldaps://ldap.cmu.edu` as the scottylabs-svc service
//! account, then batched OR-filter searches over every instructor
//! AndrewID present in the existing scrape output under `export_root`,
//! writing one aggregate `ldap.json` at the export root keyed by
//! lowercased AndrewID.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use ldap3::{LdapConn, Scope, SearchEntry};
use tracing::{info, warn};

const LDAP_URL: &str = "ldaps://ldap.cmu.edu";
const BIND_DN: &str = "uid=scottylabs-svc,ou=andrewperson,dc=andrew,dc=cmu,dc=edu";
const SEARCH_BASE: &str = "ou=andrewperson,dc=andrew,dc=cmu,dc=edu";
const BATCH_SIZE: usize = 50;

/// Attributes worth keeping, requested server-side so the response
/// never carries more than we save. `cmuDepartment`,
/// `eduPersonAffiliation`, and `eduPersonSchoolCollegeName` are
/// multi-valued for some people; every attribute is stored as a list.
const ATTRS: &[&str] = &[
    "uid",
    "cn",
    "displayName",
    "givenName",
    "sn",
    "title",
    "cmuDepartment",
    "eduPersonPrimaryAffiliation",
    "eduPersonAffiliation",
    "eduPersonSchoolCollegeName",
    "mail",
    "telephoneNumber",
    "postalAddress",
    "eduPersonOrcid",
    "guid",
];

/// Look up every known instructor AndrewID in the CMU directory and
/// write the aggregate to `<export_root>/ldap.json`. Re-runs overwrite.
pub fn scrape_ldap(export_root: &Path, password: &str) -> Result<()> {
    let uids = collect_instructor_uids(export_root)?;
    if uids.is_empty() {
        warn!(
            "no instructor AndrewIDs found under {}; run a courses scrape first",
            export_root.display()
        );
        return Ok(());
    }

    info!(uids = uids.len(), "collected instructor AndrewIDs");

    let mut conn = LdapConn::new(LDAP_URL)?;
    conn.simple_bind(BIND_DN, password)?
        .success()
        .context("LDAP bind failed")?;

    let uid_list: Vec<&String> = uids.iter().collect();
    let mut out: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();
    for (i, chunk) in uid_list.chunks(BATCH_SIZE).enumerate() {
        let filter = format!(
            "(|{})",
            chunk
                .iter()
                .map(|u| format!("(uid={})", ldap3::ldap_escape(u.as_str())))
                .collect::<String>()
        );

        let (entries, _) = conn
            .search(SEARCH_BASE, Scope::OneLevel, &filter, ATTRS)?
            .success()?;

        for entry in entries {
            let entry = SearchEntry::construct(entry);
            let Some(uid) = entry
                .attrs
                .get("uid")
                .and_then(|v| v.first())
                .map(|s| s.to_lowercase())
            else {
                continue;
            };

            out.insert(uid, entry.attrs.into_iter().collect());
        }

        if (i + 1) % 10 == 0 {
            info!(batches = i + 1, found = out.len(), "ldap progress");
        }
    }

    conn.unbind()?;
    info!(
        found = out.len(),
        missing = uids.len() - out.len(),
        "directory lookup complete"
    );

    fs::write(export_root.join("ldap.json"), serde_json::to_string(&out)?)?;
    Ok(())
}

/// Union of every instructor `username` across every student subdir's
/// `courses_history` output, both `info.json` and section files.
fn collect_instructor_uids(export_root: &Path) -> Result<BTreeSet<String>> {
    let mut uids = BTreeSet::new();

    for student in fs::read_dir(export_root)? {
        let history = student?.path().join("courses_history");
        if !history.is_dir() {
            continue;
        }

        for course in fs::read_dir(&history)? {
            let course_dir = course?.path();
            if !course_dir.is_dir() {
                continue;
            }

            for file in fs::read_dir(&course_dir)? {
                let Ok(text) = fs::read_to_string(file?.path()) else {
                    continue;
                };
                let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
                    continue;
                };

                collect_usernames(&json, &mut uids);
            }
        }
    }

    Ok(uids)
}

/// Recursively collect non-empty `username` string values. Instructor
/// objects are the only carriers of that key in the scrape output.
fn collect_usernames(v: &serde_json::Value, out: &mut BTreeSet<String>) {
    match v {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(u)) = map.get("username")
                && !u.is_empty()
            {
                out.insert(u.to_lowercase());
            }

            for val in map.values() {
                collect_usernames(val, out);
            }
        }
        serde_json::Value::Array(arr) => {
            for val in arr {
                collect_usernames(val, out);
            }
        }
        _ => {}
    }
}
