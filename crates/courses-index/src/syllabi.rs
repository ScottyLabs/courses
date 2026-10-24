//! The past-syllabus index (Courseweave's `scripts/build-syllabi.ts`, `scripts/lib/syllabi.ts` and
//! `src/lib/syllabus-files.ts`): `syllabi/<NN>` maps course ids to their syllabi, newest term first. Reads the
//! syllabus registry export, `exported/syllabi/<term>/<dept>/<course>-<section>.<ext>`.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::js::{self, Object};

/// File types the server serves, by extension.
pub const TYPES: [&str; 8] = ["pdf", "docx", "doc", "pptx", "odt", "rtf", "txt", "html"];

#[derive(Clone, Debug, PartialEq)]
pub struct SyllabusFile {
    pub course_id: String,
    pub term: String,
    /// Sections sharing the file (identical copies are merged); empty when it isn't per section.
    pub sections: Vec<String>,
    /// `<term>/<dept>/<file name>`: the storage key.
    pub key: String,
    pub path: PathBuf,
    pub size: u64,
    /// File type, or `None` for a Canvas page link (`.url`).
    pub kind: Option<String>,
    pub url: Option<String>,
}

/// One entry of a `syllabi/<NN>` document.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SyllabusEntry {
    pub term: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// `21241-1.pdf` and `21-241-1.pdf` are course 21241, section 1. Names without a short section (`48734_Notes.pdf`)
/// have none.
pub fn parse_name(name: &str) -> Option<(String, Vec<String>)> {
    // `name.replace(/\.[^.]+$/, '')`
    let base = match name.rfind('.') {
        Some(i) if i + 1 < name.len() => &name[..i],
        _ => name,
    };
    let b = base.as_bytes();
    let digit = |i: usize| b.get(i).is_some_and(u8::is_ascii_digit);
    if !(digit(0) && digit(1)) {
        return None;
    }
    let mut i = 2;
    if b.get(2) == Some(&b'-') && digit(3) && digit(4) && digit(5) {
        i = 3;
    }
    if !(digit(i) && digit(i + 1) && digit(i + 2)) {
        return None;
    }
    let course_id = format!("{}{}", &base[..2], &base[i..i + 3]);
    let rest = &base[i + 3..];
    if rest.is_empty() {
        return Some((course_id, Vec::new()));
    }
    let section = rest.strip_prefix(['-', '_', ' '])?;
    // `(.+)` doesn't match line terminators.
    if section.is_empty() || section.contains(['\n', '\r', '\u{2028}', '\u{2029}']) {
        return None;
    }
    let short = section.len() <= 4 && section.bytes().all(|b| b.is_ascii_alphanumeric());
    Some((
        course_id,
        if short {
            vec![section.to_uppercase()]
        } else {
            Vec::new()
        },
    ))
}

/// The file's type, when it's one we serve.
pub fn file_type(key: &str) -> Option<String> {
    let i = key.rfind('.')?;
    let ext = &key[i + 1..];
    if ext.is_empty() || !ext.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return None;
    }
    let ext = ext.to_ascii_lowercase();
    TYPES.contains(&ext.as_str()).then_some(ext)
}

fn is_term(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 3
        && matches!(b[0], b'S' | b'M' | b'N' | b'F')
        && b[1].is_ascii_digit()
        && b[2].is_ascii_digit()
}

/// `F25/MSC/21241-1.pdf`: a registry term, a department code and a file name with a served type.
pub fn is_key(key: &str) -> bool {
    let mut parts = key.splitn(3, '/');
    let (Some(term), Some(dept), Some(file)) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    is_term(term)
        && (1..=6).contains(&dept.len())
        && dept
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        && (1..=200).contains(&file.encode_utf16().count())
        && !file.contains(['/', '\\'])
        && !file.starts_with('.')
        && file_type(key).is_some()
}

/// Sort key of a registry term (S, M for summer one, N for summer two, F): later terms are larger.
pub fn term_order(term: &str) -> f64 {
    let season = match term.chars().next() {
        Some('S') => 0.0,
        Some('M') => 1.0,
        Some('N') => 2.0,
        Some('F') => 3.0,
        _ => 0.0,
    };
    js::number(term.get(1..).unwrap_or("")) * 4.0 + season
}

fn sorted_entries(dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mut out: Vec<(String, PathBuf)> = std::fs::read_dir(dir)
        .with_context(|| format!("listing {}", dir.display()))?
        .filter_map(|e| {
            let e = e.ok()?;
            Some((e.file_name().into_string().ok()?, e.path()))
        })
        .collect();
    out.sort();
    Ok(out)
}

pub struct Scan {
    pub files: Vec<SyllabusFile>,
    pub skipped: Vec<String>,
}

