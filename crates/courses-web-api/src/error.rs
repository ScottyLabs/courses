//! The API's error shape: `{ code, message }`, plus `current` on a 409 conflict.

use axum::{
    Json,
    extract::Request,
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Not signed in.
    Auth,
    /// Signed in, but not allowed: a non-CMU account, or a write from another origin.
    Forbidden,
    NotFound,
    /// A write based on a stale revision. The body's `current` holds the stored record.
    Conflict,
    TooLarge,
    /// Rate limited or over a daily write cap. See `Retry-After`.
    TooMany,
    BadRequest,
    Unavailable,
}

/// Every error response.
#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: ErrorCode,
    pub message: String,
    /// The stored record, on a conflict.
    pub current: Option<serde_json::Value>,
    /// Seconds, on `too_many`.
    pub retry_after: Option<u64>,
}

pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    pub fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            current: None,
            retry_after: None,
        }
    }

    pub fn auth() -> Self {
        Self::auth_msg("Sign in with your CMU account.")
    }

    pub fn auth_msg(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, ErrorCode::Auth, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, ErrorCode::Forbidden, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorCode::NotFound, message)
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ErrorCode::BadRequest, message)
    }

    pub fn too_large(message: impl Into<String>) -> Self {
        Self::new(StatusCode::PAYLOAD_TOO_LARGE, ErrorCode::TooLarge, message)
    }

    pub fn too_many(retry_after: u64, message: impl Into<String>) -> Self {
        Self {
            retry_after: Some(retry_after.max(1)),
            ..Self::new(StatusCode::TOO_MANY_REQUESTS, ErrorCode::TooMany, message)
        }
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            ErrorCode::Unavailable,
            message,
        )
    }

    /// A stale write. `current` is the stored record the client should merge into.
    pub fn conflict(current: impl Serialize) -> Self {
        Self {
            current: Some(serde_json::to_value(current).unwrap_or(serde_json::Value::Null)),
            ..Self::new(
                StatusCode::CONFLICT,
                ErrorCode::Conflict,
                "This changed since you loaded it; merge your change into `current` and try again.",
            )
        }
    }

    /// Logs the cause and answers with a generic 500.
    pub fn internal(error: impl std::fmt::Display) -> Self {
        tracing::error!(error = %error, "request failed");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Unavailable,
            "Something went wrong on our side.",
        )
    }
}

impl From<rusqlite::Error> for ApiError {
    fn from(e: rusqlite::Error) -> Self {
        Self::internal(format_args!("database: {e}"))
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        Self::internal(format_args!("{e:#}"))
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        Self::internal(format_args!("json: {e}"))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut body = serde_json::json!({ "code": self.code, "message": self.message });
        if let Some(current) = self.current {
            body["current"] = current;
        }
        let mut response = (self.status, Json(body)).into_response();
        let headers = response.headers_mut();
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        if let Some(seconds) = self.retry_after {
            headers.insert(header::RETRY_AFTER, HeaderValue::from(seconds));
        }
        response
    }
}

/// Turns axum's own plain-text errors under `/api/` (bad JSON, a missing route, a wrong method, an oversized body)
/// into the API's JSON error shape.
pub async fn json_errors(request: Request, next: Next) -> Response {
    let api = request.uri().path().starts_with("/api/");
    let response = next.run(request).await;
    let status = response.status();
    if !api || !(status.is_client_error() || status.is_server_error()) {
        return response;
    }
    let is_json = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/json"));
    if is_json {
        return response;
    }
    let (parts, body) = response.into_parts();
    let text = axum::body::to_bytes(body, 4096)
        .await
        .map(|b| String::from_utf8_lossy(&b).trim().to_owned())
        .unwrap_or_default();
    let message = |fallback: &str| {
        if text.is_empty() {
            fallback.to_owned()
        } else {
            text.chars().take(500).collect()
        }
    };
    let mut error = match parts.status {
        StatusCode::UNAUTHORIZED => ApiError::auth(),
        StatusCode::FORBIDDEN => ApiError::forbidden(message("Forbidden.")),
        StatusCode::NOT_FOUND => ApiError::not_found(message("No such route.")),
        StatusCode::METHOD_NOT_ALLOWED => ApiError::new(
            StatusCode::METHOD_NOT_ALLOWED,
            ErrorCode::BadRequest,
            "Method not allowed.",
        ),
        StatusCode::PAYLOAD_TOO_LARGE => {
            ApiError::too_large("Request bodies are limited to 128 KiB.")
        }
        StatusCode::TOO_MANY_REQUESTS => ApiError::too_many(60, message("Too many requests.")),
        s if s.is_server_error() => ApiError::new(
            s,
            ErrorCode::Unavailable,
            "Something went wrong on our side.",
        ),
        _ => ApiError::bad_request(message("Bad request.")),
    };
    if let Some(retry) = parts.headers.get(header::RETRY_AFTER) {
        error.retry_after = retry.to_str().ok().and_then(|s| s.parse().ok());
    }
    let mut out = error.into_response();
    for name in [header::ALLOW, header::SET_COOKIE] {
        for value in parts.headers.get_all(&name) {
            out.headers_mut().append(&name, value.clone());
        }
    }
    out
}
