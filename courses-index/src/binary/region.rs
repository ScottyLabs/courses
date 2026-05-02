//! Region-based catalog encoding. The catalog file is split into named
//! regions of raw bincode, with the whole file optionally gzip-wrapped at
//! the outer layer for transit and storage. The browser uses the native
//! `DecompressionStream` API to unwrap the gzip before handing raw bytes
//! to wasm, while native callers go through `flate2` for the same job.
//!
//! Region bodies themselves are uncompressed inside the gzip wrap, which
//! keeps wasm-side reading allocation-free. The browser already takes
//! advantage of that with [`read_payload_minimal`], which skips the
//! `professors` and `fce_rows` regions during cold start because the
//! search UI doesn't read them. Future OPFS-streaming work can borrow
//! ranges directly out of the decompressed buffer in the same shape.
//!
//! File layout (after gzip is unwrapped):
//!
//! ```text
//! [16-byte header]
//!   "CIDX"             4 bytes
//!   version u32 LE     4 bytes (= 4)
//!   flags u32 LE       4 bytes (currently unused at the global level)
//!   region_count u32   4 bytes
//!
//! [region table, region_count * 40 bytes]
//!   name              16 bytes ASCII, NUL-padded
//!   body_offset u64    8 bytes (absolute file offset within unwrapped data)
//!   body_len u64       8 bytes
//!   flags u32          4 bytes (reserved, currently 0)
//!   reserved u32       4 bytes
//!
//! [region bodies, concatenated, in table order]
//! ```
//!
//! Region names used by this crate:
//!
//! - `courses`        bincode of `Vec<Course>`
//! - `professors`     bincode of `Vec<Professor>`
//! - `sections`       bincode of `Vec<SectionTime>`
//! - `fce_rows`       bincode of `Vec<FceRow>`
//! - `prebuilt_text`  bincode of `PrebuiltText` (optional)

use anyhow::{Context, Result, bail};
use bincode::config::{self, Configuration};

use super::storage::CatalogStorage;
use crate::doc::{Course, FceRow, Professor, SectionTime};
use crate::index::text::PrebuiltText;
use crate::load::Corpus;

const MAGIC: &[u8; 4] = b"CIDX";
pub const FORMAT_VERSION: u32 = 4;
const REGION_HEADER_LEN: u64 = 16;
const REGION_ENTRY_LEN: u64 = 40;

const REGION_COURSES: &str = "courses";
const REGION_PROFESSORS: &str = "professors";
const REGION_SECTIONS: &str = "sections";
const REGION_FCE_ROWS: &str = "fce_rows";
const REGION_PREBUILT_TEXT: &str = "prebuilt_text";

fn config() -> Configuration {
    config::standard()
}

#[derive(Debug, Clone)]
pub struct RegionEntry {
    pub name: String,
    pub body_offset: u64,
    pub body_len: u64,
    pub flags: u32,
}

#[derive(Debug)]
pub struct CatalogPayload {
    pub corpus: Corpus,
    pub prebuilt_text: Option<PrebuiltText>,
}

/// Read the region table without decoding any region bodies.
pub fn read_table(storage: &dyn CatalogStorage) -> Result<Vec<RegionEntry>> {
    let total = storage.len()?;
    if total < REGION_HEADER_LEN {
        bail!("catalog truncated (under 16 bytes)");
    }
    let header = storage.read_range(0, REGION_HEADER_LEN)?;
    if &header[..4] != MAGIC {
        bail!("not a catalog file (magic mismatch)");
    }
    let version = u32::from_le_bytes(header[4..8].try_into().unwrap());
    if version != FORMAT_VERSION {
        bail!(
            "unsupported catalog version {} (expected {})",
            version,
            FORMAT_VERSION
        );
    }
    let _flags = u32::from_le_bytes(header[8..12].try_into().unwrap());
    let region_count = u32::from_le_bytes(header[12..16].try_into().unwrap()) as u64;

    let table_bytes = storage.read_range(REGION_HEADER_LEN, region_count * REGION_ENTRY_LEN)?;
    let mut out = Vec::with_capacity(region_count as usize);
    for i in 0..region_count as usize {
        let base = i * REGION_ENTRY_LEN as usize;
        let entry = &table_bytes[base..base + REGION_ENTRY_LEN as usize];
        let name = parse_name(&entry[..16]);
        let body_offset = u64::from_le_bytes(entry[16..24].try_into().unwrap());
        let body_len = u64::from_le_bytes(entry[24..32].try_into().unwrap());
        let flags = u32::from_le_bytes(entry[32..36].try_into().unwrap());
        if body_offset + body_len > total {
            bail!("region {name:?} extends past end of file");
        }
        out.push(RegionEntry {
            name,
            body_offset,
            body_len,
            flags,
        });
    }
    Ok(out)
}

pub fn read_region_bytes<'a>(
    storage: &'a dyn CatalogStorage,
    entry: &RegionEntry,
) -> Result<std::borrow::Cow<'a, [u8]>> {
    storage.read_range(entry.body_offset, entry.body_len)
}

/// Decode every region into a fully-populated [`CatalogPayload`]. Use this
/// for the native CLI where every code path may need the full corpus.
pub fn read_payload(storage: &dyn CatalogStorage) -> Result<CatalogPayload> {
    read_payload_with(storage, RegionSet::All)
}

