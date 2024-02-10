//! Cookie based session middleware for axum applications.
//!
//! [`SessionLayer`] provides client sessions via `async_session`.
//!  Sessions are backed by cryptographically signed cookies. These cookies
//! are generated when they’re not found or otherwise invalid. When a valid,
//! known cookie is received in a request, the session is hydrated from this
//! cookie. The middleware leverages `http::Extensions` to attach an
//! `async_session::Session` to the request. Request handlers can then
//! interact with the session:
//!
//! ```rust
//! use async_session::Session;
//! use axum::{Extension , http::StatusCode, response::IntoResponse};
//!
//! async fn handler(Extension(session): Extension<Session>) -> impl IntoResponse {
//!     match session.get::<String>("key") {
//!         Some(value) => Ok(value),
//!         _ => Err(StatusCode::NOT_FOUND),
//!     }
//! }
//! ```

use std::{
    task::{Context, Poll},
    time::Duration,
};

use async_session::{
    base64,
    hmac::{Hmac, Mac, NewMac},
    sha2::Sha256,
    SessionStore,
};
use axum::{
    body::Body,
    http::{
        header::{HeaderValue, COOKIE, SET_COOKIE},
        Request, StatusCode,
    },
    response::Response,
};
use axum_extra::extract::cookie::{Cookie, Key, SameSite};
use futures::future::BoxFuture;
use tower::{Layer, Service};

const BASE64_DIGEST_LEN: usize = 44;

/// Controls how the session data is persisted and created.
#[derive(Clone)]
pub enum Persistence {
    /// Store empty "guest" sessions and always reach for the storage layer.
    Always,
    /// Do not store empty "guest" sessions, only ping the storage layer if
    /// the session data changed.
    ChangedOnly,
    /// Do not store empty "guest" sessions, always ping the storage layer for existing sessions.
    ExistingOnly,
}

#[derive(Clone)]
pub struct SessionLayer<Store> {
    store: Store,
    cookie_path: String,
    cookie_name: String,
    cookie_domain: Option<String>,
    session_ttl: Option<Duration>,
    persistence: Persistence,
    same_site_policy: SameSite,
    secure: Option<bool>,
    key: Key,
}

impl<Store: SessionStore> SessionLayer<Store> {
    /// Creates a layer which will attach an [`async_session::Session`] to
    /// requests via an extension. This session is derived from a
    /// cryptographically signed cookie. When the client sends a valid,
    /// known cookie then the session is hydrated from this. Otherwise a new
    /// cookie is created and returned in the response.
    ///
    /// The default behaviour is to enable "guest" sessions with [`Persistence::Always`].
    ///
    /// # Panics
    ///
    /// `SessionLayer::new` will panic if the secret is less than 64 bytes.
    pub fn new(store: Store, secret: &[u8]) -> Self {
        Self {
            store,
            persistence: Persistence::Always,
            cookie_path: "/".into(),
            cookie_name: "axum_sid".into(),
            cookie_domain: None,
            same_site_policy: SameSite::Strict,
            session_ttl: Some(Duration::from_secs(24 * 60 * 60)),
            secure: None,
            key: Key::from(secret),
        }
    }

    /// Sets how the persistence and guest sessions are handled. Defaults to [`Persistence::Always`].
    pub fn with_persistence(mut self, persistence: Persistence) -> Self {
        self.persistence = persistence;
        self
    }

    /// Sets a cookie for the session. Defaults to `"/"`.
    pub fn with_cookie_path(mut self, cookie_path: impl AsRef<str>) -> Self {
        self.cookie_path = cookie_path.as_ref().to_owned();
        self
    }

    /// Sets a cookie name for the session. Defaults to `"axum_sid"`.
    pub fn with_cookie_name(mut self, cookie_name: impl AsRef<str>) -> Self {
        self.cookie_name = cookie_name.as_ref().to_owned();
        self
    }

