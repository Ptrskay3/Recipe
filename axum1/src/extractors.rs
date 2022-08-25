use std::{convert::Infallible, ops::Deref};

use crate::error::ApiError;
use async_redis_session::RedisSessionStore;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts, Extension};
use sqlx::{pool, PgPool, Postgres};

pub struct DatabaseConnection(pub pool::PoolConnection<Postgres>);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<PgPool>::from_request_parts(parts, state)
            .await
            .expect("`Database` extension is missing");

        let conn = pool.acquire().await?;
        Ok(Self(conn))
    }
}

pub struct RedisConnection(pub RedisSessionStore);

#[async_trait]
impl<S> FromRequestParts<S> for RedisConnection
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(store) = Extension::<RedisSessionStore>::from_request_parts(parts, state)
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
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(session) =
            Extension::<crate::session_ext::Session>::from_request_parts(parts, state)
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
impl<S> FromRequestParts<S> for MaybeAuthUser
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(session) =
            Extension::<crate::session_ext::Session>::from_request_parts(parts, state)
                .await
                .expect("`SessionLayer` should be added");

        let user_id = session.get::<uuid::Uuid>("user_id");

        match user_id {
            Some(id) => Ok(Self(Some(AuthUser::new(id)))),
            None => Ok(Self(None)),
        }
    }
}
