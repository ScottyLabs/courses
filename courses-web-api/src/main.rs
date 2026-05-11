//! Web-app backend serving the catalog binary at `/catalog/binary`, the
//! catalog content hash at `/catalog/version`, the OpenAPI spec at
//! `/openapi.json`, and (when `--static-dir` is set) the SPA bundle at `/`.
//!
//! The catalog comes from one of two sources:
//!
//! - **S3 mode** when `--s3-bucket` is set. The handler polls the object's
//!   ETag every `--poll-interval` seconds; on change it pulls the body into
//!   memory and serves it from there. The canonical mode used by the main
//!   deploy and by local dev (against the in-devenv garage instance).
//! - **Upstream proxy mode** when `--catalog-upstream-url` is set. The
//!   handler reverse-proxies `/catalog/version` and `/catalog/binary` to
//!   the URL. Used by PR/staging deploys that piggyback on the main
//!   deploy's catalog rather than getting their own scrape.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use arc_swap::ArcSwap;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::Client as S3Client;
use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use clap::Parser;
use serde::Serialize;
use tokio::net::TcpListener;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{info, warn};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "HOST", default_value = "0.0.0.0")]
    host: String,

    #[arg(long, env = "PORT", default_value_t = 3002)]
    port: u16,

    /// S3 bucket holding `catalog.bin`. Mutex with `--catalog-upstream-url`.
    #[arg(long, env = "S3_BUCKET")]
    s3_bucket: Option<String>,

    /// S3 endpoint URL (e.g. `https://s3.scottylabs.org`). Defaults to AWS
    /// public S3 if unset.
    #[arg(long, env = "S3_ENDPOINT")]
    s3_endpoint: Option<String>,

    /// Object key inside the bucket.
    #[arg(long, env = "S3_KEY", default_value = "catalog.bin")]
    s3_key: String,

    /// Seconds between S3 ETag checks. The scraper publishes manually so
    /// the cadence can be coarse.
    #[arg(long, env = "POLL_INTERVAL", default_value_t = 300)]
    poll_interval: u64,

    /// Reverse-proxy `/catalog/*` to this URL instead of reading S3 directly.
    /// Mutex with `--s3-bucket`.
    #[arg(long, env = "CATALOG_UPSTREAM_URL")]
    catalog_upstream_url: Option<String>,

    /// When set, the frontend SPA bundle at this path is served at `/`.
    #[arg(long, env = "STATIC_DIR")]
    static_dir: Option<PathBuf>,

    /// Print the OpenAPI spec as JSON to stdout and exit. Used by the
    /// frontend's type-generation step.
    #[arg(long)]
    emit_openapi: bool,
}

#[derive(Clone)]
enum CatalogSource {
    S3 {
        cache: Arc<ArcSwap<S3Cache>>,
    },
    Upstream {
        client: reqwest::Client,
        base: String,
    },
}

struct S3Cache {
    etag: String,
    bytes: Bytes,
}

#[derive(Clone)]
struct AppState {
    source: CatalogSource,
}

/// Identity of the currently-served catalog file.
#[derive(Serialize, Clone, ToSchema)]
struct CatalogVersion {
    /// Opaque content identifier. In S3 mode this is the object ETag; in
    /// upstream-proxy mode it is whatever the upstream returned. The OPFS
    /// cache only needs it to be stable across re-fetches of identical
    /// content.
    hash: String,
    /// File size in bytes.
    bytes: u64,
}

#[derive(OpenApi)]
#[openapi(
    info(title = "courses-web-api", version = "0.1.0"),
    components(schemas(CatalogVersion))
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    if args.emit_openapi {
        let dummy = AppState {
            source: CatalogSource::Upstream {
                client: reqwest::Client::new(),
                base: String::new(),
            },
        };
        let (_, openapi) = build_router(dummy).split_for_parts();
        println!("{}", openapi.to_pretty_json()?);
        return Ok(());
    }

    if args.s3_bucket.is_some() == args.catalog_upstream_url.is_some() {
        return Err(anyhow!(
            "exactly one of --s3-bucket, --catalog-upstream-url must be set"
        ));
    }

    let source = if let Some(bucket) = &args.s3_bucket {
        let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let mut loader =
            aws_config::defaults(BehaviorVersion::latest()).region(Region::new(region));
        if let Some(endpoint) = &args.s3_endpoint {
            loader = loader.endpoint_url(endpoint.clone());
        }
        let conf = loader.load().await;
        let s3 = aws_sdk_s3::config::Builder::from(&conf)
            .force_path_style(true)
            .build();
        let client = S3Client::from_conf(s3);

        let initial = fetch_object(&client, bucket, &args.s3_key).await?;
        info!(
            bucket = %bucket,
            key = %args.s3_key,
            etag = %initial.etag,
            bytes = initial.bytes.len(),
            "initial catalog fetched"
        );
        let cache = Arc::new(ArcSwap::from_pointee(initial));
        spawn_s3_poller(
            client.clone(),
            bucket.clone(),
            args.s3_key.clone(),
            Duration::from_secs(args.poll_interval),
            cache.clone(),
        );
        CatalogSource::S3 { cache }
    } else {
        let url = args.catalog_upstream_url.as_ref().unwrap();
        info!(upstream = %url, "running in upstream-proxy mode");
        CatalogSource::Upstream {
            client: reqwest::Client::new(),
            base: url.trim_end_matches('/').to_string(),
        }
    };

    let state = AppState { source };
    let (api_router, openapi) = build_router(state).split_for_parts();
    let mut app = api_router.merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", openapi));

    if let Some(dir) = args.static_dir.as_ref() {
        let static_files = ServeDir::new(dir)
            .append_index_html_on_directories(true)
            .fallback(ServeFile::new(dir.join("index.html")));
        app = app.fallback_service(static_files);
    }

    let app = app.layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr).await?;
    info!(
        addr = %addr,
        static_dir = ?args.static_dir.as_ref().map(|p| p.display().to_string()),
        "listening"
    );
    axum::serve(listener, app).await?;
    Ok(())
}

