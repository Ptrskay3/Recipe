use async_session::async_trait;
use axum::{extract::FromRequestParts, http::request::Parts, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::ApiError;

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct AdminUser {
    name: String,
    is_admin: bool,
}

#[async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(session) =
            Extension::<crate::session_ext::Session>::from_request_parts(parts, state)
                .await
                .expect("`SessionLayer` should be added");

        let Extension(pool) = Extension::<PgPool>::from_request_parts(parts, state)
            .await
            .expect("`Database` extension is missing");

        let mut db = pool.acquire().await?;

        let user_id = session
            .get::<uuid::Uuid>("user_id")
            .ok_or(ApiError::Unauthorized)?;

        let user = sqlx::query_as!(
            Self,
            "SELECT name, is_admin FROM users WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&mut db)
        .await?
        .ok_or(ApiError::Unauthorized)?;

        if user.is_admin {
            Ok(user)
        } else {
            Err(ApiError::Forbidden)
        }
    }
}
