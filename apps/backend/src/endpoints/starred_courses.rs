use crate::hello_world;
use crate::state::AppState;
use axum::{Router, routing::get};

pub fn router() -> Router<AppState> {
    Router::new().route("/starred-courses", get(hello_world))
}