async fn fetch_object(client: &S3Client, bucket: &str, key: &str) -> Result<S3Cache> {
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
    Ok(S3Cache { etag, bytes })
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

fn spawn_s3_poller(
    client: S3Client,
    bucket: String,
    key: String,
    interval: Duration,
    cache: Arc<ArcSwap<S3Cache>>,
) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(interval).await;
            let current_etag = cache.load().etag.clone();
            match fetch_etag(&client, &bucket, &key).await {
                Ok(remote_etag) if remote_etag != current_etag => {
                    match fetch_object(&client, &bucket, &key).await {
                        Ok(next) => {
                            info!(
                                etag = %next.etag,
                                bytes = next.bytes.len(),
                                "catalog refreshed"
                            );
                            cache.store(Arc::new(next));
                        }
                        Err(e) => warn!(error = %e, "refetch failed"),
                    }
                }
                Ok(_) => {}
                Err(e) => warn!(error = %e, "etag check failed"),
            }
        }
    });
}

#[utoipa::path(get, path = "/health", responses((status = 200, body = String)))]
async fn health() -> &'static str {
    "ok"
}

#[utoipa::path(
    get,
    path = "/catalog/version",
    responses((status = 200, body = CatalogVersion))
)]
async fn catalog_version(State(state): State<AppState>) -> Response {
    match &state.source {
        CatalogSource::S3 { cache, .. } => {
            let cur = cache.load_full();
            axum::Json(CatalogVersion {
                hash: cur.etag.clone(),
                bytes: cur.bytes.len() as u64,
            })
            .into_response()
        }
        CatalogSource::Upstream { client, base } => {
            match client.get(format!("{base}/catalog/version")).send().await {
                Ok(resp) => {
                    let status =
                        StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::OK);
                    let bytes = resp.bytes().await.unwrap_or_default();
                    let mut response = Response::new(axum::body::Body::from(bytes));
                    *response.status_mut() = status;
                    response
                        .headers_mut()
                        .insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
                    response
                }
                Err(e) => (StatusCode::BAD_GATEWAY, format!("upstream: {e}")).into_response(),
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/catalog/binary",
    responses(
        (status = 200, description = "Catalog binary, gzipped (Content-Encoding: gzip)", content_type = "application/octet-stream"),
        (status = 502, description = "Upstream proxy failure"),
        (status = 503, description = "Catalog unavailable")
    )
)]
async fn catalog_binary(State(state): State<AppState>) -> Response {
    match &state.source {
        CatalogSource::S3 { cache, .. } => {
            let cur = cache.load_full();
            let is_gzip = cur.bytes.starts_with(&[0x1f, 0x8b]);
            let mut response = Response::new(axum::body::Body::from(cur.bytes.clone()));
            let h = response.headers_mut();
            h.insert(
                header::CONTENT_TYPE,
                "application/octet-stream".parse().unwrap(),
            );
            h.insert(header::CONTENT_LENGTH, cur.bytes.len().into());
            h.insert(header::CACHE_CONTROL, "public, max-age=60".parse().unwrap());
            if is_gzip {
                h.insert(header::CONTENT_ENCODING, "gzip".parse().unwrap());
            }
            response
        }
        CatalogSource::Upstream { client, base } => {
            match client.get(format!("{base}/catalog/binary")).send().await {
                Ok(resp) => {
                    let status =
                        StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::OK);
                    let mut response =
                        Response::new(axum::body::Body::from_stream(resp.bytes_stream()));
                    *response.status_mut() = status;
                    response.headers_mut().insert(
                        header::CONTENT_TYPE,
                        "application/octet-stream".parse().unwrap(),
                    );
                    response
                        .headers_mut()
                        .insert(header::CONTENT_ENCODING, "gzip".parse().unwrap());
                    response
                }
                Err(e) => (StatusCode::BAD_GATEWAY, format!("upstream: {e}")).into_response(),
            }
        }
    }
}

fn build_router(state: AppState) -> OpenApiRouter {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .routes(routes!(catalog_version))
        .routes(routes!(catalog_binary))
        .with_state(state)
}
