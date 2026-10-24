//! Install a pinned, authenticated catalog generation outside the static root.
//!
//! Releases on the CDN are AES-256-GCM ciphertext, never plaintext member data.
//! The manifest is committed with the application, making rollback deterministic.
use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
};

const FILES: [&str; 6] = [
    "public.sqlite",
    "member.sqlite",
    "public-browser.sqlite",
    "member-browser.sqlite",
    "public-browser.sqlite.gz",
    "member-browser.sqlite.gz",
];
#[derive(Deserialize)]
struct Release {
    id: String,
    files: BTreeMap<String, Artifact>,
}
#[derive(Deserialize)]
struct Artifact {
    url: String,
    sha256: String,
    bytes: usize,
    plain_sha256: String,
    plain_bytes: usize,
}

pub async fn prepare(user_path: &str) -> Result<PathBuf> {
    if let Ok(path) = std::env::var("CATALOG_DIR") {
        return Ok(path.into());
    }
    // Local development uses the importer output; managed deployments use a pinned release.
    if std::env::var_os("DATABASE_PATH").is_none() {
        return Ok("data/catalog".into());
    }
    let release: Release = serde_json::from_str(include_str!("../../../deployments/catalog.json"))?;
    let root = Path::new(user_path)
        .parent()
        .context("DATABASE_PATH must have a parent")?
        .join("catalogs");
    let dest = root.join(&release.id);
    if dest.join("complete").is_file() {
        return Ok(dest);
    }
    let key = STANDARD.decode(
        std::env::var("CATALOG_ENCRYPTION_KEY").context("CATALOG_ENCRYPTION_KEY is required")?,
    )?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| anyhow::anyhow!("Catalog key must be 32 bytes"))?;
    let stage = root.join(format!(".stage-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir_all(&stage).await?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()?;
    let result: Result<()> = async {
        for name in FILES {
            let artifact = release
                .files
                .get(name)
                .context("Incomplete release manifest")?;
            if !artifact.url.starts_with("https://") {
                bail!("Catalog download requires HTTPS");
            }
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
                bail!("Catalog ciphertext checksum mismatch: {name}");
            }
            let plain = decrypt(&cipher, &bytes, artifact.plain_bytes)?;
            if format!("{:x}", Sha256::digest(&plain)) != artifact.plain_sha256 {
                bail!("Catalog plaintext checksum mismatch: {name}");
            }
            tokio::fs::write(stage.join(name), plain).await?;
        }
        tokio::fs::write(stage.join("complete"), &release.id).await?;
        tokio::fs::rename(&stage, &dest).await?;
        Ok(())
    }
    .await;
    if result.is_err() {
        let _ = tokio::fs::remove_dir_all(&stage).await;
    }
    result?;
    Ok(dest)
}

fn decrypt(cipher: &Aes256Gcm, bytes: &[u8], expected: usize) -> Result<Vec<u8>> {
    if bytes.len() < 28 {
        bail!("Truncated catalog artifact");
    }
    let compressed = cipher
        .decrypt(
            &Nonce::from(<[u8; 12]>::try_from(&bytes[..12]).unwrap()),
            &bytes[12..],
        )
        .map_err(|_| anyhow::anyhow!("Catalog authentication failed"))?;
    let mut output = Vec::new();
    flate2::read::GzDecoder::new(compressed.as_slice())
        .take(expected as u64 + 1)
        .read_to_end(&mut output)?;
    if output.len() != expected {
        bail!("Catalog expanded size mismatch");
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
}
