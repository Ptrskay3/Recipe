use std::ops::Deref;

use crate::error::ApiError;
use async_redis_session::RedisSessionStore;
use async_session::Session;
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
    Extension,
};
use sqlx::{pool, PgPool, Postgres};

pub struct DatabaseConnection(pub pool::PoolConnection<Postgres>);

#[async_trait]
impl<B> FromRequest<B> for DatabaseConnection
where
    B: Send,
{
    type Rejection = ApiError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<PgPool>::from_request(req)
            .await
            .expect("`Database` extension is missing");

        let conn = pool.acquire().await?;
        Ok(Self(conn))
    }
}

pub struct RedisConnection(pub RedisSessionStore);

#[async_trait]
impl<B> FromRequest<B> for RedisConnection
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(store) = Extension::<RedisSessionStore>::from_request(req)
            .await
            .expect("`RedisSessionStore` extension is missing");

        Ok(Self(store))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct AuthUser(uuid::Uuid);

impl AuthUser {
    fn new(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }
}

impl Deref for AuthUser {
    type Target = uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<B> FromRequest<B> for AuthUser
where
    B: Send,
{
    type Rejection = ApiError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(session) = Extension::<Session>::from_request(req)
            .await
            .expect("`SessionLayer` should be added");

        session
            .get::<uuid::Uuid>("user_id")
            .map(Self::new)
            .ok_or(ApiError::Unauthorized)
    }
}

pub struct MaybeAuthUser(pub Option<AuthUser>);

impl MaybeAuthUser {
    pub fn into_inner(self) -> Option<AuthUser> {
        self.0
    }
}

#[async_trait]
impl<B> FromRequest<B> for MaybeAuthUser
where
    B: Send,
{
    type Rejection = ApiError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(session) = Extension::<Session>::from_request(req)
            .await
            .expect("`SessionLayer` should be added");

        let user_id = session.get::<uuid::Uuid>("user_id");

        match user_id {
            Some(id) => Ok(Self(Some(AuthUser::new(id)))),
            None => Ok(Self(None)),
        }
    }
}

// FIXME: This is really dumb, probably this will be just straight up deleted.

// #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
// pub struct CreatorOrAdmin;

// #[async_trait]
// impl<B> FromRequest<B> for CreatorOrAdmin
// where
//     B: Send,
// {
//     type Rejection = ApiError;

//     async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
//         let Extension(session) = Extension::<Session>::from_request(req)
//             .await
//             .expect("`SessionLayer` should be added");

//         let user_id = session
//             .get::<uuid::Uuid>("user_id")
//             .ok_or(ApiError::Unauthorized)?;

//         let Path((_, id)) = Path::<(String, uuid::Uuid)>::from_request(req)
//             .await
//             .context("something failed I don't know yet")?;

//         let Extension(pool) = Extension::<PgPool>::from_request(req)
//             .await
//             .expect("`Database` extension is missing");

//         let mut db = pool.acquire().await?;

//         let admin_user = sqlx::query!("SELECT is_admin FROM users WHERE user_id = $1", user_id)
//             .fetch_optional(&mut db)
//             .await?
//             .ok_or(ApiError::Unauthorized)?
//             .is_admin;

//         if admin_user {
//             return Ok(Self);
//         }

//         let exists = sqlx::query!(
//             r#"
//             SELECT 1 as e FROM ingredient_suggestions
//             WHERE user_id = $1 AND id = $2;
//             "#,
//             user_id,
//             Some(id)
//         )
//         .fetch_optional(&mut db)
//         .await?
//         .ok_or(ApiError::Unauthorized)?
//         .e == Some(1);
//         if exists {
//             return Ok(Self);
//         }
//         Err(ApiError::Unauthorized)
//     }
// }
