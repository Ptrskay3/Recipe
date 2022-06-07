use std::ops::Deref;

use crate::{AXUM_SESSION_COOKIE_NAME, error::ApiError};
use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    headers::Cookie,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Extension, TypedHeader,
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
            .await.expect("`Database` extension is missing");

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
            .expect("`RedisSessionStore` extension missing");

        Ok(Self(store))
    }
}

#[async_trait]
impl<B> FromRequest<B> for AuthUser
where
    B: Send,
{
    type Rejection = AuthRedirect;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(store) = Extension::<RedisSessionStore>::from_request(req)
            .await
            .expect("`RedisSessionStore` extension missing");

        let cookie = Option::<TypedHeader<Cookie>>::from_request(req)
            .await
            .unwrap();

        let session_cookie = cookie
            .as_ref()
            .and_then(|cookie| cookie.get(AXUM_SESSION_COOKIE_NAME));

        tracing::debug!(
            "got session cookie from user agent, {}={:?}",
            AXUM_SESSION_COOKIE_NAME,
            session_cookie
        );

        if session_cookie.is_none() {
            return Err(AuthRedirect);
        }

        let user_id = if let Some(session) = store
            .load_session(session_cookie.unwrap().into())
            .await
            .map_err(|_| AuthRedirect)?
        {
            if let Some(user_id) = session.get::<AuthUser>("user_id") {
                tracing::debug!("session decoded success, user_id={:?}", user_id);
                // TODO: Rotate cookie value to prevent session fixation attacks
                // https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html#renew-the-session-id-after-any-privilege-level-change
                //
                // session.regenerate();
                user_id
            } else {
                tracing::debug!("no `user_id` found in session");
                return Err(AuthRedirect);
            }
        } else {
            tracing::debug!("invalid `session_cookie`");
            return Err(AuthRedirect);
        };

        Ok(user_id)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct AuthUser(uuid::Uuid);

impl Deref for AuthUser {
    type Target = uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AuthUser {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

pub struct AuthRedirect;

impl IntoResponse for AuthRedirect {
    fn into_response(self) -> Response {
        Redirect::temporary("/auth").into_response()
    }
}
