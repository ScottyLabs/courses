//! Friends endpoint skeletons. See sites/docs/src/backend-development.md.
use super::*;

/// TODO: implement `friends` against the retained database schema.
pub async fn friends(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `request_friend` against the retained database schema.
pub async fn request_friend(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Json(_v): Json<Value>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `accept_friend` against the retained database schema.
pub async fn accept_friend(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Path(_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    Err(not_implemented())
}

/// TODO: implement `remove_friend` against the retained database schema.
pub async fn remove_friend(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Path(_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    Err(not_implemented())
}

/// TODO: implement `invite` against the retained database schema.
pub async fn invite(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
) -> ApiResult {
    Err(not_implemented())
}

/// TODO: implement `revoke_invite` against the retained database schema.
pub async fn revoke_invite(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Path(_token): Path<String>,
) -> Result<StatusCode, ApiError> {
    Err(not_implemented())
}

/// TODO: implement `redeem_invite` against the retained database schema.
pub async fn redeem_invite(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Path(_token): Path<String>,
) -> Result<StatusCode, ApiError> {
    Err(not_implemented())
}

/// TODO: implement `friend_visibility` against the retained database schema.
pub async fn friend_visibility(
    _user: Result<CurrentUser, StatusCode>,
    Extension(_b): Extension<Backend>,
    Json(_v): Json<Value>,
) -> ApiResult {
    Err(not_implemented())
}
