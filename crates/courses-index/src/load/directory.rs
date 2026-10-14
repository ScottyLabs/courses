//! Merge `<exported_root>/ldap.json` (CMU directory attributes keyed by
//! lowercased AndrewID, written by the scraper's `ldap` mode) onto
//! [`Professor`]s by username. A missing file is a no-op so catalogs
//! still build from scrapes that predate directory enrichment.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use tracing::debug;

use crate::doc::Professor;

type DirectoryMap = HashMap<String, HashMap<String, Vec<String>>>;

/// Output of the directory phase.
#[derive(Debug, Default)]
pub struct DirectoryLoad {
    pub entries: usize,
    pub professors_matched: usize,
}

/// Apply directory attributes onto professors by lowercased username.
pub fn load_directory(exported_root: &Path, professors: &mut [Professor]) -> Result<DirectoryLoad> {
    let path = exported_root.join("ldap.json");
    if !path.exists() {
        return Ok(DirectoryLoad::default());
    }

    let map: DirectoryMap = serde_json::from_str(&fs::read_to_string(&path)?)?;

    let mut matched = 0usize;
    for prof in professors.iter_mut() {
        let Some(username) = prof.username.as_deref() else {
            continue;
        };
        let Some(attrs) = map.get(&username.to_lowercase()) else {
            continue;
        };

        let first = |key: &str| attrs.get(key).and_then(|v| v.first()).cloned();
        let all = |key: &str| attrs.get(key).cloned().unwrap_or_default();

        prof.email = first("mail");
        prof.title = first("title");
        prof.directory_departments = all("cmuDepartment");
        prof.affiliation = first("eduPersonPrimaryAffiliation");
        prof.colleges = all("eduPersonSchoolCollegeName");
        prof.office = first("postalAddress");
        prof.phone = first("telephoneNumber");
        prof.orcid = first("eduPersonOrcid");
        matched += 1;
    }

    debug!(entries = map.len(), matched, "directory merge complete");
    Ok(DirectoryLoad {
        entries: map.len(),
        professors_matched: matched,
    })
}
