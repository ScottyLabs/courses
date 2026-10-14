//! Syllabi pipeline that walks Canvas's Syllabus Registry course to discover
//! every (term, dept) sub-course, enumerates each sub-course's `Available
//! Syllabi` module, and saves File items as their original downloads and Page
//! items as plain-text `.url` pointers under
//! `<syllabi_dir>/<term>/<dept>/<course_section>.<ext>`.
//!
//! Page items get the URL-only treatment because the page bodies are usually
//! stub redirects pointing into enrollment-restricted course sites; saving the
//! pointer lets downstream consumers decide whether to follow it.
//! `Unavailable Syllabi` and `Individualized Experiences` modules are skipped
//! on purpose because their items have no retrievable content.

use anyhow::{Context, Result};
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use rayon::prelude::*;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{debug, info, warn};

use crate::canvas::{Canvas, ModuleItem};

#[derive(Debug, Clone)]
pub struct Task {
    pub term: String,
    pub dept: String,
    pub course_section: String,
    pub item_type: String,
    pub url: String,
}

pub fn build_tasks(canvas: &Canvas) -> Result<Vec<Task>> {
    let term_modules = canvas
        .list_master_modules()
        .context("listing syllabus-registry term modules")?;

    let dept_refs: Vec<(String, String, String)> = term_modules
        .iter()
        .filter(|m| !matches!(m.name.as_str(), "Notice to Users" | "Archive"))
        .flat_map(|m| {
            let term_code = parse_term_code(&m.name).unwrap_or_else(|| m.name.clone());
            m.items.iter().filter_map(move |it| {
                let sis = parse_sis_id(&it.external_url)?;
                let dept_code = sis.rsplit('-').next()?.to_string();
                Some((term_code.clone(), dept_code, sis))
            })
        })
        .collect();
    info!(
        count = dept_refs.len(),
        "discovered (term, dept) sub-courses"
    );

    let done = AtomicUsize::new(0);
    let total = dept_refs.len();

    let tasks: Vec<Task> = dept_refs
        .par_iter()
        .flat_map_iter(|(term, dept, sis)| {
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            if n.is_multiple_of(200) {
                info!(done = n, total, "sub-course discovery");
            }

            match canvas.list_subcourse_modules(sis) {
                Ok(mods) => extract_available(&mods)
                    .into_iter()
                    .filter_map(|item| {
                        let course_section = course_section(&item.title)?;
                        let url = match item.item_type.as_str() {
                            "File" => item.url.clone(),
                            "Page" => item.url.clone(),
                            _ => return None,
                        };
                        if url.is_empty() {
                            return None;
                        }

                        Some(Task {
                            term: term.clone(),
                            dept: dept.clone(),
                            course_section,
                            item_type: item.item_type.clone(),
                            url,
                        })
                    })
                    .collect::<Vec<_>>(),
                Err(e) => {
                    warn!(term = %term, dept = %dept, error = %e, "sub-course modules fetch failed");
                    vec![]
                }
            }
        })
        .collect();
    Ok(tasks)
}

/// List the syllabus objects already in the CDN bucket as ext-stripped
/// `<term>/<dept>/<course_section>` keys for the cumulative skip check.
pub fn load_existing_keys() -> Result<HashSet<String>> {
    let endpoint = env::var("CDN_S3_ENDPOINT").context("CDN_S3_ENDPOINT must be set")?;
    let bucket = env::var("CDN_S3_BUCKET").context("CDN_S3_BUCKET must be set")?;
    let access_key_id = env::var("CDN_ACCESS_KEY_ID").context("CDN_ACCESS_KEY_ID must be set")?;
    let secret_access_key =
        env::var("CDN_SECRET_ACCESS_KEY").context("CDN_SECRET_ACCESS_KEY must be set")?;

    let credentials = Credentials::new(access_key_id, secret_access_key, None, None, "courses-cdn");
    let config = aws_sdk_s3::config::Builder::new()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .endpoint_url(endpoint)
        .credentials_provider(credentials)
        .force_path_style(true)
        .build();
    let client = aws_sdk_s3::Client::from_conf(config);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let mut pages = client
            .list_objects_v2()
            .bucket(&bucket)
            .prefix("syllabi/")
            .into_paginator()
            .send();

        let mut keys = HashSet::new();
        while let Some(page) = pages.next().await {
            for object in page?.contents() {
                if let Some(key) = object.key().and_then(|k| k.strip_prefix("syllabi/")) {
                    let stem = key.rsplit_once('.').map_or(key, |(stem, _)| stem);
                    keys.insert(stem.to_string());
                }
            }
        }

        Ok(keys)
    })
}