/// Every syllabus file and Canvas page link in the export, plus what was skipped and why.
pub fn scan(dir: &Path) -> Result<Scan> {
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    if !dir.exists() {
        return Ok(Scan { files, skipped });
    }
    for (term, term_dir) in sorted_entries(dir)? {
        if !is_term(&term) {
            continue;
        }
        for (dept, dept_dir) in sorted_entries(&term_dir)? {
            if !dept_dir.is_dir() {
                continue;
            }
            for (name, path) in sorted_entries(&dept_dir)? {
                if name.starts_with('.') {
                    continue;
                }
                let key = format!("{term}/{dept}/{name}");
                let Some((course_id, sections)) = parse_name(&name) else {
                    skipped.push(format!("{key}: no course number"));
                    continue;
                };
                if name.ends_with(".url") {
                    let url = std::fs::read_to_string(&path).unwrap_or_default();
                    let url = js::trim(&url);
                    if is_canvas_page(url) {
                        files.push(SyllabusFile {
                            course_id,
                            term: term.clone(),
                            sections,
                            key,
                            path,
                            size: 0,
                            kind: None,
                            url: Some(url.to_owned()),
                        });
                    } else {
                        skipped.push(format!("{key}: not a Canvas page link"));
                    }
                    continue;
                }
                let kind = file_type(&name);
                if kind.is_none() || !is_key(&key) {
                    skipped.push(format!("{key}: unsupported file"));
                    continue;
                }
                let size = std::fs::metadata(&path)
                    .with_context(|| format!("reading {}", path.display()))?
                    .len();
                files.push(SyllabusFile {
                    course_id,
                    term: term.clone(),
                    sections,
                    key,
                    path,
                    size,
                    kind,
                    url: None,
                });
            }
        }
    }
    Ok(Scan {
        files: merge_duplicates(files, |p| {
            std::fs::read(p).with_context(|| format!("reading {}", p.display()))
        })?,
        skipped,
    })
}

/// `https://canvas.cmu.edu/courses/<n>/pages/<slug>`
fn is_canvas_page(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://canvas.cmu.edu/courses/") else {
        return false;
    };
    let Some((n, slug)) = rest.split_once("/pages/") else {
        return false;
    };
    !n.is_empty()
        && n.bytes().all(|b| b.is_ascii_digit())
        && !slug.is_empty()
        && slug
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// One file per distinct syllabus: departments often upload the same file for every section. Copies (same course,
/// term, type and bytes) merge into the first, which collects their sections. Only same-size files are hashed.
/// Links come first, then files in the order their group first appears, as Courseweave orders them.
pub fn merge_duplicates(
    files: Vec<SyllabusFile>,
    content: impl Fn(&Path) -> Result<Vec<u8>> + Sync,
) -> Result<Vec<SyllabusFile>> {
    let mut out = Vec::new();
    let mut groups: Vec<Vec<SyllabusFile>> = Vec::new();
    let mut group_of: HashMap<(String, String, String, u64), usize> = HashMap::new();
    for f in files {
        let Some(kind) = f.kind.clone() else {
            out.push(f);
            continue;
        };
        let key = (f.course_id.clone(), f.term.clone(), kind, f.size);
        match group_of.get(&key) {
            Some(&i) => groups[i].push(f),
            None => {
                group_of.insert(key, groups.len());
                groups.push(vec![f]);
            }
        }
    }
    let hashed: Vec<Vec<String>> = groups
        .par_iter()
        .map(|list| {
            if list.len() == 1 {
                return Ok(vec![String::new()]);
            }
            list.par_iter()
                .map(|f| Ok(format!("{:x}", Sha256::digest(content(&f.path)?))))
                .collect()
        })
        .collect::<Result<_>>()?;
    for (list, hashes) in groups.into_iter().zip(hashed) {
        let mut firsts: Vec<(String, usize)> = Vec::new();
        for (f, hash) in list.into_iter().zip(hashes) {
            if let Some(&(_, i)) = firsts.iter().find(|(h, _)| *h == hash) {
                let first: &mut SyllabusFile = &mut out[i];
                for s in f.sections {
                    if !first.sections.contains(&s) {
                        first.sections.push(s);
                    }
                }
            } else {
                firsts.push((hash, out.len()));
                out.push(f);
            }
        }
    }
    Ok(out)
}

