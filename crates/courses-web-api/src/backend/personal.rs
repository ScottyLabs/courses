//! Personal endpoint skeletons. See sites/docs/src/backend-development.md.
use super::*;

/// TODO: implement `schedules` against the retained database schema.
pub async fn schedules(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `create_schedule` against the retained database schema.
pub async fn create_schedule(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Json(_v): Json<Value>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `save_schedule` against the retained database schema.
pub async fn save_schedule(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Path(_id): Path<String>,
    Json(_v): Json<Value>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `delete_schedule` against the retained database schema.
pub async fn delete_schedule(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Path(_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    Err(not_implemented())
}

/// TODO: implement `share` against the retained database schema.
pub async fn share(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Path(_id): Path<String>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `revoke_share` against the retained database schema.
pub async fn revoke_share(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Path(_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    Err(not_implemented())
}

/// TODO: implement `shared` against the retained database schema.
pub async fn shared(Extension(_b): Extension<Backend>, Path(_id): Path<String>) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `saved` against the retained database schema.
pub async fn saved(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Query(_q): Query<HashMap<String, String>>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `save_course` against the retained database schema.
pub async fn save_course(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Path(_code): Path<String>,
    Json(_v): Json<Value>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `unsave_course` against the retained database schema.
pub async fn unsave_course(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Path(_code): Path<String>,
    Query(_q): Query<HashMap<String, String>>,
) -> Result<StatusCode, ApiError> {
    Err(not_implemented())
}
