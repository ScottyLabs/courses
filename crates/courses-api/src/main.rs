//! Legacy public API skeleton; the browser uses courses-web-api instead.
use anyhow::Result;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde_json::json;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

#[derive(OpenApi)]
#[openapi(info(title = "Legacy Courses API skeleton", version = "0.1.0"))]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".into());
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .routes(routes!(version))
        .routes(routes!(binary_handler))
        .routes(routes!(course_by_code))
        .split_for_parts();
    let app = router
        .merge(Scalar::with_url("/api/scalar", api.clone()))
        .route(
            "/api/openapi.json",
            get(move || async move { axum::Json(api) }),
        )
        .layer(TraceLayer::new_for_http());
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn not_implemented() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        axum::Json(json!({"message": "Backend endpoint not implemented"})),
    )
        .into_response()
}

#[utoipa::path(get, path = "/api/health", tag = "health", responses((status = OK, body = str)))]
async fn health() -> &'static str {
    "ok"
}

#[utoipa::path(get, path = "/api/version", responses((status = 501, description = "Not implemented")))]
async fn version() -> Response {
    not_implemented()
}

#[utoipa::path(get, path = "/api/binary", responses((status = 501, description = "Not implemented")))]
async fn binary_handler() -> Response {
    not_implemented()
}

#[utoipa::path(get, path = "/api/courses/{code}", params(("code" = String, Path)), responses((status = 501, description = "Not implemented")))]
async fn course_by_code(axum::extract::Path(_code): axum::extract::Path<String>) -> Response {
    not_implemented()
}
