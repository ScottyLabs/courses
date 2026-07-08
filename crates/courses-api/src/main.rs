//! Public-facing course catalog API. Pulls the catalog binary from the CDN
//! S3 bucket, builds an in-memory index, and exposes REST routes under
//! `/api`. Re-fetches and rebuilds on S3 ETag change.
//!
//! S3 credentials come from the typed secretspec SDK (`CDN_*`).

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use arc_swap::ArcSwap;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use courses_index::{binary, index::Index};
use serde::Serialize;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

secretspec_derive::declare_secrets!("../../secretspec.toml");

struct Catalog {
    etag: String,
    bytes: bytes::Bytes,
    index: Index,
}

#[derive(Clone)]
struct AppState {
    catalog: Arc<ArcSwap<Catalog>>,
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "health", description = "Health check"),
        (name = "catalog", description = "Course catalog"),
    ),
    info(title = "ScottyLabs Courses API", version = "0.1.0")
)]
struct ApiDoc;

/// Build an S3 client for the CDN bucket from the given endpoint and credentials.
fn build_s3_client(endpoint: String, access_key_id: String, secret_access_key: String) -> S3Client {
    let credentials = Credentials::new(access_key_id, secret_access_key, None, None, "courses-cdn");
    let config = aws_sdk_s3::config::Builder::new()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .endpoint_url(endpoint)
        .credentials_provider(credentials)
        .force_path_style(true)
        .build();
    S3Client::from_conf(config)
}

async fn fetch_and_build(client: &S3Client, bucket: &str, key: &str) -> Result<Catalog> {
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .with_context(|| format!("get_object s3://{bucket}/{key}"))?;
    let etag = resp
        .e_tag()
        .ok_or_else(|| anyhow!("missing ETag on s3://{bucket}/{key}"))?
        .trim_matches('"')
        .to_string();
    let bytes = resp.body.collect().await?.into_bytes();
    let payload = binary::read_catalog_from_slice(&bytes)?;
    let index = match payload.prebuilt_text {
        Some(p) => Index::build_with_prebuilt_text(payload.corpus, p)?,
        None => Index::build(payload.corpus),
    };
    Ok(Catalog { etag, bytes, index })
}

async fn fetch_etag(client: &S3Client, bucket: &str, key: &str) -> Result<String> {
    let resp = client
        .head_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .with_context(|| format!("head_object s3://{bucket}/{key}"))?;
    Ok(resp
        .e_tag()
        .ok_or_else(|| anyhow!("missing ETag on s3://{bucket}/{key}"))?
        .trim_matches('"')
        .to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let secrets = SecretSpec::builder()
        .with_provider("env")
        .with_reason("courses-api startup")
        .load()?
        .secrets;

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".into());
    let s3_key = std::env::var("S3_KEY").unwrap_or_else(|_| "catalog.bin".into());
    let poll_interval: u64 = std::env::var("POLL_INTERVAL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);

    let endpoint = secrets
        .cdn_s3_endpoint
        .context("CDN_S3_ENDPOINT must be set")?;
    let bucket = secrets.cdn_s3_bucket.context("CDN_S3_BUCKET must be set")?;
    let access_key_id = secrets
        .cdn_access_key_id
        .context("CDN_ACCESS_KEY_ID must be set")?;
    let secret_access_key = secrets
        .cdn_secret_access_key
        .context("CDN_SECRET_ACCESS_KEY must be set")?;

    let client = build_s3_client(endpoint, access_key_id, secret_access_key);

    let initial = fetch_and_build(&client, &bucket, &s3_key).await?;
    info!(
        bucket = %bucket,
        key = %s3_key,
        etag = %initial.etag,
        bytes = initial.bytes.len(),
        courses = initial.index.n_docs,
        "catalog loaded"
    );

    let catalog = Arc::new(ArcSwap::from_pointee(initial));
    spawn_s3_poller(
        client.clone(),
        bucket.clone(),
        s3_key.clone(),
        Duration::from_secs(poll_interval),
        catalog.clone(),
    );

    let state = AppState { catalog };
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .routes(routes!(version))
        .routes(routes!(binary_handler))
        .routes(routes!(course_by_code))
        .with_state(state)
        .split_for_parts();

    let app = router
        .merge(Scalar::with_url("/api/scalar", api.clone()))
        .route(
            "/api/openapi.json",
            get(move || {
                let api = api.clone();
                async move { axum::Json(api) }
            }),
        )
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    info!(addr = %addr, "listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn spawn_s3_poller(
    client: S3Client,
    bucket: String,
    key: String,
    interval: Duration,
    catalog: Arc<ArcSwap<Catalog>>,
) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(interval).await;
            let current_etag = catalog.load().etag.clone();
            match fetch_etag(&client, &bucket, &key).await {
                Ok(remote_etag) if remote_etag != current_etag => {
                    match fetch_and_build(&client, &bucket, &key).await {
                        Ok(next) => {
                            info!(
                                etag = %next.etag,
                                courses = next.index.n_docs,
                                "catalog rebuilt"
                            );
                            catalog.store(Arc::new(next));
                        }
                        Err(e) => warn!(error = %e, "rebuild failed"),
                    }
                }
                Ok(_) => {}
                Err(e) => warn!(error = %e, "etag check failed"),
            }
        }
    });
}

#[utoipa::path(get, path = "/api/health", tag = "health", responses((status = OK, body = str)))]
async fn health() -> &'static str {
    "ok"
}

#[derive(Serialize, ToSchema)]
struct VersionInfo {
    format_version: u32,
    course_count: u32,
    bytes: usize,
    etag: String,
}

#[utoipa::path(get, path = "/api/version", tag = "catalog", responses((status = OK, body = VersionInfo)))]
async fn version(State(state): State<AppState>) -> impl IntoResponse {
    let cat = state.catalog.load();
    axum::Json(VersionInfo {
        format_version: binary::FORMAT_VERSION,
        course_count: cat.index.n_docs,
        bytes: cat.bytes.len(),
        etag: cat.etag.clone(),
    })
}

#[utoipa::path(
    get,
    path = "/api/binary",
    tag = "catalog",
    responses((status = OK, description = "Catalog binary, gzipped when Content-Encoding: gzip", content_type = "application/octet-stream"))
)]
async fn binary_handler(State(state): State<AppState>) -> Response {
    let cat = state.catalog.load_full();
    let is_gzip = cat.bytes.starts_with(&[0x1f, 0x8b]);
    let mut response = Response::new(axum::body::Body::from(cat.bytes.clone()));
    let h = response.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    h.insert(header::CACHE_CONTROL, "public, max-age=60".parse().unwrap());
    if is_gzip {
        h.insert(header::CONTENT_ENCODING, "gzip".parse().unwrap());
    }
    response
}

#[utoipa::path(
    get,
    path = "/api/courses/{code}",
    tag = "catalog",
    params(("code" = String, Path, description = "Course code, e.g. 15-122")),
    responses((status = OK, description = "Course JSON"), (status = NOT_FOUND, description = "Unknown course code"))
)]
async fn course_by_code(State(state): State<AppState>, Path(code): Path<String>) -> Response {
    let cat = state.catalog.load();
    let Some(&id) = cat.index.code_to_id.get(&code) else {
        return (StatusCode::NOT_FOUND, "course not found").into_response();
    };
    let course = &cat.index.courses[id as usize];
    axum::Json(course).into_response()
}
