use anyhow::Context;
use axum::{
    routing::{get, post, put},
    Form, Json, Router,
};
use axum_extra::extract::SignedCookieJar;
use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;

use crate::{
    error::{ApiError, ResultExt},
    extractors::{AuthUser, DatabaseConnection, MaybeAuthUser, RedisConnection},
};

mod helpers;
mod password;

use helpers::{set_authorization_headers, unset_authorization_headers};
use password::{compute_password_hash, validate_credentials};

#[must_use]
pub fn auth_router() -> Router {
    Router::new()
        .route("/me", get(me))
        .route("/auth", post(authorize))
        .route("/register", post(register))
        .route("/logout", get(logout))
        .route("/update_password", put(update_password))
}

async fn me(maybe_auth_user: MaybeAuthUser) -> Result<Json<Uuid>, ApiError> {
    if let Some(auth_user) = maybe_auth_user.into_inner() {
        return Ok(Json(*auth_user));
    }
    Err(ApiError::Unauthorized)
}

#[derive(Debug, serde::Deserialize)]
pub struct Credentials {
    name: String,
    password: Secret<String>,
}

async fn authorize(
    Form(credentials): Form<Credentials>,
    RedisConnection(store): RedisConnection,
    conn: DatabaseConnection,
    jar: SignedCookieJar,
) -> Result<SignedCookieJar, ApiError> {
    let user_id = validate_credentials(credentials, conn).await?;
    let cookie_jar = set_authorization_headers(store, user_id, jar).await?;
    Ok(cookie_jar)
}

async fn logout(
    _user: AuthUser,
    RedisConnection(store): RedisConnection,
    cookie_jar: SignedCookieJar,
) -> Result<SignedCookieJar, ApiError> {
    let cookie_jar = unset_authorization_headers(cookie_jar, store).await?;
    Ok(cookie_jar)
}

#[derive(serde::Deserialize)]
pub struct Register {
    name: String,
    email: String,
    password: Secret<String>,
}

async fn register(
    Form(form): Form<Register>,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<(), ApiError> {
    let Register {
        name,
        email,
        password,
    } = form;
    let password_hash =
        crate::utils::spawn_blocking_with_tracing(move || compute_password_hash(password))
            .await
            .map_err(|_| ApiError::Anyhow(anyhow::anyhow!("Failed to hash password")))??;

    sqlx::query!(
        r#"
        INSERT INTO users (name, email, password_hash)
        VALUES ($1, $2, $3)
        "#,
        name,
        email,
        password_hash.expose_secret(),
    )
    .execute(&mut conn)
    .await
    .on_constraint("users_name_key", |_| {
        ApiError::unprocessable_entity([("name", "name already taken")])
    })
    .on_constraint("users_email_key", |_| {
        ApiError::unprocessable_entity([("email", "email already taken")])
    })?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct UpdatePassword {
    name: String,
    password: Secret<String>,
}

async fn update_password(
    user_id: AuthUser,
    Form(form): Form<UpdatePassword>,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<(), ApiError> {
    let UpdatePassword { name, password } = form;
    let password_hash =
        crate::utils::spawn_blocking_with_tracing(move || compute_password_hash(password))
            .await
            .map_err(|_| ApiError::Anyhow(anyhow::anyhow!("Failed to hash password")))??;

    sqlx::query!(
        r#"
        UPDATE users
        SET password_hash = $1
        WHERE name = $2 AND user_id = $3
        "#,
        password_hash.expose_secret(),
        name,
        // FIXME: after the 0.6 release of sqlx, this nonsense can go away
        sqlx::types::uuid::Uuid::from(user_id),
    )
    .execute(&mut conn)
    .await
    .context("Failed to change user's password in the database.")?;
    Ok(())
}
