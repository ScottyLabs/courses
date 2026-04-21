pub mod config;
pub mod db;
pub mod endpoints;
pub mod error;
pub mod middleware;
pub mod server;
pub mod state;

use axum::Json;
use serde_json::json;
use tokio::net::TcpListener;

pub async fn hello_world() -> Json<serde_json::Value> {
    Json(json!({ "message": "Hello, world!" }))
}

pub async fn run() -> anyhow::Result<()> {
    let config = config::Config::from_env()?;
    let database = db::connection::connect(&config).await?;
    let state = state::AppState::new(config, database);

    let host = state.config.get_host();
    let listener = TcpListener::bind(host.clone()).await?;
    let app = server::build_router(state);
    println!("CMUCourses backend running at http://{}/", host);
    axum::serve(listener, app).await?;

    Ok(())
}
