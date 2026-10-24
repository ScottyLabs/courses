//! The catalog: a read-only `catalog.sqlite` (decision 15), its JSON documents (decision 16) and the public course API.

pub mod courses;

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::Context;
use axum::{
    Extension,
    body::Body,
    extract::Path as UrlPath,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use courses_catalog::{Lookup, Reader};

use crate::{
    auth::Viewer,
    error::{ApiError, ApiResult},
    state::AppState,
};

/// Readers kept open between requests.
const POOL_MAX: usize = 8;

#[derive(Clone)]
pub struct Catalog {
    path: PathBuf,
    pool: Arc<Mutex<Vec<Reader>>>,
}

impl Catalog {
    /// Opens the catalog once to check it, failing when the file is missing or has another schema version.
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let reader = Reader::open(path).with_context(|| {
            format!(
                "the catalog at {} can't be read; build one with `courses-index build` or set CATALOG_DIR",
                path.display()
            )
        })?;
        let terms: i64 = reader
            .connection()
            .query_row("SELECT count(*) FROM terms", [], |r| r.get(0))?;
        if terms == 0 {
            anyhow::bail!("the catalog at {} has no terms", path.display());
        }
        Ok(Self {
            path: path.to_owned(),
            pool: Arc::new(Mutex::new(vec![reader])),
        })
    }

    /// Runs `f` with a pooled reader on a blocking thread.
    pub async fn with<T, F>(&self, f: F) -> ApiResult<T>
    where
        T: Send + 'static,
        F: FnOnce(&Reader) -> ApiResult<T> + Send + 'static,
    {
        let catalog = self.clone();
        tokio::task::spawn_blocking(move || {
            let pooled = catalog.pool.lock().ok().and_then(|mut p| p.pop());
            let reader = match pooled {
                Some(reader) => reader,
                None => Reader::open(&catalog.path).map_err(|e| {
                    tracing::error!(error = %format!("{e:#}"), "opening the catalog failed");
                    ApiError::unavailable("The catalog is unavailable.")
                })?,
            };
            let result = f(&reader);
            if let Ok(mut pool) = catalog.pool.lock()
                && pool.len() < POOL_MAX
            {
                pool.push(reader);
            }
            result
        })
        .await
        .map_err(ApiError::internal)?
    }

    /// Health: a fresh reader checks the file and its schema version.
    pub async fn check(&self) -> ApiResult<String> {
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || {
            let reader = Reader::open(&path).map_err(|e| {
                tracing::error!(error = %format!("{e:#}"), "catalog health check failed");
                ApiError::unavailable("The catalog is unavailable.")
            })?;
            Ok(reader.schema_version()?.to_string())
        })
        .await
        .map_err(ApiError::internal)?
    }
}

/// Whether the client takes gzip.
fn accepts_gzip(headers: &HeaderMap) -> bool {
    headers
        .get_all(header::ACCEPT_ENCODING)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .any(|part| {
            let mut pieces = part.split(';');
            let coding = pieces.next().unwrap_or("").trim();
            let q_zero = pieces.any(|p| {
                p.trim()
                    .strip_prefix("q=")
                    .and_then(|q| q.trim().parse::<f32>().ok())
                    == Some(0.0)
            });
            (coding.eq_ignore_ascii_case("gzip") || coding == "*") && !q_zero
        })
}

/// Whether `If-None-Match` names this entity tag (weak comparison).
pub fn not_modified(headers: &HeaderMap, etag: &str) -> bool {
    headers
        .get_all(header::IF_NONE_MATCH)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .map(|t| t.trim())
        .any(|t| t == "*" || t.trim_start_matches("W/").trim_matches('"') == etag)
}

/// A catalog document, as described in decision 16. Signed-in CMU users get the member version when there is one;
/// everyone else gets the public version. The body is the stored gzip, decompressed only for clients that don't
/// accept gzip.
#[utoipa::path(
    get,
    path = "/api/catalog/{name}",
    tag = "catalog",
    params(("name" = String, Path, description = "Document name, such as `terms`, `term/S27` or `fce/courses/15`")),
    responses(
        (status = OK, description = "The JSON document", content_type = "application/json", body = serde_json::Value),
        (status = NOT_MODIFIED, description = "The document matches `If-None-Match`"),
        (status = UNAUTHORIZED, description = "The document is for signed-in CMU users only", body = crate::error::ErrorBody),
        (status = NOT_FOUND, body = crate::error::ErrorBody),
    )
)]
pub async fn document(
    viewer: Viewer,
    Extension(state): Extension<AppState>,
    UrlPath(name): UrlPath<String>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let member = viewer.is_member();
    let lookup = state
        .catalog
        .with(move |r| Ok(r.document(&name, member)?))
        .await?;
    let doc = match lookup {
        Lookup::Found(doc) => doc,
        Lookup::MemberOnly if matches!(viewer, Viewer::NonCmu) => {
            return Err(ApiError::forbidden(
                "This is only available to CMU accounts.",
            ));
        }
        Lookup::MemberOnly => {
            return Err(ApiError::auth_msg(
                "Sign in with your CMU account to see this.",
            ));
        }
        Lookup::Missing => return Err(ApiError::not_found("No such catalog document.")),
    };
    let etag = HeaderValue::from_str(&format!("\"{}\"", doc.etag)).map_err(ApiError::internal)?;
    let common = [
        (header::ETAG, etag),
        (
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-cache"),
        ),
        (
            header::VARY,
            HeaderValue::from_static("Cookie, Accept-Encoding"),
        ),
    ];
    if not_modified(&headers, &doc.etag) {
        return Ok((StatusCode::NOT_MODIFIED, common).into_response());
    }
    let json = HeaderValue::from_static("application/json");
    if accepts_gzip(&headers) {
        Ok((
            common,
            [
                (header::CONTENT_TYPE, json),
                (header::CONTENT_ENCODING, HeaderValue::from_static("gzip")),
            ],
            Body::from(doc.gzip),
        )
            .into_response())
    } else {
        let plain = tokio::task::spawn_blocking(move || courses_catalog::gunzip(&doc.gzip))
            .await
            .map_err(ApiError::internal)??;
        Ok((common, [(header::CONTENT_TYPE, json)], Body::from(plain)).into_response())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_negotiation_and_etags() {
        let mut h = HeaderMap::new();
        assert!(!accepts_gzip(&h));
        h.insert(header::ACCEPT_ENCODING, "br, gzip;q=0.8".parse().unwrap());
        assert!(accepts_gzip(&h));
        h.insert(header::ACCEPT_ENCODING, "gzip;q=0, br".parse().unwrap());
        assert!(!accepts_gzip(&h));
        h.insert(header::ACCEPT_ENCODING, "*".parse().unwrap());
        assert!(accepts_gzip(&h));

        h.insert(header::IF_NONE_MATCH, "\"a\", W/\"abc\"".parse().unwrap());
        assert!(not_modified(&h, "abc"));
        assert!(!not_modified(&h, "ab"));
    }
}
