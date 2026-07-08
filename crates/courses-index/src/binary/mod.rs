//! Binary serialization for the catalog. The on-disk format is region-based
//! (see [`region`]) wrapped in an outer gzip layer when written for
//! distribution, which the browser unwraps via the native
//! `DecompressionStream` API and the native side unwraps via `flate2`. The
//! [`storage`] module abstracts where the bytes physically live, with a
//! `MemoryStorage` backed by a borrowed slice and a native `FileStorage`
//! backed by a file handle.

pub mod region;
pub mod storage;

pub use region::{
    CatalogPayload, FORMAT_VERSION, RegionEntry, read_payload, read_payload_minimal, read_table,
};
pub use storage::{CatalogStorage, MemoryStorage, OwnedMemoryStorage};

#[cfg(not(target_arch = "wasm32"))]
pub use storage::FileStorage;

#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
use anyhow::{Context, Result};

#[cfg(all(not(target_arch = "wasm32"), feature = "gzip_runtime"))]
use crate::index::text::PrebuiltText;
#[cfg(all(not(target_arch = "wasm32"), feature = "gzip_runtime"))]
use crate::load::Corpus;

#[cfg(feature = "gzip_runtime")]
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

/// Encode `corpus` (and optionally a prebuilt text bundle) into a catalog
/// file at `path`. When `compress` is true the file is gzip-wrapped so the
/// browser can decode it via `DecompressionStream`. With `compress = false`
/// the file is written raw, useful for binary diff tooling.
#[cfg(all(not(target_arch = "wasm32"), feature = "gzip_runtime"))]
pub fn write_catalog(
    path: &Path,
    corpus: &Corpus,
    prebuilt_text: Option<&PrebuiltText>,
    compress: bool,
) -> Result<()> {
    let raw = region::write_payload(corpus, prebuilt_text)?;
    let body = if compress { gzip_encode(&raw)? } else { raw };
    fs::write(path, &body).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// Read a catalog file from disk, transparently unwrapping the outer gzip
/// layer if present.
#[cfg(all(not(target_arch = "wasm32"), feature = "gzip_runtime"))]
pub fn read_catalog(path: &Path) -> Result<CatalogPayload> {
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let raw = if bytes.starts_with(&GZIP_MAGIC) {
        gzip_decode(&bytes)?
    } else {
        bytes
    };
    let storage = OwnedMemoryStorage::new(raw);
    region::read_payload(&storage)
}

#[cfg(all(not(target_arch = "wasm32"), feature = "gzip_runtime"))]
fn gzip_encode(input: &[u8]) -> Result<Vec<u8>> {
    use std::io::Write;
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(input).context("gzip encode write")?;
    encoder.finish().context("gzip encode finish")
}

#[cfg(feature = "gzip_runtime")]
fn gzip_decode(input: &[u8]) -> anyhow::Result<Vec<u8>> {
    use std::io::Read;
    let mut decoder = flate2::read::GzDecoder::new(input);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| anyhow::anyhow!("gzip decode: {e}"))?;
    Ok(out)
}

/// Decode an in-memory catalog buffer with every region populated. With
/// the `gzip_runtime` feature enabled, transparently unwraps an outer gzip
/// layer.
pub fn read_catalog_from_slice(bytes: &[u8]) -> anyhow::Result<CatalogPayload> {
    read_with(bytes, region::read_payload)
}

/// Decode an in-memory catalog buffer without the `professors` and
/// `fce_rows` regions. The browser wasm build calls this so cold start
/// skips the bincode work for regions the search UI doesn't read.
pub fn read_catalog_minimal_from_slice(bytes: &[u8]) -> anyhow::Result<CatalogPayload> {
    read_with(bytes, region::read_payload_minimal)
}

fn read_with<F>(bytes: &[u8], decode: F) -> anyhow::Result<CatalogPayload>
where
    F: Fn(&dyn CatalogStorage) -> anyhow::Result<CatalogPayload>,
{
    #[cfg(feature = "gzip_runtime")]
    {
        if bytes.starts_with(&GZIP_MAGIC) {
            let raw = gzip_decode(bytes)?;
            let storage = OwnedMemoryStorage::new(raw);
            return decode(&storage);
        }
    }
    let storage = MemoryStorage::new(bytes);
    decode(&storage)
}

/// Write a zstd patch from `old_path` to `new_path`. Pairs with
/// [`apply_patch`].
#[cfg(not(target_arch = "wasm32"))]
pub fn write_patch(old_path: &Path, new_path: &Path, patch_path: &Path) -> Result<()> {
    let old = fs::read(old_path).with_context(|| format!("reading {}", old_path.display()))?;
    let new = fs::read(new_path).with_context(|| format!("reading {}", new_path.display()))?;
    let patch = encode_with_dict(&new, &old)?;
    fs::write(patch_path, &patch).with_context(|| format!("writing {}", patch_path.display()))?;
    Ok(())
}

/// Reconstruct a catalog by applying `patch` against `old`. Mirrors
/// [`write_patch`].
#[cfg(not(target_arch = "wasm32"))]
pub fn apply_patch(old_path: &Path, patch_path: &Path) -> Result<Vec<u8>> {
    let old = fs::read(old_path).with_context(|| format!("reading {}", old_path.display()))?;
    let patch =
        fs::read(patch_path).with_context(|| format!("reading {}", patch_path.display()))?;
    decode_with_dict(&patch, &old)
}

#[cfg(not(target_arch = "wasm32"))]
fn encode_with_dict(input: &[u8], dict: &[u8]) -> Result<Vec<u8>> {
    use std::io::Write;
    const ZSTD_LEVEL: i32 = 19;
    let prepared = zstd::dict::EncoderDictionary::copy(dict, ZSTD_LEVEL);
    let mut output: Vec<u8> = Vec::new();
    let mut encoder = zstd::Encoder::with_prepared_dictionary(&mut output, &prepared)
        .context("zstd encoder init")?;
    encoder.write_all(input).context("zstd patch write")?;
    encoder.finish().context("zstd patch finish")?;
    Ok(output)
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_with_dict(patch: &[u8], dict: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;
    let prepared = zstd::dict::DecoderDictionary::copy(dict);
    let mut decoder =
        zstd::Decoder::with_prepared_dictionary(patch, &prepared).context("zstd decoder init")?;
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).context("zstd patch read")?;
    Ok(out)
}
