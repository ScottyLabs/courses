//! Storage abstraction for the catalog blob. Decouples "where bytes come
//! from" from "how the catalog is decoded" so the same reader logic works
//! against an in-memory `Vec<u8>`, an mmap'd file on disk, or (eventually)
//! an OPFS file in the browser. Inspired by the `MemoryProvider` pattern in
//! the veeso/wasm-dbms project.
//!
//! Reads are addressed by byte offset and length and return `Cow<[u8]>`, so
//! backends that already hold the bytes (in-memory, mmap) hand back a
//! borrowed slice with no copy or allocation.

use std::borrow::Cow;

use anyhow::{Context, Result, bail};

#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
use memmap2::Mmap;

pub trait CatalogStorage {
    fn len(&self) -> Result<u64>;

    fn read_range(&self, offset: u64, len: u64) -> Result<Cow<'_, [u8]>>;

    fn read_all(&self) -> Result<Cow<'_, [u8]>> {
        let n = self.len()?;
        self.read_range(0, n)
    }
}

pub struct MemoryStorage<'a> {
    bytes: &'a [u8],
}

impl<'a> MemoryStorage<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}

impl CatalogStorage for MemoryStorage<'_> {
    fn len(&self) -> Result<u64> {
        Ok(self.bytes.len() as u64)
    }

    fn read_range(&self, offset: u64, len: u64) -> Result<Cow<'_, [u8]>> {
        Ok(Cow::Borrowed(slice_range(self.bytes, offset, len)?))
    }

    fn read_all(&self) -> Result<Cow<'_, [u8]>> {
        Ok(Cow::Borrowed(self.bytes))
    }
}

pub struct OwnedMemoryStorage {
    bytes: Vec<u8>,
}

impl OwnedMemoryStorage {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl CatalogStorage for OwnedMemoryStorage {
    fn len(&self) -> Result<u64> {
        Ok(self.bytes.len() as u64)
    }

    fn read_range(&self, offset: u64, len: u64) -> Result<Cow<'_, [u8]>> {
        Ok(Cow::Borrowed(slice_range(&self.bytes, offset, len)?))
    }

    fn read_all(&self) -> Result<Cow<'_, [u8]>> {
        Ok(Cow::Borrowed(&self.bytes))
    }
}

/// File-backed storage using `mmap`. Region reads return borrowed slices
/// into the mapped buffer with no copy; the kernel pages bytes in on
/// demand and shares the mapping across reads.
#[cfg(not(target_arch = "wasm32"))]
pub struct FileStorage {
    mmap: Mmap,
}

#[cfg(not(target_arch = "wasm32"))]
impl FileStorage {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
        let mmap =
            unsafe { Mmap::map(&file) }.with_context(|| format!("mmap {}", path.display()))?;
        Ok(Self { mmap })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl CatalogStorage for FileStorage {
    fn len(&self) -> Result<u64> {
        Ok(self.mmap.len() as u64)
    }

    fn read_range(&self, offset: u64, len: u64) -> Result<Cow<'_, [u8]>> {
        Ok(Cow::Borrowed(slice_range(&self.mmap, offset, len)?))
    }

    fn read_all(&self) -> Result<Cow<'_, [u8]>> {
        Ok(Cow::Borrowed(&self.mmap))
    }
}

fn slice_range(bytes: &[u8], offset: u64, len: u64) -> Result<&[u8]> {
    let start = offset as usize;
    let end = start.checked_add(len as usize).context("range overflow")?;
    if end > bytes.len() {
        bail!(
            "read out of bounds: offset={} len={} buffer_len={}",
            offset,
            len,
            bytes.len()
        );
    }
    Ok(&bytes[start..end])
}
