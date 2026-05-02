//! Web-app backend serving the catalog binary at `/catalog/binary` and,
//! when `--static-dir` is set, the SPA bundle at `/`.

use std::path::PathBuf;

use anyhow::Result;
use axum::{
    Router,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use clap::Parser;
use tokio::{fs::File, net::TcpListener};
use tokio_util::io::ReaderStream;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::info;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "BIND_ADDR", default_value = "0.0.0.0:3002")]
    bind: String,

    #[arg(long, env = "CATALOG_PATH")]
    catalog_path: PathBuf,

    /// When set, the frontend SPA bundle at this path is served at `/`.
    #[arg(long, env = "STATIC_DIR")]
    static_dir: Option<PathBuf>,
}

#[derive(Clone)]
struct AppState {
    catalog_path: PathBuf,
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
    let state = AppState {
        catalog_path: args.catalog_path,
    };

    let mut app = Router::new()
        .route("/health", get(health))
        .route("/catalog/binary", get(catalog_binary))
        .with_state(state);

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

async fn health() -> &'static str {
    "ok"
}

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
