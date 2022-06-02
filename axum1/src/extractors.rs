use axum::{
    async_trait,
    TypedHeader,
    extract::{FromRequest, RequestParts},
    headers::Cookie,
    http::{StatusCode, HeaderValue},
    Extension,
};
use async_session::{MemoryStore, Session, SessionStore as _};
use sqlx::PgPool;
use crate::AXUM_SESSION_COOKIE_NAME;
pub struct DatabaseConnection(pub sqlx::pool::PoolConnection<sqlx::Postgres>);

#[async_trait]
impl<B> FromRequest<B> for DatabaseConnection
where
    B: Send,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<PgPool>::from_request(req)
            .await
            .map_err(internal_error)?;

        let conn = pool.acquire().await.map_err(internal_error)?;
        Ok(Self(conn))
    }
}

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}


#[derive(Debug)]
pub struct FreshUserId {
    pub user_id: UserId,
    pub cookie: HeaderValue,
}

#[derive(Debug)]
pub enum UserIdFromSession {
    FoundUserId(UserId),
    CreatedFreshUserId(FreshUserId),
}

#[async_trait]
impl<B> FromRequest<B> for UserIdFromSession
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(store) = Extension::<MemoryStore>::from_request(req)
            .await
            .expect("`MemoryStore` extension missing");

        let cookie = Option::<TypedHeader<Cookie>>::from_request(req)
            .await
            .unwrap();

        let session_cookie = cookie
            .as_ref()
            .and_then(|cookie| cookie.get(AXUM_SESSION_COOKIE_NAME));

        // return the new created session cookie for client
        if session_cookie.is_none() {
            let user_id = UserId::new();
            let mut session = Session::new();
            session.insert("user_id", user_id).unwrap();
            let cookie = store.store_session(session).await.unwrap().unwrap();
            return Ok(Self::CreatedFreshUserId(FreshUserId {
                user_id,
                cookie: HeaderValue::from_str(
                    format!("{}={}", AXUM_SESSION_COOKIE_NAME, cookie).as_str(),
                )
                .unwrap(),
            }));
        }

        tracing::debug!(
            "UserIdFromSession: got session cookie from user agent, {}={}",
            AXUM_SESSION_COOKIE_NAME,
            session_cookie.unwrap()
        );
        // continue to decode the session cookie
        let user_id = if let Some(session) = store
            .load_session(session_cookie.unwrap().to_owned())
            .await
            .unwrap()
        {
            if let Some(user_id) = session.get::<UserId>("user_id") {
                tracing::debug!(
                    "UserIdFromSession: session decoded success, user_id={:?}",
                    user_id
                );
                user_id
            } else {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "No `user_id` found in session",
                ));
            }
        } else {
            tracing::debug!(
                "UserIdFromSession: err session not exists in store, {}={}",
                AXUM_SESSION_COOKIE_NAME,
                session_cookie.unwrap()
            );
            return Err((StatusCode::BAD_REQUEST, "No session found for cookie"));
        };

        Ok(Self::FoundUserId(user_id))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct UserId(uuid::Uuid);

impl UserId {
    fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