    /// Sets a cookie domain for the session. Defaults to `None`.
    pub fn with_cookie_domain(mut self, cookie_domain: impl AsRef<str>) -> Self {
        self.cookie_domain = Some(cookie_domain.as_ref().to_owned());
        self
    }

    /// Sets a cookie same site policy for the session. Defaults to
    /// `SameSite::Strict`.
    pub fn with_same_site_policy(mut self, policy: SameSite) -> Self {
        self.same_site_policy = policy;
        self
    }

    /// Sets a cookie time-to-live (ttl) for the session. Defaults to
    /// `Duration::from_secs(60 * 60 24)`; one day.
    pub fn with_session_ttl(mut self, session_ttl: Option<Duration>) -> Self {
        self.session_ttl = session_ttl;
        self
    }

    /// Sets a cookie secure attribute for the session. Defaults to `false`.
    pub fn with_secure(mut self, secure: bool) -> Self {
        self.secure = Some(secure);
        self
    }

    fn should_store(&self, cookie_value: &Option<String>) -> bool {
        matches!(self.persistence, Persistence::Always)
            || (matches!(self.persistence, Persistence::ExistingOnly) && cookie_value.is_some())
    }

    async fn load_or_create(&self, cookie_value: Option<String>) -> crate::session_ext::Session {
        let session = match cookie_value {
            Some(cookie_value) => self.store.load_session(cookie_value).await.ok().flatten(),
            None => None,
        };

        let inner = session
            .and_then(|session| session.validate())
            .unwrap_or_default();

        crate::session_ext::Session::from_inner(inner)
    }

    fn build_cookie(&self, secure: bool, cookie_value: String) -> Cookie<'static> {
        let mut cookie = Cookie::build((self.cookie_name.clone(), cookie_value))
            .http_only(true)
            .same_site(self.same_site_policy)
            .secure(secure)
            .path(self.cookie_path.clone());

        if let Some(ttl) = self.session_ttl {
            cookie = cookie.expires(Some((std::time::SystemTime::now() + ttl).into()));
        }

        if let Some(cookie_domain) = self.cookie_domain.clone() {
            cookie = cookie.domain(cookie_domain);
        }

        let mut cookie = cookie.build();

        self.sign_cookie(&mut cookie);

        cookie
    }

    fn build_removal_cookie(&self, secure: bool) -> Cookie<'static> {
        let mut cookie = Cookie::build((self.cookie_name.clone(), ""))
            .http_only(true)
            .same_site(self.same_site_policy)
            .secure(secure)
            .build();

        cookie.make_removal();

        self.sign_cookie(&mut cookie);

        cookie
    }

    // This is mostly based on:
    // https://github.com/SergioBenitez/cookie-rs/blob/master/src/secure/signed.rs#L33-L43
    /// Signs the cookie's value providing integrity and authenticity.
    fn sign_cookie(&self, cookie: &mut Cookie<'_>) {
        // Compute HMAC-SHA256 of the cookie's value.
        let mut mac = Hmac::<Sha256>::new_from_slice(self.key.signing()).expect("a good key");
        mac.update(cookie.value().as_bytes());

        // Cookie's new value is [MAC | original-value].
        let mut new_value = base64::encode(mac.finalize().into_bytes());
        new_value.push_str(cookie.value());
        cookie.set_value(new_value);
    }

    // This is mostly based on:
    // https://github.com/SergioBenitez/cookie-rs/blob/master/src/secure/signed.rs#L45-L63
    /// Given a signed value `str` where the signature is prepended to `value`,
    /// verifies the signed value and returns it. If there's a problem, returns
    /// an `Err` with a string describing the issue.
    fn verify_signature(&self, cookie_value: &str) -> Result<String, &'static str> {
        if cookie_value.len() < BASE64_DIGEST_LEN {
            return Err("length of value is <= BASE64_DIGEST_LEN");
        }

        // Split [MAC | original-value] into its two parts.
        let (digest_str, value) = cookie_value.split_at(BASE64_DIGEST_LEN);
        let digest = base64::decode(digest_str).map_err(|_| "bad base64 digest")?;

        // Perform the verification.
        let mut mac = Hmac::<Sha256>::new_from_slice(self.key.signing()).expect("a good key");
        mac.update(value.as_bytes());
        mac.verify(&digest)
            .map(|_| value.to_string())
            .map_err(|_| "value did not verify")
    }

    /// Returns the cookie value corresponding to `self.cookie_name`.
    ///
    /// Multiple cookies may be all concatenated into a single Cookie header
    /// separated with semicolons (HTTP/1.1 behaviour) or into multiple separate
    /// Cookie headers (HTTP/2 behaviour, see [RFC 7540](https://www.rfc-editor.org/rfc/rfc7540#section-8.1.2.5)).
    /// Search for the session cookie from all Cookie headers, assuming both forms are possible.
    pub(crate) fn cookie_value(&self, request: &Request<Body>) -> Option<String> {
        request
            .headers()
            .get_all(COOKIE)
            .iter()
            .filter_map(|cookie_header| cookie_header.to_str().ok())
            .flat_map(|cookie_header| cookie_header.split(';'))
            .filter_map(|cookie_header| Cookie::parse_encoded(cookie_header.trim()).ok())
            .filter(|cookie| cookie.name() == self.cookie_name)
            .find_map(|cookie| self.verify_signature(cookie.value()).ok())
    }
}

