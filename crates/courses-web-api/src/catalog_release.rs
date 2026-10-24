//! Installs the pinned, authenticated catalog release outside the static root.
//!
//! `deployments/catalog.json` pins one encrypted `catalog.sqlite` on the CDN by its hashes. Releases are AES-256-GCM
//! ciphertext (a 12-byte nonce, then the sealed gzip of the file), never plaintext member data. The manifest is
//! committed with the application, so a deploy always gets the catalog it was tested with.
//!
//! ```json
//! { "id": "<uuid>", "schema_version": 2,
//!   "files": { "catalog.sqlite": { "url": "https://...", "sha256": "...", "bytes": 1,
//!                                  "plain_sha256": "...", "plain_bytes": 1 } } }
//! ```

use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
};

use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use sha2::{Digest, Sha256};

const MANIFEST: &str = include_str!("../../../deployments/catalog.json");

#[derive(Deserialize, Debug)]
struct Release {
    id: String,
    schema_version: Option<u32>,
    files: BTreeMap<String, Artifact>,
}

#[derive(Deserialize, Debug)]
struct Artifact {
    url: String,
    sha256: String,
    bytes: usize,
    plain_sha256: String,
    plain_bytes: usize,
}

/// Parses a manifest, refusing one that doesn't pin a `catalog.sqlite` this build reads.
fn parse(manifest: &str) -> Result<Release> {
    let release: Release = serde_json::from_str(manifest)
        .context("deployments/catalog.json isn't a release manifest")?;
    if !release.files.contains_key(courses_catalog::FILE_NAME) {
        bail!(
            "deployments/catalog.json pins an old-format release ({}) instead of one `{}`. Build a catalog with \
             `courses-index build`, publish it and commit the new manifest before deploying this build",
            release.files.keys().cloned().collect::<Vec<_>>().join(", "),
            courses_catalog::FILE_NAME,
        );
    }
    if release.files.len() != 1 {
        bail!(
            "deployments/catalog.json must pin exactly one file, {}",
            courses_catalog::FILE_NAME
        );
    }
    match release.schema_version {
        Some(v) if v == courses_catalog::SCHEMA_VERSION => Ok(release),
        Some(v) => bail!(
            "deployments/catalog.json pins catalog schema {v}, but this build reads schema {}",
            courses_catalog::SCHEMA_VERSION
        ),
        None => bail!("deployments/catalog.json has no schema_version"),
    }
}

/// The catalog file to serve: `$CATALOG_DIR/catalog.sqlite` when set; the local build at `data/catalog` when no
/// managed database is configured; otherwise the pinned release, installed under the database's directory.
pub async fn prepare(managed_db: Option<&Path>) -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("CATALOG_DIR").filter(|d| !d.is_empty()) {
        return Ok(PathBuf::from(dir).join(courses_catalog::FILE_NAME));
    }
    let Some(db_path) = managed_db else {
        return Ok(Path::new("data/catalog").join(courses_catalog::FILE_NAME));
    };
    let release = parse(MANIFEST)?;
    let root = db_path
        .parent()
        .context("DATABASE_PATH must have a parent directory")?
        .join("catalogs");
    install(&release, &root).await
}

