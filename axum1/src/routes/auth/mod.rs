use axum::{routing::get, Json, Router};
use uuid::Uuid;

use crate::{error::ApiError, extractors::MaybeAuthUser};

pub fn auth_router() -> Router {
    Router::new().route("/me", get(me))
}

async fn me(maybe_auth_user: MaybeAuthUser) -> Result<Json<Uuid>, ApiError> {
    if let Some(auth_user) = maybe_auth_user.into_inner() {
        return Ok(Json(*auth_user));
    }
    Err(ApiError::Unauthorized)
}
