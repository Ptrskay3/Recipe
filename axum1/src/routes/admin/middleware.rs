use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, state::AppState};
use tower_sessions::Session;

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct AdminUser {
    name: String,
    is_admin: bool,
}

#[async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .expect("`SessionLayer` should be added");

        let AppState { db_pool, .. } = AppState::from_ref(state);

        let mut db = db_pool.acquire().await?;

        let user_id = session
            .get::<uuid::Uuid>("user_id")
            .await?
            .ok_or(ApiError::Unauthorized)?;

        let user = sqlx::query_as!(
            Self,
            "SELECT name, is_admin FROM users WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&mut *db)
        .await?
        .ok_or(ApiError::Unauthorized)?;

        if user.is_admin {
            Ok(user)
        } else {
            Err(ApiError::Forbidden)
        }
    }
}
