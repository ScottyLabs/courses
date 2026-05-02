//! Web-app backend serving the catalog binary at `/catalog/binary`, the
//! catalog content hash at `/catalog/version`, the OpenAPI spec at
//! `/openapi.json`, and (when `--static-dir` is set) the SPA bundle at `/`.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use arc_swap::ArcSwap;
use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use clap::Parser;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tokio::{fs::File, net::TcpListener, sync::mpsc};
use tokio_util::io::ReaderStream;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{error, info, warn};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "BIND_ADDR", default_value = "0.0.0.0:3002")]
    bind: String,

    #[arg(long, env = "CATALOG_PATH")]
    catalog_path: Option<PathBuf>,

    /// When set, the frontend SPA bundle at this path is served at `/`.
    #[arg(long, env = "STATIC_DIR")]
    static_dir: Option<PathBuf>,

    /// Print the OpenAPI spec as JSON to stdout and exit. Used by the
    /// frontend's type-generation step.
    #[arg(long)]
    emit_openapi: bool,
}

#[derive(Clone)]
struct AppState {
    catalog_path: PathBuf,
    version: Arc<ArcSwap<CatalogVersion>>,
}

/// Identity of the currently-served catalog file.
#[derive(Serialize, Clone, ToSchema)]
struct CatalogVersion {
    /// SHA-256 of the catalog file as hex. Stable cache key for OPFS.
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

fn hash_catalog(path: &std::path::Path) -> Result<CatalogVersion> {
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(CatalogVersion {
        hash: hex::encode(hasher.finalize()),
        bytes: bytes.len() as u64,
    })
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

    if args.emit_openapi {
        let dummy = AppState {
            catalog_path: PathBuf::new(),
            version: Arc::new(ArcSwap::from_pointee(CatalogVersion {
                hash: String::new(),
                bytes: 0,
            })),
        };
        let (_, openapi) = build_router(dummy).split_for_parts();
        println!("{}", openapi.to_pretty_json()?);
        return Ok(());
    }

    let catalog_path = args
        .catalog_path
        .clone()
        .ok_or_else(|| anyhow::anyhow!("--catalog-path is required"))?;

    let initial = hash_catalog(&catalog_path).unwrap_or_else(|e| {
        warn!(error = %e, "initial catalog hash failed, serving empty version");
        CatalogVersion {
            hash: String::new(),
            bytes: 0,
        }
    });
    info!(hash = %initial.hash, bytes = initial.bytes, "catalog version computed");

    let state = AppState {
        catalog_path: catalog_path.clone(),
        version: Arc::new(ArcSwap::from_pointee(initial)),
    };

    spawn_watcher(catalog_path, state.clone());

    let (api_router, openapi) = build_router(state).split_for_parts();

    let mut app = api_router.merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", openapi));

    if let Some(dir) = args.static_dir.as_ref() {
        let static_files = ServeDir::new(dir)
            .append_index_html_on_directories(true)
            .fallback(ServeFile::new(dir.join("index.html")));
        app = app.fallback_service(static_files);
    }

    let app = app.layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(&args.bind).await?;
    info!(
        addr = %args.bind,
        static_dir = ?args.static_dir.as_ref().map(|p| p.display().to_string()),
        "listening"
    );
    axum::serve(listener, app).await?;
    Ok(())
}

fn spawn_watcher(path: PathBuf, state: AppState) {
    use notify::{RecursiveMode, Watcher};
    use std::time::Duration;

    let (tx, mut rx) = mpsc::channel::<()>(8);

    let watch_path = path.clone();
    std::thread::spawn(move || {
        let mut watcher = match notify::recommended_watcher(move |res: notify::Result<_>| {
            if let Err(e) = res {
                warn!(error = %e, "fs watcher error");
                return;
            }
            let _ = tx.blocking_send(());
        }) {
            Ok(w) => w,
            Err(e) => {
                error!(error = %e, "failed to start fs watcher");
                return;
            }
        };
        if let Err(e) = watcher.watch(&watch_path, RecursiveMode::NonRecursive) {
            error!(error = %e, path = %watch_path.display(), "watch failed");
            return;
        }
        std::thread::park();
    });

    tokio::spawn(async move {
        while rx.recv().await.is_some() {
            tokio::time::sleep(Duration::from_millis(250)).await;
            while rx.try_recv().is_ok() {}
            match tokio::task::spawn_blocking({
                let path = path.clone();
                move || hash_catalog(&path)
            })
            .await
            {
                Ok(Ok(next)) => {
                    info!(hash = %next.hash, bytes = next.bytes, "catalog version updated");
                    state.version.store(Arc::new(next));
                }
                Ok(Err(e)) => warn!(error = %e, "rehash failed"),
                Err(e) => warn!(error = %e, "rehash task panicked"),
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
    let v = state.version.load_full();
    axum::Json((*v).clone()).into_response()
}

#[utoipa::path(
    get,
    path = "/catalog/binary",
    responses(
        (status = 200, description = "Catalog binary, gzipped (Content-Encoding: gzip)", content_type = "application/octet-stream"),
        (status = 503, description = "Catalog file unavailable")
    )
)]
async fn catalog_binary(State(state): State<AppState>) -> Response {
    use tokio::io::AsyncReadExt;

    let mut peek = match File::open(&state.catalog_path).await {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("catalog unavailable: {e}"),
            )
                .into_response();
        }
    };
    let mut head = [0u8; 2];
    let is_gzip = peek.read_exact(&mut head).await.is_ok() && head == [0x1f, 0x8b];
    drop(peek);

    let file = match File::open(&state.catalog_path).await {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("catalog unavailable: {e}"),
            )
                .into_response();
        }
    };
    let metadata = match file.metadata().await {
        Ok(m) => m,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("stat: {e}")).into_response();
        }
    };

    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    let mut response = Response::new(body);
    let h = response.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    h.insert(header::CONTENT_LENGTH, metadata.len().into());
    h.insert(header::CACHE_CONTROL, "public, max-age=60".parse().unwrap());
    if is_gzip {
        h.insert(header::CONTENT_ENCODING, "gzip".parse().unwrap());
    }
    response
}

fn build_router(state: AppState) -> OpenApiRouter {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .routes(routes!(catalog_version))
        .routes(routes!(catalog_binary))
        .with_state(state)
}
