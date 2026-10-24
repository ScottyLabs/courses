//! The term-independent course catalog (Courseweave's `scripts/build-catalog.ts`): `catalog/index` (requisites,
//! grading, offering pattern) and `catalog/details/<NN>` (descriptions and course profiles).
//!
//! Sources: Stellic course info (`exported/<andrew id>/courses_history/<id>/info.json`: requisite trees with minimum
//! grades, grading, offering tags, profiles) and SOC course detail pages (`exported/soc/details.json`: descriptions,
//! prerequisite text, cross-listings, notes).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::js::{self, Object, ser_opt_num};
use crate::model::{course_code, is_course_id};
use crate::requisite::{
    Requisite, StellicCourseRef, StellicNode, from_stellic, parse_text, simplify,
};
use crate::soc::{SocDetail, SocDetailsCache};
use crate::sources::{read_json, stellic_users};
use crate::titles::same_course_title;

#[derive(Debug, Default, Deserialize)]
pub struct StellicInfo {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub long_desc: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub units: Option<f64>,
    #[serde(default)]
    pub min_units: Option<f64>,
    #[serde(default)]
    pub max_units: Option<f64>,
    #[serde(default)]
    pub prereqs: Option<Prereqs>,
    #[serde(default)]
    pub co_reqs: Option<Vec<StellicCourseRef>>,
    #[serde(default)]
    pub anti_reqs: Option<Vec<StellicCourseRef>>,
    #[serde(default)]
    pub equiv: Option<Vec<StellicCourseRef>>,
    #[serde(default)]
    pub grade_option: Option<GradeOption>,
    #[serde(default)]
    pub offering_tags: Option<Vec<OfferingTag>>,
    #[serde(default)]
    pub attributes: Option<Vec<Attribute>>,
    #[serde(default)]
    pub custom_fields: Option<HashMap<String, Option<String>>>,
    #[serde(default)]
    pub is_repeatable: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Prereqs {
    #[serde(default)]
    pub req_obj: Option<StellicNode>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub raw_pre_req: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct GradeOption {
    #[serde(default)]
    pub is_pass_fail: Option<bool>,
    #[serde(default)]
    pub never_graded: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct OfferingTag {
    pub name: String,
    #[serde(default)]
    pub sem: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Attribute {
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    pub id: String,
    pub code: String,
    pub title: String,
    #[serde(serialize_with = "ser_opt_num")]
    pub units: Option<f64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "ser_opt_num"
    )]
    pub units_max: Option<f64>,
    pub prereqs: Option<Requisite>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prereq_text: Option<String>,
    pub coreqs: Vec<String>,
    pub antireqs: Vec<String>,
    pub equivalents: Vec<String>,
    pub pass_fail: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offered: Option<Vec<&'static str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intermittent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeatable: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goals: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_topics: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assessment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relevance: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<Profile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_required: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogIndex<'a> {
    pub generated_at: &'a str,
    pub courses: Object<&'a CatalogEntry>,
}

#[derive(Default)]
pub struct Stats {
    pub tree: usize,
    pub soc_text: usize,
    pub text_only: usize,
    pub none: usize,
    pub soc_descriptions: usize,
    pub stellic_descriptions: usize,
    pub related_dropped: usize,
}

pub struct Catalog {
    /// When the SOC details were fetched.
    pub fetched_at: Option<String>,
    /// Every course, by id.
    pub entries: BTreeMap<String, CatalogEntry>,
    /// Details by course-number prefix, then id.
    pub details: BTreeMap<String, BTreeMap<String, CatalogDetails>>,
    pub stats: Stats,
}

impl Catalog {
    pub fn pass_fail(&self) -> BTreeSet<String> {
        self.entries
            .values()
            .filter(|e| e.pass_fail)
            .map(|e| e.id.clone())
            .collect()
    }

    pub fn index<'a>(&'a self, generated_at: &'a str) -> CatalogIndex<'a> {
        CatalogIndex {
            generated_at,
            courses: self.entries.iter().map(|(k, v)| (k.clone(), v)).collect(),
        }
    }

    pub fn details_chunks(&self) -> impl Iterator<Item = (&String, Object<&CatalogDetails>)> {
        self.details
            .iter()
            .map(|(prefix, chunk)| (prefix, chunk.iter().map(|(k, v)| (k.clone(), v)).collect()))
    }
}

fn clean_text(s: Option<&str>) -> Option<String> {
    let s = s?.replace('\r', "");
    let mut out = String::with_capacity(s.len());
    let mut gap = false;
    for c in s.chars() {
        if c == ' ' || c == '\t' {
            if !gap {
                out.push(' ');
            }
            gap = true;
        } else {
            out.push(c);
            gap = false;
        }
    }
    Some(js::trim(&out).to_owned()).filter(|t| !t.is_empty())
}

fn to_id(code: &str) -> String {
    code.chars().filter(char::is_ascii_digit).collect()
}

fn id_list<'a>(refs: impl IntoIterator<Item = &'a StellicCourseRef>, own: &str) -> Vec<String> {
    let set: BTreeSet<String> = refs
        .into_iter()
        .map(|r| to_id(&r.code))
        .filter(|id| is_course_id(id) && id != own)
        .collect();
    set.into_iter().collect()
}

type Offered = (Option<Vec<&'static str>>, Option<bool>);

fn offered_from(info: &StellicInfo) -> Offered {
    let tags = info.offering_tags.as_deref().unwrap_or_default();
    let mut seasons: Vec<Option<&'static str>> = Vec::new();
    for t in tags {
        if t.name == "specific_sem"
            && let Some(sem) = t.sem.filter(|s| *s != 0.0 && !s.is_nan())
        {
            let season = match sem as i64 {
                1 if sem == 1.0 => Some("fall"),
                2 if sem == 2.0 => Some("spring"),
                3 if sem == 3.0 => Some("summer"),
                _ => None,
            };
            if !seasons.contains(&season) {
                seasons.push(season);
            }
        }
    }
    let intermittent = tags.iter().any(|t| t.name == "intermittent");
    let offered = (!seasons.is_empty()).then(|| {
        ["fall", "spring", "summer"]
            .into_iter()
            .filter(|s| seasons.contains(&Some(*s)))
            .collect()
    });
    (offered, intermittent.then_some(true))
}

/// Every account's `info.json` per course. With several accounts, the last in name order wins.
fn find_infos(exported: &Path) -> Result<BTreeMap<String, std::path::PathBuf>> {
    let mut out = BTreeMap::new();
    for user in stellic_users(exported) {
        let hist = exported.join(&user).join("courses_history");
        for id in crate::sources::stellic::course_dirs(&hist)? {
            let path = hist.join(&id).join("info.json");
            if path.exists() {
                out.insert(id, path);
            }
        }
    }
    Ok(out)
}

pub fn read_infos(exported: &Path) -> Result<BTreeMap<String, StellicInfo>> {
    let paths = find_infos(exported)?;
    paths
        .into_par_iter()
        .map(|(id, path)| -> Result<(String, StellicInfo)> {
            let bytes =
                std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
            let info = serde_json::from_slice(&bytes)
                .with_context(|| format!("parsing {}", path.display()))?;
            Ok((id, info))
        })
        .collect()
}

pub fn read_soc_details(exported: &Path) -> Result<SocDetailsCache> {
    let path = exported.join("soc").join("details.json");
    if path.exists() {
        read_json(&path)
    } else {
        Ok(SocDetailsCache::default())
    }
}

pub fn build(exported: &Path) -> Result<Catalog> {
    let (infos, soc) = rayon::join(|| read_infos(exported), || read_soc_details(exported));
    let (infos, soc) = (infos?, soc?);
    info!(
        stellic = infos.len(),
        soc = soc.courses.len(),
        "catalog sources"
    );
    let fetched_at = Some(soc.fetched_at.clone()).filter(|f| !f.is_empty());
    let soc = soc.courses;
    Ok(build_from(&infos, &soc, fetched_at))
}

pub fn build_from(
    infos: &BTreeMap<String, StellicInfo>,
    soc: &BTreeMap<String, SocDetail>,
    fetched_at: Option<String>,
) -> Catalog {
    let mut stats = Stats::default();
    let mut entries = BTreeMap::new();
    let mut details: BTreeMap<String, BTreeMap<String, CatalogDetails>> = BTreeMap::new();
    let ids: BTreeSet<&String> = infos.keys().chain(soc.keys()).collect();

    for id in ids {
        let info = infos.get(id);
        let s = soc.get(id);

        let mut prereqs: Option<Requisite> = None;
        if let Some(tree) = info.and_then(|i| i.prereqs.as_ref()?.req_obj.as_ref()) {
            prereqs = from_stellic(tree, None);
            if prereqs.is_some() {
                stats.tree += 1;
            }
        }
        let soc_text = s
            .and_then(|s| s.prereq_text.as_deref())
            .filter(|t| !t.is_empty());
        if prereqs.is_none()
            && let Some(text) = soc_text
        {
            prereqs = parse_text(text);
            if prereqs.is_some() {
                stats.soc_text += 1;
            } else {
                prereqs = Some(Requisite::Text(text.to_owned()));
                stats.text_only += 1;
            }
        }
        if prereqs.is_none() {
            let p = info.and_then(|i| i.prereqs.as_ref());
            let text = clean_text(p.and_then(|p| p.text.as_deref()))
                .or_else(|| clean_text(p.and_then(|p| p.raw_pre_req.as_deref())));
            if let Some(text) = text {
                prereqs = Some(parse_text(&text).unwrap_or(Requisite::Text(text)));
                stats.text_only += 1;
            } else {
                stats.none += 1;
            }
        }
        let prereqs = prereqs.map(simplify);
        // Keep the SOC wording when it doesn't round-trip to a course expression (standing requirements).
        let prereq_text = soc_text
            .filter(|t| !matches!(prereqs, Some(Requisite::Text(_))) && parse_text(t).is_none())
            .map(str::to_owned);

        let min_units = info.and_then(|i| i.min_units.or(i.units));
        let max_units = info.and_then(|i| i.max_units);
        let units = min_units.or_else(|| s.and_then(|s| s.units).flatten());
        let pass_fail = info
            .and_then(|i| i.grade_option.as_ref())
            .is_some_and(|g| g.is_pass_fail == Some(true) || g.never_graded == Some(true));

        let mut coreqs: BTreeSet<String> = id_list(
            info.and_then(|i| i.co_reqs.as_deref()).unwrap_or_default(),
            id,
        )
        .into_iter()
        .collect();
        coreqs.extend(
            s.and_then(|s| s.coreqs.clone())
                .unwrap_or_default()
                .into_iter()
                .filter(|c| c != id),
        );

        let title = clean_text(info.and_then(|i| i.name.as_deref()))
            .or_else(|| s.and_then(|s| s.title.clone()));
        let title_of = |other: &str| {
            clean_text(infos.get(other).and_then(|i| i.name.as_deref()))
                .or_else(|| soc.get(other).and_then(|s| s.title.clone()))
        };
        let mut same = |other: &str, name: Option<&str>| {
            let t = clean_text(name).or_else(|| title_of(other));
            let ok = match (title.as_deref(), t.as_deref()) {
                (Some(a), Some(b)) if !a.is_empty() && !b.is_empty() => same_course_title(a, b),
                _ => true,
            };
            if !ok {
                stats.related_dropped += 1;
            }
            ok
        };
        let equiv: Vec<&StellicCourseRef> = info
            .and_then(|i| i.equiv.as_deref())
            .unwrap_or_default()
            .iter()
            .filter(|r| same(&to_id(&r.code), r.name.as_deref()))
            .collect();
        let mut equivalents: BTreeSet<String> = id_list(equiv, id).into_iter().collect();
        for c in s
            .and_then(|s| s.cross_listed.as_deref())
            .unwrap_or_default()
        {
            if c != id && is_course_id(c) && same(c, None) {
                equivalents.insert(c.clone());
            }
        }

        let (offered, intermittent) = info.map(offered_from).unwrap_or_default();
        let entry = CatalogEntry {
            id: id.clone(),
            code: course_code(id),
            title: clean_text(info.and_then(|i| i.name.as_deref()))
                .or_else(|| s.and_then(|s| s.title.clone()))
                .unwrap_or_else(|| course_code(id)),
            units: units.filter(|u| *u >= 0.0),
            units_max: match (max_units, min_units) {
                (Some(max), Some(min)) if max > min => Some(max),
                _ => None,
            },
            prereqs,
            prereq_text,
            coreqs: coreqs.into_iter().collect(),
            antireqs: id_list(
                info.and_then(|i| i.anti_reqs.as_deref())
                    .unwrap_or_default(),
                id,
            ),
            equivalents: equivalents.into_iter().collect(),
            pass_fail,
            offered,
            intermittent,
            repeatable: info.and_then(|i| i.is_repeatable).filter(|r| *r),
        };
        entries.insert(id.clone(), entry);

        let description = clean_text(s.and_then(|s| s.description.as_deref()))
            .or_else(|| clean_text(info.and_then(|i| i.long_desc.as_deref())));
        if s.and_then(|s| s.description.as_deref())
            .is_some_and(|d| !d.is_empty())
        {
            stats.soc_descriptions += 1;
        } else if info
            .and_then(|i| i.long_desc.as_deref())
            .is_some_and(|d| !d.is_empty())
        {
            stats.stellic_descriptions += 1;
        }
        let field = |k: &str| {
            clean_text(
                info.and_then(|i| i.custom_fields.as_ref())
                    .and_then(|cf| cf.get(k)?.as_deref()),
            )
        };
        let profile = Profile {
            goals: field("goals"),
            key_topics: field("key_topics"),
            assessment: field("assessment_structure"),
            resources: field("learning_resources"),
            background: field("prerequisite_knowledge"),
            relevance: field("relevance"),
        };
        let has_profile = [
            &profile.goals,
            &profile.key_topics,
            &profile.assessment,
            &profile.resources,
            &profile.background,
            &profile.relevance,
        ]
        .iter()
        .any(|f| f.is_some());
        let mut attributes: Vec<String> = Vec::new();
        for a in info
            .and_then(|i| i.attributes.as_deref())
            .unwrap_or_default()
        {
            if let Some(name) = clean_text(a.name.as_deref())
                && !attributes.contains(&name)
            {
                attributes.push(name);
            }
        }
        let website = clean_text(info.and_then(|i| i.website.as_deref())).or_else(|| {
            s.and_then(|s| s.urls.as_ref())?
                .iter()
                .find(|u| !is_department_home(u))
                .cloned()
        });
        let d = CatalogDetails {
            description,
            website,
            attributes: (!attributes.is_empty()).then_some(attributes),
            profile: has_profile.then_some(profile),
            notes: s.and_then(|s| s.notes.clone()).filter(|n| !n.is_empty()),
            permission_required: s.and_then(|s| s.permission).filter(|p| *p),
        };
        if d.description.is_some()
            || d.website.is_some()
            || d.attributes.is_some()
            || d.profile.is_some()
            || d.notes.is_some()
            || d.permission_required.is_some()
        {
            details
                .entry(id[..2].to_owned())
                .or_default()
                .insert(id.clone(), d);
        }
    }

    Catalog {
        fetched_at,
        entries,
        details,
        stats,
    }
}

/// `https://www.<dept>.cmu.edu/`: a department's home page rather than the course's.
fn is_department_home(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("https://www.")
        .or_else(|| lower.strip_prefix("http://www."));
    let Some(rest) = rest else { return false };
    let rest = rest.strip_suffix('/').unwrap_or(rest);
    rest.strip_suffix(".cmu.edu")
        .is_some_and(|dept| !dept.is_empty() && dept.bytes().all(|b| b.is_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(json: &str) -> StellicInfo {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn builds_entries_from_stellic_and_soc() {
        let mut infos = BTreeMap::new();
        infos.insert(
            "15122".to_owned(),
            info(
                r#"{"name":"Principles of Imperative Computation","units":12.0,"min_units":12.0,"max_units":12.0,
                "prereqs":{"req_obj":{"constraints":[{"type":"mingrade","data":{"grade":"C"}},{"type":"course","data":{"course":{"code":"15-112"}}}]}},
                "co_reqs":[{"code":"21-127"},{"code":"21-128"},{"code":"15-122"}],
                "equiv":[{"code":"15-622","name":"Imperative Computation"},{"code":"15-123","name":"Principles of Imperative Computation"}],
                "offering_tags":[{"name":"specific_sem","sem":2},{"name":"specific_sem","sem":1},{"name":"intermittent"}],
                "custom_fields":{"goals":"  Write\r\n  code ","relevance":null},"attributes":[{"name":"x"},{"name":"x"}],
                "is_repeatable":false,"long_desc":"stellic"}"#,
            ),
        );
        infos.insert(
            "02100".to_owned(),
            info(r#"{"name":"Pass Fail","units":3.0,"min_units":3.0,"max_units":9.0,"grade_option":{"is_pass_fail":true},"prereqs":{"text":"Junior standing"}}"#),
        );
        let mut soc = BTreeMap::new();
        soc.insert(
            "15122".to_owned(),
            SocDetail {
                semester: "F26".into(),
                description: Some("Pointers.".into()),
                prereq_text: Some("15112".into()),
                cross_listed: Some(vec!["15122".into(), "98012".into()]),
                urls: Some(vec![
                    "https://www.cs.cmu.edu/".into(),
                    "https://www.cs.cmu.edu/~15122".into(),
                ]),
                permission: Some(false),
                ..SocDetail::default()
            },
        );
        soc.insert(
            "98012".to_owned(),
            SocDetail {
                semester: "F26".into(),
                title: Some("Fun with Robots".into()),
                prereq_text: Some("Sophomore standing".into()),
                units: Some(None),
                ..SocDetail::default()
            },
        );
        let catalog = build_from(&infos, &soc, Some("2026-10-21T23:31:21.393Z".into()));
        let json = serde_json::to_string(&catalog.index("2026-10-21T23:31:21.393Z")).unwrap();
        assert_eq!(
            json,
            concat!(
                r#"{"generatedAt":"2026-10-21T23:31:21.393Z","courses":{"#,
                r#""15122":{"id":"15122","code":"15-122","title":"Principles of Imperative Computation","units":12,"#,
                r#""prereqs":{"type":"course","id":"15112","minGrade":"C"},"coreqs":["21127","21128"],"antireqs":[],"#,
                r#""equivalents":["15123"],"passFail":false,"offered":["fall","spring"],"intermittent":true},"#,
                r#""98012":{"id":"98012","code":"98-012","title":"Fun with Robots","units":null,"#,
                r#""prereqs":{"type":"text","text":"Sophomore standing"},"coreqs":[],"antireqs":[],"equivalents":[],"passFail":false},"#,
                r#""02100":{"id":"02100","code":"02-100","title":"Pass Fail","units":3,"unitsMax":9,"#,
                r#""prereqs":{"type":"text","text":"Junior standing"},"coreqs":[],"antireqs":[],"equivalents":[],"passFail":true}}}"#
            )
        );
        assert_eq!(
            catalog.pass_fail().into_iter().collect::<Vec<_>>(),
            ["02100"]
        );
        let d = serde_json::to_string(&catalog.details["15"]).unwrap();
        assert_eq!(
            d,
            r#"{"15122":{"description":"Pointers.","website":"https://www.cs.cmu.edu/~15122","attributes":["x"],"profile":{"goals":"Write\n code"}}}"#
        );
        assert_eq!(catalog.stats.related_dropped, 2);
    }

    #[test]
    fn department_home_pages() {
        assert!(is_department_home("https://www.cs.cmu.edu/"));
        assert!(is_department_home("http://www.math.cmu.edu"));
        assert!(!is_department_home("https://www.cs.cmu.edu/~15122"));
        assert!(!is_department_home("https://cs.cmu.edu/"));
    }
}
