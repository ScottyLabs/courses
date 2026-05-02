//! Storage abstraction for the catalog blob. Decouples "where bytes come
//! from" from "how the catalog is decoded" so the same reader logic works
//! against an in-memory `Vec<u8>`, a native file handle, or (eventually) an
//! OPFS file in the browser. Inspired by the `MemoryProvider` pattern in
//! the veeso/wasm-dbms project.
//!
//! Reads are addressed by byte offset and length. The default
//! [`CatalogStorage::read_all`] impl just calls `read_range(0, len())` and
//! is fine for storage backends that already hold the full buffer; lazier
//! impls (mmap, OPFS) override it to avoid forcing a full materialization.

use anyhow::{Context, Result, bail};

#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{Read, Seek, SeekFrom};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;

pub trait CatalogStorage {
    fn len(&self) -> Result<u64>;

    fn read_range(&self, offset: u64, len: u64) -> Result<Vec<u8>>;

    fn read_all(&self) -> Result<Vec<u8>> {
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

impl<'a> CatalogStorage for MemoryStorage<'a> {
    fn len(&self) -> Result<u64> {
        Ok(self.bytes.len() as u64)
    }

    fn read_range(&self, offset: u64, len: u64) -> Result<Vec<u8>> {
        let start = offset as usize;
        let end = start.checked_add(len as usize).context("range overflow")?;
        if end > self.bytes.len() {
            bail!(
                "read out of bounds: offset={} len={} buffer_len={}",
                offset,
                len,
                self.bytes.len()
            );
        }
        Ok(self.bytes[start..end].to_vec())
    }

    fn read_all(&self) -> Result<Vec<u8>> {
        Ok(self.bytes.to_vec())
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

    fn read_range(&self, offset: u64, len: u64) -> Result<Vec<u8>> {
        MemoryStorage::new(&self.bytes).read_range(offset, len)
    }

    fn read_all(&self) -> Result<Vec<u8>> {
        Ok(self.bytes.clone())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct FileStorage {
    file: Mutex<File>,
    len: u64,
}

#[cfg(not(target_arch = "wasm32"))]
impl FileStorage {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
        let len = file.metadata()?.len();
        Ok(Self {
            file: Mutex::new(file),
            len,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl CatalogStorage for FileStorage {
    fn len(&self) -> Result<u64> {
        Ok(self.len)
    }

    fn read_range(&self, offset: u64, len: u64) -> Result<Vec<u8>> {
        let mut file = self
            .file
            .lock()
            .map_err(|_| anyhow::anyhow!("storage lock poisoned"))?;
        file.seek(SeekFrom::Start(offset)).context("seek failed")?;
        let mut buf = vec![0u8; len as usize];
        file.read_exact(&mut buf).context("read failed")?;
        Ok(buf)
    }
}
