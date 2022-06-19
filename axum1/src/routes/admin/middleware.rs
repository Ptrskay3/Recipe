use anyhow::Context;
use async_redis_session::RedisSessionStore;
use async_session::{async_trait, SessionStore};
use axum::{
    extract::{FromRequest, RequestParts},
    Extension,
};
use axum_extra::extract::SignedCookieJar;
use sqlx::PgPool;

use crate::{error::ApiError, extractors::AuthUser, AXUM_SESSION_COOKIE_NAME};

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AdminUser {
    name: String,
}

#[async_trait]
impl<B> FromRequest<B> for AdminUser
where
    B: Send,
{
    type Rejection = ApiError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(store) = Extension::<RedisSessionStore>::from_request(req)
            .await
            .expect("`RedisSessionStore` extension is missing");

        let cookie_jar = Option::<SignedCookieJar>::from_request(req)
            .await
            .expect("`SignedCookieJar` extension is missing");

        // TODO: this logic is already done somewhere, we should figure out
        // how to use that to acquire a connection to the db.
        let Extension(pool) = Extension::<PgPool>::from_request(req)
            .await
            .expect("`Database` extension is missing");

        let mut db = pool.acquire().await?;

        let session_cookie = cookie_jar
            .as_ref()
            .and_then(|cookie| cookie.get(AXUM_SESSION_COOKIE_NAME));

        if session_cookie.is_none() {
            return Err(ApiError::Unauthorized);
        }

        let user_id = if let Some(session) = store
            .load_session(session_cookie.unwrap().value().into())
            .await
            .context("Failed to load session for some unexpected reason")?
        {
            if let Some(user_id) = session.get::<AuthUser>("user_id") {
                user_id
            } else {
                return Err(ApiError::Unauthorized);
            }
        } else {
            return Err(ApiError::Unauthorized);
        };

        // TODO: add an indicator column for admins
        let user = sqlx::query_as!(
            Self,
            "SELECT name FROM users WHERE user_id = $1",
            sqlx::types::uuid::Uuid::from(user_id)
        )
        .fetch_optional(&mut db)
        .await
        .map_err(|_| ApiError::Unauthorized)?;

        if let Some(user) = user {
            // FIXME: This is only for testing purposes.
            if user.name == "peter" {
                return Ok(user);
            }
        }
        Err(ApiError::Unauthorized)
    }
}
