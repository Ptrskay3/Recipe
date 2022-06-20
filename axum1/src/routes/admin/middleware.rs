use async_session::{async_trait, Session};
use axum::{
    extract::{FromRequest, RequestParts},
    Extension,
};
use sqlx::PgPool;

use crate::{error::ApiError, extractors::AuthUser};

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AdminUser {
    name: String,
    is_admin: Option<bool>,
}

#[async_trait]
impl<B> FromRequest<B> for AdminUser
where
    B: Send,
{
    type Rejection = ApiError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(session) = Extension::<Session>::from_request(req)
            .await
            .expect("`SessionLayer` should be added");

        let Extension(pool) = Extension::<PgPool>::from_request(req)
            .await
            .expect("`Database` extension is missing");

        let mut db = pool.acquire().await?;

        let user_id = session
            .get::<AuthUser>("user_id")
            .ok_or(ApiError::Unauthorized)?;

        let user = sqlx::query_as!(
            Self,
            "SELECT name, is_admin FROM users WHERE user_id = $1",
            sqlx::types::uuid::Uuid::from(user_id)
        )
        .fetch_optional(&mut db)
        .await
        .map_err(|_| ApiError::Unauthorized)?;

        if let Some(user) = user {
            if user.is_admin.unwrap_or(false) {
                return Ok(user);
            } else {
                return Err(ApiError::Forbidden);
            }
        }
        Err(ApiError::Unauthorized)
    }
}