impl<S, Store: SessionStore> Layer<S> for SessionLayer<Store> {
    type Service = Session<S, Store>;

    fn layer(&self, inner: S) -> Self::Service {
        Session {
            inner,
            layer: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Session<S, Store: SessionStore> {
    inner: S,
    layer: SessionLayer<Store>,
}

impl<S, Store: SessionStore> Service<Request<Body>> for Session<S, Store>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        let session_layer = self.layer.clone();

        let cookie_value = self.layer.cookie_value(&request);

        let secure = self
            .layer
            .secure
            .unwrap_or_else(|| request.uri().scheme_str() == Some("https"));

        let not_ready_service = self.inner.clone();
        let mut ready_service = std::mem::replace(&mut self.inner, not_ready_service);

        Box::pin(async move {
            let mut session = session_layer.load_or_create(cookie_value.clone()).await;

            if let Some(ttl) = session_layer.session_ttl {
                session.expire_in(ttl);
            }

            request.extensions_mut().insert(session.clone());

            let mut response: Response = ready_service.call(request).await?;
            if session.is_destroyed() {
                if let Err(e) = session_layer
                    .store
                    .destroy_session(session.into_inner())
                    .await
                {
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    tracing::error!("Failed to destroy session: {:?}", e);
                }

                let removal_cookie = session_layer.build_removal_cookie(secure);

                response.headers_mut().insert(
                    SET_COOKIE,
                    HeaderValue::from_str(&removal_cookie.to_string()).unwrap(),
                );

            // Store if
            //  - We have guest sessions
            //  - We received a valid cookie and we use the `ExistingOnly` policy.
            //  - If we use the `ChangedOnly` policy, only `session.data_changed()` should trigger this branch.
            } else if session_layer.should_store(&cookie_value) || session.data_changed() {
                if session.should_regenerate() {
                    if let Err(e) = session_layer
                        .store
                        .destroy_session(session.clone().into_inner())
                        .await
                    {
                        tracing::error!("Failed to destroy old session on regenerate: {:?}", e);
                    }
                    session.inner_regenerate();
                }
                match session_layer
                    .store
                    .store_session(session.into_inner())
                    .await
                {
                    Ok(Some(cookie_value)) => {
                        let cookie = session_layer.build_cookie(secure, cookie_value);
                        response.headers_mut().insert(
                            SET_COOKIE,
                            HeaderValue::from_str(&cookie.to_string()).unwrap(),
                        );
                    }
                    Ok(None) => {}
                    Err(e) => {
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        tracing::error!("Failed to reach session storage {:?}", e);
                    }
                }
            }

            Ok(response)
        })
    }
}
