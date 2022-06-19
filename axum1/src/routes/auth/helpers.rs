use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum_extra::extract::{
    cookie::{Cookie as AxumCookie, SameSite},
    SignedCookieJar,
};

use crate::{error::ApiError, AXUM_SESSION_COOKIE_NAME};

pub async fn set_authorization_headers(
    store: RedisSessionStore,
    user_id: uuid::Uuid,
    jar: SignedCookieJar,
) -> Result<SignedCookieJar, ApiError> {
    let mut session = Session::new();
    // TODO: do not hardcode this here..
    session.expire_in(std::time::Duration::from_secs(1200));
    session.insert("user_id", user_id).unwrap();
    let cookie = store.store_session(session).await?.unwrap();
    // TODO: expiry and refresh
    let cookie = AxumCookie::build(AXUM_SESSION_COOKIE_NAME, cookie)
        .path("/")
        .same_site(SameSite::Lax)
        .expires(None)
        .finish();
    Ok(jar.add(cookie))
}

pub async fn unset_authorization_headers(
    cookie_jar: SignedCookieJar,
    store: RedisSessionStore,
) -> Result<SignedCookieJar, ApiError> {
    let session_cookie = cookie_jar.get(AXUM_SESSION_COOKIE_NAME).unwrap();
    let loaded_session = store
        .load_session(session_cookie.value().to_owned())
        .await?
        .unwrap();
    store.destroy_session(loaded_session).await.unwrap();

    Ok(cookie_jar.remove(AxumCookie::named(AXUM_SESSION_COOKIE_NAME)))
}