/// Decode only the regions needed by the search index itself: courses,
/// sections, and the optional prebuilt-text bundle. The browser's wasm
/// build uses this so cold start skips the bincode-decode of the
/// professors and fce_rows regions, which the search UI doesn't read.
pub fn read_payload_minimal(storage: &dyn CatalogStorage) -> Result<CatalogPayload> {
    read_payload_with(storage, RegionSet::IndexOnly)
}

#[derive(Copy, Clone)]
enum RegionSet {
    All,
    IndexOnly,
}

fn read_payload_with(storage: &dyn CatalogStorage, set: RegionSet) -> Result<CatalogPayload> {
    let table = read_table(storage)?;
    let mut courses: Option<Vec<Course>> = None;
    let mut professors: Option<Vec<Professor>> = None;
    let mut sections: Option<Vec<SectionTime>> = None;
    let mut fce_rows: Option<Vec<FceRow>> = None;
    let mut prebuilt_text: Option<PrebuiltText> = None;

    for entry in &table {
        let want = match (entry.name.as_str(), set) {
            (REGION_COURSES | REGION_SECTIONS | REGION_PREBUILT_TEXT, _) => true,
            (REGION_PROFESSORS | REGION_FCE_ROWS, RegionSet::All) => true,
            (REGION_PROFESSORS | REGION_FCE_ROWS, RegionSet::IndexOnly) => false,
            _ => false,
        };
        if !want {
            continue;
        }
        let bytes = read_region_bytes(storage, entry)?;
        match entry.name.as_str() {
            REGION_COURSES => courses = Some(decode_bincode(&bytes, "courses")?),
            REGION_PROFESSORS => professors = Some(decode_bincode(&bytes, "professors")?),
            REGION_SECTIONS => sections = Some(decode_bincode(&bytes, "sections")?),
            REGION_FCE_ROWS => fce_rows = Some(decode_bincode(&bytes, "fce_rows")?),
            REGION_PREBUILT_TEXT => prebuilt_text = Some(decode_bincode(&bytes, "prebuilt_text")?),
            _ => {}
        }
    }

    let mut corpus = Corpus {
        courses: courses.ok_or_else(|| anyhow::anyhow!("missing 'courses' region"))?,
        professors: match set {
            RegionSet::All => {
                professors.ok_or_else(|| anyhow::anyhow!("missing 'professors' region"))?
            }
            RegionSet::IndexOnly => Vec::new(),
        },
        sections: sections.ok_or_else(|| anyhow::anyhow!("missing 'sections' region"))?,
        fce_rows: match set {
            RegionSet::All => {
                fce_rows.ok_or_else(|| anyhow::anyhow!("missing 'fce_rows' region"))?
            }
            RegionSet::IndexOnly => Vec::new(),
        },
    };
    crate::load::dedup_descriptions(&mut corpus);
    Ok(CatalogPayload {
        corpus,
        prebuilt_text,
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write_payload(corpus: &Corpus, prebuilt_text: Option<&PrebuiltText>) -> Result<Vec<u8>> {
    let mut bodies: Vec<(String, Vec<u8>)> = Vec::with_capacity(5);
    bodies.push(encoded_region(REGION_COURSES, &corpus.courses)?);
    bodies.push(encoded_region(REGION_PROFESSORS, &corpus.professors)?);
    bodies.push(encoded_region(REGION_SECTIONS, &corpus.sections)?);
    bodies.push(encoded_region(REGION_FCE_ROWS, &corpus.fce_rows)?);
    if let Some(pt) = prebuilt_text {
        bodies.push(encoded_region(REGION_PREBUILT_TEXT, pt)?);
    }

    let region_count = bodies.len() as u32;
    let table_size = region_count as u64 * REGION_ENTRY_LEN;
    let mut current_offset = REGION_HEADER_LEN + table_size;

    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&region_count.to_le_bytes());

    for (name, body) in &bodies {
        out.extend_from_slice(&encode_name(name));
        out.extend_from_slice(&current_offset.to_le_bytes());
        out.extend_from_slice(&(body.len() as u64).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        current_offset += body.len() as u64;
    }
    for (_, body) in bodies {
        out.extend_from_slice(&body);
    }
    Ok(out)
}

#[cfg(not(target_arch = "wasm32"))]
fn encoded_region<T: serde::Serialize>(name: &str, value: &T) -> Result<(String, Vec<u8>)> {
    let raw = bincode::serde::encode_to_vec(value, config())
        .with_context(|| format!("encoding region {name}"))?;
    Ok((name.to_string(), raw))
}

fn decode_bincode<T: serde::de::DeserializeOwned>(bytes: &[u8], region: &str) -> Result<T> {
    let (value, _) = bincode::serde::decode_from_slice(bytes, config())
        .with_context(|| format!("decoding region {region}"))?;
    Ok(value)
}

fn parse_name(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

#[cfg(not(target_arch = "wasm32"))]
fn encode_name(name: &str) -> [u8; 16] {
    let mut buf = [0u8; 16];
    let bytes = name.as_bytes();
    let n = bytes.len().min(16);
    buf[..n].copy_from_slice(&bytes[..n]);
    buf
}
