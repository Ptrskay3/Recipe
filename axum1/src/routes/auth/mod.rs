use anyhow::Context;
use async_session::Session;
use axum::{
    routing::{get, post, put},
    Extension, Form, Json, Router,
};
use secrecy::{ExposeSecret, Secret};

use crate::{
    error::{ApiError, ResultExt},
    extractors::{AuthUser, DatabaseConnection, MaybeAuthUser},
};

mod password;

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

#[derive(sqlx::FromRow, serde::Serialize, Debug)]
struct UserDetails {
    name: String,
}

async fn me(
    maybe_auth_user: MaybeAuthUser,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Option<UserDetails>>, ApiError> {
    if let Some(auth_user) = maybe_auth_user.into_inner() {
        let name = sqlx::query_as!(
            UserDetails,
            r#"SELECT name FROM users WHERE user_id = $1"#,
            *auth_user
        )
        .fetch_one(&mut conn)
        .await?;
        return Ok(Json(Some(name)));
    }
    Ok(Json(None))
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Credentials {
    name: String,
    password: Secret<String>,
}

async fn authorize(
    Form(credentials): Form<Credentials>,
    Extension(mut session): Extension<Session>,
    conn: DatabaseConnection,
) -> Result<(), ApiError> {
    let user_id = validate_credentials(credentials, conn).await?;
    // Rotate the session cookie on privilege level change.
    // This is to prevent session-fixation attacks.
    session.regenerate();
    session
        .insert("user_id", user_id)
        .expect("user_id is serializable");
    Ok(())
}

async fn logout(
    _user: AuthUser,
    Extension(mut session): Extension<Session>,
) -> Result<(), ApiError> {
    session.destroy();
    Ok(())
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
        *user_id,
    )
    .execute(&mut conn)
    .await
    .context("Failed to change user's password in the database.")?;
    Ok(())
}