pub fn save_task(
    canvas: &Canvas,
    dir: &Path,
    task: &Task,
    incremental: bool,
    existing: &HashSet<String>,
) -> Result<()> {
    // Cumulative cross-run skip: the CDN already has this one. Absence
    // never gets cached, so a syllabus uploaded later is still picked up
    if existing.contains(&format!(
        "{}/{}/{}",
        task.term, task.dept, task.course_section
    )) {
        return Ok(());
    }

    let prog_dir = dir.join(&task.term).join(&task.dept);
    if incremental && already_saved(&prog_dir, &task.course_section) {
        return Ok(());
    }

    fs::create_dir_all(&prog_dir)?;

    match task.item_type.as_str() {
        "File" => {
            let meta = canvas.get_file_meta(&task.url)?;

            let ext = std::path::Path::new(&meta.filename)
                .extension()
                .and_then(|s| s.to_str())
                .or_else(|| {
                    std::path::Path::new(&meta.display_name)
                        .extension()
                        .and_then(|s| s.to_str())
                })
                .unwrap_or("bin");

            let bytes = canvas.download_bytes(&meta.url)?;

            fs::write(
                prog_dir.join(format!("{}.{ext}", task.course_section)),
                bytes,
            )?;
        }
        "Page" => {
            let page_url = canvas_page_url(&task.url);
            fs::write(
                prog_dir.join(format!("{}.url", task.course_section)),
                page_url,
            )?;
        }
        other => {
            debug!(item_type = %other, "skipping unsupported item type");
        }
    }
    Ok(())
}

fn parse_term_code(name: &str) -> Option<String> {
    let open = name.rfind('(')?;
    let close = name.rfind(')')?;
    if close > open + 1 {
        Some(name[open + 1..close].to_string())
    } else {
        None
    }
}

fn parse_sis_id(external_url: &str) -> Option<String> {
    let key = "sis_course_id:";
    let i = external_url.find(key)?;
    let rest = &external_url[i + key.len()..];
    let end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        .unwrap_or(rest.len());
    Some(rest[..end].to_string())
}

fn extract_available(mods: &[crate::canvas::Module]) -> Vec<&ModuleItem> {
    mods.iter()
        .filter(|m| {
            let n = m.name.to_ascii_lowercase();
            n.contains("available syllabi") && !n.contains("unavailable syllabi")
        })
        .flat_map(|m| m.items.iter())
        .collect()
}

fn course_section(title: &str) -> Option<String> {
    let prefix = title.split(':').next()?.trim();
    if prefix.is_empty() {
        return None;
    }

    let safe: String = prefix
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    Some(safe)
}

fn canvas_page_url(api_url: &str) -> String {
    api_url.replacen("/api/v1/", "/", 1)
}

fn already_saved(prog_dir: &Path, course_section: &str) -> bool {
    let prefix = format!("{course_section}.");
    let Ok(entries) = fs::read_dir(prog_dir) else {
        return false;
    };

    entries.flatten().any(|e| {
        e.file_name()
            .to_str()
            .map(|n| n.starts_with(&prefix))
            .unwrap_or(false)
    })
}
