//! Public-facing course catalog API. Pulls the catalog binary from S3,
//! builds an in-memory index, and exposes versioned REST routes under
//! `/v1/`. Re-fetches and rebuilds on S3 ETag change.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use arc_swap::ArcSwap;
use aws_sdk_s3::Client as S3Client;
use axum::{
    Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use clap::Parser;
use courses_index::{binary, index::Index};
use serde::Serialize;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "BIND_ADDR", default_value = "0.0.0.0:3001")]
    bind: String,

    #[arg(long, env = "S3_BUCKET")]
    s3_bucket: String,

    #[arg(long, env = "S3_ENDPOINT")]
    s3_endpoint: Option<String>,

    #[arg(long, env = "S3_KEY", default_value = "catalog.bin")]
    s3_key: String,

    #[arg(long, env = "POLL_INTERVAL", default_value_t = 300)]
    poll_interval: u64,
}

struct Catalog {
    etag: String,
    bytes: bytes::Bytes,
    index: Index,
}

#[derive(Clone)]
struct AppState {
    catalog: Arc<ArcSwap<Catalog>>,
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

    let args = Args::parse();

    let mut loader = aws_config::from_env();
    if let Some(endpoint) = &args.s3_endpoint {
        loader = loader.endpoint_url(endpoint.clone());
    }
    let conf = loader.load().await;
    let s3 = aws_sdk_s3::config::Builder::from(&conf)
        .force_path_style(true)
        .build();
    let client = S3Client::from_conf(s3);

    let initial = fetch_and_build(&client, &args.s3_bucket, &args.s3_key).await?;
    info!(
        bucket = %args.s3_bucket,
        key = %args.s3_key,
        etag = %initial.etag,
        bytes = initial.bytes.len(),
        courses = initial.index.n_docs,
        "catalog loaded"
    );

    let catalog = Arc::new(ArcSwap::from_pointee(initial));
    spawn_s3_poller(
        client.clone(),
        args.s3_bucket.clone(),
        args.s3_key.clone(),
        Duration::from_secs(args.poll_interval),
        catalog.clone(),
    );

    let state = AppState { catalog };
    let app = Router::new()
        .route("/v1/health", get(health))
        .route("/v1/version", get(version))
        .route("/v1/binary", get(binary_handler))
        .route("/v1/courses/{code}", get(course_by_code))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(&args.bind).await?;
    info!(addr = %args.bind, "listening");
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

async fn health() -> &'static str {
    "ok"
}

#[derive(Serialize)]
struct VersionInfo {
    format_version: u32,
    course_count: u32,
    bytes: usize,
    etag: String,
}

async fn version(State(state): State<AppState>) -> impl IntoResponse {
    let cat = state.catalog.load();
    axum::Json(VersionInfo {
        format_version: binary::FORMAT_VERSION,
        course_count: cat.index.n_docs,
        bytes: cat.bytes.len(),
        etag: cat.etag.clone(),
    })
}

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

async fn course_by_code(State(state): State<AppState>, Path(code): Path<String>) -> Response {
    let cat = state.catalog.load();
    let Some(&id) = cat.index.code_to_id.get(&code) else {
        return (StatusCode::NOT_FOUND, "course not found").into_response();
    };
    let course = &cat.index.courses[id as usize];
    axum::Json(course).into_response()
}
