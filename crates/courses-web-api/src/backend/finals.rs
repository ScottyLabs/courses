//! Finals endpoint skeletons. See sites/docs/src/backend-development.md.
use super::*;
use axum::extract::Multipart;

/// TODO: implement `finals` against the retained database schema.
pub async fn finals(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Query(_q): Query<HashMap<String, String>>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `import_finals` against the retained database schema.
pub async fn import_finals(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Query(_q): Query<HashMap<String, String>>,
    _multipart: Multipart,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `clear_finals` against the retained database schema.
pub async fn clear_finals(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Query(_q): Query<HashMap<String, String>>,
) -> Result<StatusCode, ApiError> {
    Err(not_implemented())
}
