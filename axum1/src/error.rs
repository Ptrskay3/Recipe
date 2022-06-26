use axum::http::header::WWW_AUTHENTICATE;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use sqlx::error::DatabaseError;
use std::borrow::Cow;
use std::collections::HashMap;

/// A common error type that can be used throughout the API.
///
/// Can be returned in a `Result` from an API handler function.
///
/// For convenience, this represents both API errors as well as internal recoverable errors,
/// and maps them to appropriate status codes along with at least a minimally useful error
/// message in a plain text body, or a JSON body in the case of `UnprocessableEntity`.
#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    // Return `400 Bad Request`
    #[error("bad request")]
    BadRequest,

    /// Return `401 Unauthorized`
    #[error("authentication required")]
    Unauthorized,

    /// Return `403 Forbidden`
    #[error("user may not perform that action")]
    Forbidden,

    /// Return `404 Not Found`
    #[error("request path not found")]
    NotFound,

    /// Return `422 Unprocessable Entity`
    ///
    /// This also serializes the `errors` map to JSON.
    ///
    /// For a good API, the other status codes should also ideally map to some sort of JSON body
    /// that the frontend can deal with.
    #[error("error in the request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },

    /// Automatically return `500 Internal Server Error` on a `sqlx::Error`.
    ///
    /// Via the generated `From<sqlx::Error> for Error` impl,
    /// this allows using `?` on database calls in handler functions without a manual mapping step.
    ///
    /// I highly recommend creating an error type like this if only to make handler function code
    /// nicer.
    ///
    /// The actual error message isn't returned to the client for security reasons.
    /// It should be logged instead.
    ///
    /// Note that this could also contain database constraint errors, which should usually
    /// be transformed into client errors (e.g. `422 Unprocessable Entity` or `409 Conflict`).
    /// See `ResultExt` below for a convenient way to do this.
    #[error("an error occurred with the database")]
    Sqlx(#[from] sqlx::Error),

    /// Return `500 Internal Server Error` on a `anyhow::Error`.
    ///
    /// `anyhow::Error` is used in a few places to capture context and backtraces
    /// on unrecoverable (but technically non-fatal) errors which could be highly useful for
    /// debugging. Just use `.context()` to refine the logged error.
    ///
    /// Via the generated `From<anyhow::Error> for Error` impl, this allows the
    /// use of `?` in handler functions to automatically convert `anyhow::Error` into a response.
    ///
    /// Like with `Error::Sqlx`, the actual error message is not returned to the client
    /// for security reasons.
    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),

    #[error("an internal server error occured")]
    Reqwest(#[from] reqwest::Error),
}

impl ApiError {
    /// Convenient constructor for `Error::UnprocessableEntity`.
    ///
    /// Multiple for the same key are collected into a list for that key.
    pub fn unprocessable_entity<K, V>(errors_in: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let mut errors = HashMap::new();

        for (key, val) in errors_in {
            errors
                .entry(key.into())
                .or_insert_with(Vec::new)
                .push(val.into());
        }

        Self::UnprocessableEntity { errors }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Sqlx(_) | Self::Anyhow(_) | Self::Reqwest(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Axum allows you to return `Result` from handler functions, but the error type
/// also must be some sort of response type.
///
/// By default, the generated `Display` impl is used to return a plaintext error message
/// to the client.
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::UnprocessableEntity { errors } => {
                #[derive(serde::Serialize)]
                struct Errors {
                    errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
                }

                return (StatusCode::UNPROCESSABLE_ENTITY, Json(Errors { errors })).into_response();
            }
            Self::Unauthorized => {
                return (
                    self.status_code(),
                    // Include the `WWW-Authenticate` challenge required in the specification
                    // for the `401 Unauthorized` response code:
                    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401
                    //
                    // Let's just try to adhere to web standards wherever possible,
                    // if nothing else than to try to act as a vanguard of sanity on the web.
                    [(WWW_AUTHENTICATE, HeaderValue::from_static("Token"))]
                        .into_iter()
                        .collect::<HeaderMap>(),
                    self.to_string(),
                )
                    .into_response();
            }

            Self::Sqlx(ref e) => {
                // TODO: Probably `tracing`should be used to link to the HTTP request by `TraceLayer`.
                log::error!("SQLx error: {:?}", e);
            }

            Self::Anyhow(ref e) => {
                // TODO: Probably `tracing`should be used to link to the HTTP request by `TraceLayer`.
                log::error!("Generic error: {:?}", e);
            }
            Self::Reqwest(ref e) => {
                // TODO: Probably `tracing`should be used to link to the HTTP request by `TraceLayer`.
                log::error!("Reqwest error: {:?}", e);
            }

            // Other errors get mapped normally.
            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}

/// A little helper trait for more easily converting database constraint errors into API errors.
///
/// ```rust,ignore
/// let user_id = sqlx::query_scalar!(
///     r#"insert into "user" (username, email, password_hash) values ($1, $2, $3) returning user_id"#,
///     username,
///     email,
///     password_hash
/// )
///     .fetch_one(&ctxt.db)
///     .await
///     .on_constraint("user_username_key", |_| Error::unprocessable_entity([("username", "already taken")]))?;
/// ```
///
/// To actually make this work in a generic context would make it quite a bit more complex,
/// as you'd need an intermediate error type to represent either a mapped or an unmapped error,
/// and even then it's not clear how to handle `?` in the unmapped case without more boilerplate.
pub trait ResultExt<T> {
    /// If `self` contains a SQLx database constraint error with the given name,
    /// transform the error.
    ///
    /// Otherwise, the result is passed through unchanged.
    fn on_constraint(
        self,
        name: &str,
        f: impl FnOnce(Box<dyn DatabaseError>) -> ApiError,
    ) -> Result<T, ApiError>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<ApiError>,
{
    fn on_constraint(
        self,
        name: &str,
        map_err: impl FnOnce(Box<dyn DatabaseError>) -> ApiError,
    ) -> Result<T, ApiError> {
        self.map_err(|e| match e.into() {
            ApiError::Sqlx(sqlx::Error::Database(dbe)) if dbe.constraint() == Some(name) => {
                map_err(dbe)
            }
            e => e,
        })
    }
}
