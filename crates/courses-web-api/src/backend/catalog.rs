//! Catalog endpoint skeletons. See sites/docs/src/backend-development.md.
use super::*;

/// TODO: implement `terms` against the retained database schema.
pub async fn terms(Extension(_b): Extension<Backend>) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `geneds` against the retained database schema.
pub async fn geneds(Extension(_b): Extension<Backend>) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `courses` against the retained database schema.
pub async fn courses(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Query(_q): Query<HashMap<String, String>>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `course` against the retained database schema.
pub async fn course(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Path(_code): Path<String>,
    Query(_q): Query<HashMap<String, String>>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `instructors` against the retained database schema.
pub async fn instructors(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Query(_q): Query<HashMap<String, String>>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `instructor` against the retained database schema.
pub async fn instructor(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Path(_id): Path<String>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `programs` against the retained database schema.
pub async fn programs(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Query(_q): Query<HashMap<String, String>>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `program` against the retained database schema.
pub async fn program(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Path(_id): Path<String>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `version` against the retained database schema.
pub async fn version(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `binary` against the retained database schema.
pub async fn binary(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
) -> Result<Response, ApiError> {
    Err(not_implemented())
}
