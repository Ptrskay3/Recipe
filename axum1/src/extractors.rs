use std::ops::Deref;

use crate::{error::ApiError, AXUM_SESSION_COOKIE_NAME};
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

#[async_trait]
impl<B> FromRequest<B> for AuthUser
where
    B: Send,
{
    type Rejection = AuthRedirect;

    // TODO: move cookie and session creation into here
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(store) = Extension::<RedisSessionStore>::from_request(req)
            .await
            .expect("`RedisSessionStore` extension is missing");

        let cookie = Option::<TypedHeader<Cookie>>::from_request(req)
            .await
            .expect("`TypedHeader` should not be missing");

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

        let user_id = if let Some(mut session) = store
            .load_session(session_cookie.unwrap().into())
            .await
            .map_err(|_| AuthRedirect)?
        {
            if let Some(user_id) = session.get::<AuthUser>("user_id") {
                tracing::debug!("session decoded success, user_id={:?}", user_id);
                // TODO: make this a global const
                session.set_expiry(chrono::Utc::now() + chrono::Duration::minutes(20));
                store.store_session(session).await.unwrap();

                // TODO: Rotate cookie value to prevent session fixation attacks
                // This feature will become essential, as long as we initialize sessions for guest users.
                // https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html#renew-the-session-id-after-any-privilege-level-change
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

// FIXME: after the 0.6 release of sqlx, this nonsense can go away
impl From<AuthUser> for sqlx::types::uuid::Uuid {
    fn from(auth_user: AuthUser) -> Self {
        sqlx::types::uuid::Uuid::from_bytes(*auth_user.as_bytes())
    }
}

pub struct AuthRedirect;

impl IntoResponse for AuthRedirect {
    fn into_response(self) -> Response {
        Redirect::temporary("/auth").into_response()
    }
}