/// Index entries per course-number prefix: newest term first, then by section, files before links.
pub fn index(files: &[SyllabusFile]) -> BTreeMap<String, Object<Vec<SyllabusEntry>>> {
    let mut chunks: BTreeMap<String, Object<Vec<SyllabusEntry>>> = BTreeMap::new();
    for f in files {
        let mut sections = f.sections.clone();
        sections.sort_by(|a, b| js::numeric_cmp(a, b));
        let entry = SyllabusEntry {
            term: f.term.clone(),
            sections: (!sections.is_empty()).then_some(sections),
            key: f.kind.as_ref().map(|_| f.key.clone()),
            kind: f.kind.clone(),
            size: f.kind.as_ref().map(|_| f.size),
            url: if f.kind.is_some() {
                None
            } else {
                f.url.clone()
            },
        };
        let chunk = chunks.entry(f.course_id[..2].to_owned()).or_default();
        match chunk.0.iter_mut().find(|(id, _)| *id == f.course_id) {
            Some((_, list)) => list.push(entry),
            None => chunk.push(f.course_id.clone(), vec![entry]),
        }
    }
    for chunk in chunks.values_mut() {
        for (_, list) in &mut chunk.0 {
            list.sort_by(|a, b| {
                let first = |e: &SyllabusEntry| {
                    e.sections
                        .as_ref()
                        .and_then(|s| s.first().cloned())
                        .unwrap_or_default()
                };
                term_order(&b.term)
                    .partial_cmp(&term_order(&a.term))
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| js::numeric_cmp(&first(a), &first(b)))
                    .then_with(|| a.url.is_some().cmp(&b.url.is_some()))
            });
        }
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(dir: &Path, path: &str, content: &str) {
        let full = dir.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, content).unwrap();
    }

    fn export() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        put(d, "F25/MSC/21241-1.pdf", "same syllabus");
        put(d, "F25/MSC/21241-2.pdf", "same syllabus");
        put(d, "F25/MSC/21241-3.pdf", "other syllabus");
        put(
            d,
            "F25/MSC/21241-W.url",
            "https://canvas.cmu.edu/courses/123/pages/21241-w",
        );
        put(d, "N24/MSC/21241-A.docx", "docx");
        put(d, "S25/H00/65411-A.pdf", "pdf");
        put(d, "S25/SDS/88-150-A.pdf", "dashed number");
        put(d, "S25/III/notes.pdf", "no course number");
        put(d, "S25/ISM/95794-A1.ics", "calendar");
        put(d, "S25/MSC/21120-1.url", "https://example.com/elsewhere");
        dir
    }

    #[test]
    fn reads_course_numbers_and_sections() {
        let p = |n: &str| parse_name(n).map(|(c, s)| (c, s.join(",")));
        assert_eq!(p("21241-1.pdf"), Some(("21241".into(), "1".into())));
        assert_eq!(p("88-150-A.pdf"), Some(("88150".into(), "A".into())));
        assert_eq!(p("21-241-1.pdf"), Some(("21241".into(), "1".into())));
        assert_eq!(p("73103.pdf"), Some(("73103".into(), "".into())));
        assert_eq!(
            p("48734_Possibilistic_Design.pdf"),
            Some(("48734".into(), "".into()))
        );
        assert_eq!(p("15122 a.pdf"), Some(("15122".into(), "A".into())));
        assert_eq!(p("notes.pdf"), None);
        assert_eq!(p("151223.pdf"), None);
    }

    #[test]
    fn merges_copies_keeps_links_and_skips_the_rest() {
        let dir = export();
        let Scan { files, mut skipped } = scan(dir.path()).unwrap();
        let f25: Vec<_> = files
            .iter()
            .filter(|f| f.course_id == "21241" && f.term == "F25")
            .map(|f| (f.key.as_str(), f.sections.join(","), f.kind.as_deref()))
            .collect();
        assert_eq!(
            f25,
            [
                ("F25/MSC/21241-W.url", "W".to_owned(), None),
                ("F25/MSC/21241-1.pdf", "1,2".to_owned(), Some("pdf")),
                ("F25/MSC/21241-3.pdf", "3".to_owned(), Some("pdf")),
            ]
        );
        assert!(files.iter().any(|f| f.key == "S25/H00/65411-A.pdf"));
        skipped.sort();
        assert_eq!(
            skipped,
            [
                "S25/III/notes.pdf: no course number",
                "S25/ISM/95794-A1.ics: unsupported file",
                "S25/MSC/21120-1.url: not a Canvas page link",
            ]
        );
    }

    #[test]
    fn indexes_by_prefix_newest_first_files_before_links() {
        let dir = export();
        let chunks = index(&scan(dir.path()).unwrap().files);
        assert_eq!(chunks.keys().collect::<Vec<_>>(), ["21", "65", "88"]);
        assert_eq!(
            serde_json::to_string(&chunks["21"]).unwrap(),
            concat!(
                r#"{"21241":[{"term":"F25","sections":["1","2"],"key":"F25/MSC/21241-1.pdf","type":"pdf","size":13},"#,
                r#"{"term":"F25","sections":["3"],"key":"F25/MSC/21241-3.pdf","type":"pdf","size":14},"#,
                r#"{"term":"F25","sections":["W"],"url":"https://canvas.cmu.edu/courses/123/pages/21241-w"},"#,
                r#"{"term":"N24","sections":["A"],"key":"N24/MSC/21241-A.docx","type":"docx","size":4}]}"#
            )
        );
    }

    #[test]
    fn keys_and_terms() {
        assert!(is_key("F25/MSC/21241-1.pdf"));
        assert!(is_key("S25/H00/65411-A.PDF"));
        assert!(!is_key("F25/MSC/../secret.pdf"));
        assert!(!is_key("../F25/MSC/21241-1.pdf"));
        assert!(!is_key("F25/MSC/.hidden.pdf"));
        assert!(!is_key("F25/MSC/21241-1.exe"));
        assert!(!is_key("X25/MSC/21241-1.pdf"));
        let mut terms = vec!["F24", "N24", "S24", "M24"];
        terms.sort_by(|a, b| term_order(a).partial_cmp(&term_order(b)).unwrap());
        assert_eq!(terms, ["S24", "M24", "N24", "F24"]);
    }
}
