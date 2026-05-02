//! Public-facing course catalog API. Loads the catalog binary into memory
//! at startup, exposes versioned REST routes under `/v1/`, and reloads on
//! changes to the catalog file.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use axum::{
    Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use clap::Parser;
use courses_index::{binary, index::Index};
use notify::{RecursiveMode, Watcher};
use serde::Serialize;
use tokio::{net::TcpListener, sync::mpsc};
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "BIND_ADDR", default_value = "0.0.0.0:3001")]
    bind: String,

    #[arg(long, env = "CATALOG_PATH")]
    catalog_path: PathBuf,
}

struct Catalog {
    bytes: Vec<u8>,
    index: Index,
}

#[derive(Clone)]
struct AppState {
    catalog: Arc<ArcSwap<Catalog>>,
}

fn load_catalog(path: &std::path::Path) -> Result<Catalog> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let payload = binary::read_catalog_from_slice(&bytes)?;
    let index = match payload.prebuilt_text {
        Some(p) => Index::build_with_prebuilt_text(payload.corpus, p)?,
        None => Index::build(payload.corpus),
    };
    Ok(Catalog { bytes, index })
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

    let initial = load_catalog(&args.catalog_path)
        .with_context(|| format!("initial load of {}", args.catalog_path.display()))?;
    info!(
        path = %args.catalog_path.display(),
        bytes = initial.bytes.len(),
        courses = initial.index.n_docs,
        "catalog loaded"
    );

    let state = AppState {
        catalog: Arc::new(ArcSwap::from_pointee(initial)),
    };

    spawn_watcher(args.catalog_path.clone(), state.clone());

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

fn spawn_watcher(path: PathBuf, state: AppState) {
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
                move || load_catalog(&path)
            })
            .await
            {
                Ok(Ok(next)) => {
                    info!(
                        bytes = next.bytes.len(),
                        courses = next.index.n_docs,
                        "catalog reloaded"
                    );
                    state.catalog.store(Arc::new(next));
                }
                Ok(Err(e)) => warn!(error = %e, "reload failed"),
                Err(e) => warn!(error = %e, "reload task panicked"),
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
}

async fn version(State(state): State<AppState>) -> impl IntoResponse {
    let cat = state.catalog.load();
    axum::Json(VersionInfo {
        format_version: binary::FORMAT_VERSION,
        course_count: cat.index.n_docs,
        bytes: cat.bytes.len(),
    })
}

async fn binary_handler(State(state): State<AppState>) -> Response {
    let cat = state.catalog.load_full();
    let is_gzip = cat.bytes.starts_with(&[0x1f, 0x8b]);
    let bytes: bytes::Bytes = cat.bytes.clone().into();
    let mut response = Response::new(axum::body::Body::from(bytes));
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
