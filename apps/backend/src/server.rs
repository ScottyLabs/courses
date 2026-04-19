use crate::{
    endpoints::{friends, schedules, starred_courses},
    state::AppState,
};
use axum::{Router, routing::get};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .merge(friends::router())
        .merge(schedules::router())
        .merge(starred_courses::router())
        .with_state(state)
}

async fn root() -> &'static str {
    "ScottyLabs CMUCourses backend"
}