async fn install(release: &Release, root: &Path) -> Result<PathBuf> {
    let dest = root.join(&release.id);
    let file = dest.join(courses_catalog::FILE_NAME);
    if dest.join("complete").is_file() {
        return Ok(file);
    }
    let key = STANDARD.decode(
        std::env::var("CATALOG_ENCRYPTION_KEY")
            .context("CATALOG_ENCRYPTION_KEY is required to install the catalog")?,
    )?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| anyhow::anyhow!("CATALOG_ENCRYPTION_KEY must be 32 bytes"))?;
    let artifact = &release.files[courses_catalog::FILE_NAME];
    if !artifact.url.starts_with("https://") {
        bail!("catalog downloads require HTTPS");
    }
    let stage = root.join(format!(".stage-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir_all(&stage).await?;
    let result: Result<()> = async {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()?;
        let bytes = client
            .get(&artifact.url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        if bytes.len() != artifact.bytes
            || format!("{:x}", Sha256::digest(&bytes)) != artifact.sha256
        {
            bail!("catalog ciphertext checksum mismatch");
        }
        let plain = decrypt(&cipher, &bytes, artifact.plain_bytes)?;
        if format!("{:x}", Sha256::digest(&plain)) != artifact.plain_sha256 {
            bail!("catalog plaintext checksum mismatch");
        }
        let staged = stage.join(courses_catalog::FILE_NAME);
        tokio::fs::write(&staged, plain).await?;
        // A file that decrypts but isn't a catalog this build reads must not be installed.
        let check = staged.clone();
        tokio::task::spawn_blocking(move || courses_catalog::Reader::open(&check).map(drop))
            .await??;
        tokio::fs::write(stage.join("complete"), &release.id).await?;
        tokio::fs::rename(&stage, &dest).await?;
        Ok(())
    }
    .await;
    if result.is_err() {
        let _ = tokio::fs::remove_dir_all(&stage).await;
    }
    result?;
    Ok(file)
}

fn decrypt(cipher: &Aes256Gcm, bytes: &[u8], expected: usize) -> Result<Vec<u8>> {
    if bytes.len() < 28 {
        bail!("truncated catalog artifact");
    }
    let nonce = <[u8; 12]>::try_from(&bytes[..12]).expect("12 bytes");
    let compressed = cipher
        .decrypt(&Nonce::from(nonce), &bytes[12..])
        .map_err(|_| {
            anyhow::anyhow!("catalog authentication failed; check CATALOG_ENCRYPTION_KEY")
        })?;
    let mut output = Vec::new();
    flate2::read::GzDecoder::new(compressed.as_slice())
        .take(expected as u64 + 1)
        .read_to_end(&mut output)?;
    if output.len() != expected {
        bail!("catalog expanded size mismatch");
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn tampering_wrong_keys_and_wrong_sizes_fail() {
        let cipher = Aes256Gcm::new_from_slice(&[1; 32]).unwrap();
        let mut zip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        zip.write_all(b"catalog").unwrap();
        let mut bytes = vec![2; 12];
        bytes.extend(
            cipher
                .encrypt(&Nonce::from([2u8; 12]), zip.finish().unwrap().as_slice())
                .unwrap(),
        );
        assert_eq!(decrypt(&cipher, &bytes, 7).unwrap(), b"catalog");
        assert!(decrypt(&cipher, &bytes, 8).is_err());
        assert!(decrypt(&Aes256Gcm::new_from_slice(&[3; 32]).unwrap(), &bytes, 7).is_err());
        bytes[15] ^= 1;
        assert!(decrypt(&cipher, &bytes, 7).is_err());
    }

    fn manifest(files: &[&str], schema: Option<u32>) -> String {
        let artifact = serde_json::json!({
            "url": "https://cdn.test/x.enc", "sha256": "0", "bytes": 1, "plain_sha256": "0", "plain_bytes": 1
        });
        let files: serde_json::Map<_, _> = files
            .iter()
            .map(|f| (f.to_string(), artifact.clone()))
            .collect();
        let mut m = serde_json::json!({ "id": "r1", "files": files });
        if let Some(v) = schema {
            m["schema_version"] = v.into();
        }
        m.to_string()
    }

    #[test]
    fn manifests_must_pin_one_current_catalog() {
        let current = courses_catalog::SCHEMA_VERSION;
        assert!(parse(&manifest(&["catalog.sqlite"], Some(current))).is_ok());
        let old = parse(&manifest(&["public.sqlite", "member.sqlite"], None)).unwrap_err();
        assert!(format!("{old:#}").contains("old-format release"), "{old:#}");
        assert!(parse(&manifest(&["catalog.sqlite"], Some(current + 1))).is_err());
        assert!(parse(&manifest(&["catalog.sqlite"], None)).is_err());
        assert!(parse(&manifest(&["catalog.sqlite", "extra"], Some(current))).is_err());
    }

    /// The committed manifest either pins a current release or is the old format this branch replaces; anything else
    /// is a broken manifest.
    #[test]
    fn committed_manifest_is_recognized() {
        if let Err(e) = parse(MANIFEST) {
            let e = format!("{e:#}");
            assert!(e.contains("old-format release"), "{e}");
        }
    }
}
