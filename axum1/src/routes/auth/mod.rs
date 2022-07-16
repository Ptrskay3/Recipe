use anyhow::Context;
use async_session::Session;
use axum::{
    extract::Query,
    routing::{get, post, put},
    Extension, Form, Json, Router,
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, CsrfToken, Scope,
    TokenResponse,
};
use secrecy::{ExposeSecret, Secret};
use sqlx::Acquire;

use crate::{
    error::{ApiError, ResultExt},
    extractors::{AuthUser, DatabaseConnection, MaybeAuthUser},
    queue::email::{Email, EmailClient},
};

mod confirm;
mod password;

use password::{compute_password_hash, validate_credentials};

use self::confirm::{confirm, enqueue_delivery_task, generate_confirmation_token, store_token};

#[must_use]
pub fn auth_router() -> Router {
    Router::new()
        .route("/me", get(me))
        .route("/auth", post(authorize))
        .route("/register", post(register))
        .route("/logout", get(logout))
        .route("/update_password", put(update_password))
        .route("/confirm", get(confirm))
        .route("/forget_password_gen", post(forget_password_gen))
        .route("/forget_password", post(forget_password))
        .route("/auth/discord_authorize", get(discord_authorize))
        .route("/auth/discord", get(discord_auth))
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
        .await
        .map_err(|_| ApiError::NotFound)?;
        return Ok(Json(Some(name)));
    }
    Ok(Json(None))
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Credentials {
    email: String,
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

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize, Clone)]
struct UserId {
    user_id: uuid::Uuid,
}

#[derive(serde::Deserialize)]
pub struct Register {
    name: String,
    email: String,
    password: Secret<String>,
}

#[tracing::instrument(name = "Registering a new user", skip(form, conn))]
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
            .context("Failed to hash password")??;

    let mut tx = conn.begin().await?;

    let user_id = sqlx::query_as!(
        UserId,
        r#"
        INSERT INTO users (name, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING user_id;
        "#,
        name,
        email,
        password_hash.expose_secret(),
    )
    .fetch_one(&mut tx)
    .await
    .on_constraint("users_name_key", |_| {
        ApiError::unprocessable_entity([("name", "name already taken")])
    })
    .on_constraint("users_email_key", |_| {
        ApiError::unprocessable_entity([("email", "email already taken")])
    })?;

    let token = generate_confirmation_token();

    store_token(&mut tx, &token, user_id.user_id)
        .await
        .context("Failed to store the confirmation token for a new subscriber.")?;

    enqueue_delivery_task(&mut tx, token)
        .await
        .context("Failed to enqueue confirmation delivery task")?;

    tx.commit().await?;

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
    let UpdatePassword { name, password, .. } = form;
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

#[derive(serde::Deserialize)]
struct ForgetPassword {
    name: String,
    email: String,
}

async fn forget_password_gen(
    Form(form): Form<ForgetPassword>,
    DatabaseConnection(mut conn): DatabaseConnection,
    Extension(client): Extension<EmailClient>,
) -> Result<(), ApiError> {
    let ForgetPassword { name, email } = form;

    let result = sqlx::query!(
        r#"
        SELECT user_id
        FROM users
        WHERE name = $1 AND email = $2
        "#,
        name,
        email,
    )
    .fetch_optional(&mut conn)
    .await?;

    if let Some(user_id) = result.map(|r| r.user_id) {
        let token = uuid::Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO forget_password_tokens (token, user_id)
            VALUES ($1, $2)
            "#,
            token,
            user_id
        )
        .execute(&mut conn)
        .await?;

        client
            .send_mail(
                Email::parse(email)?,
                "Recipe App - Your password reset",
                &format!(
                    "Visit http://localhost:3001/forget_password?token={}",
                    token
                ),
                &format!(
                    "Visit http://localhost:3001/forget_password?token={}",
                    token
                ),
            )
            .await?;
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct ForgetPasswordParameters {
    token: uuid::Uuid,
}

#[derive(serde::Deserialize)]
pub struct ResetPassword {
    password: Secret<String>,
}

#[derive(serde::Deserialize)]
pub struct ResetDetails {
    token: uuid::Uuid,
    user_id: uuid::Uuid,
}

async fn forget_password(
    Query(params): Query<ForgetPasswordParameters>,
    DatabaseConnection(mut conn): DatabaseConnection,
    Form(form): Form<ResetPassword>,
) -> Result<(), ApiError> {
    let mut tx = conn.begin().await?;

    // TODO: expiry is a little dumb this way, but let's just don't care about it for now.
    let result = sqlx::query_as!(
        ResetDetails,
        r#"
        SELECT user_id, token
        FROM forget_password_tokens
        WHERE created_at > current_timestamp - INTERVAL '2 days' AND token = $1
        ORDER BY created_at DESC
        LIMIT 1;
        "#,
        params.token,
    )
    .fetch_optional(&mut tx)
    .await?;

    if let Some(reset_details) = result {
        let password_hash =
            crate::utils::spawn_blocking_with_tracing(move || compute_password_hash(form.password))
                .await
                .context("Failed to hash password")??;

        sqlx::query!(
            r#"
            UPDATE users
            SET password_hash = $1
            WHERE user_id = $2
            "#,
            password_hash.expose_secret(),
            reset_details.user_id,
        )
        .execute(&mut tx)
        .await
        .context("Failed to change user's password in the database.")?;

        sqlx::query!(
            r#"
            DELETE FROM forget_password_tokens
            WHERE user_id = $1 AND token = $2
            "#,
            reset_details.user_id,
            reset_details.token,
        )
        .execute(&mut tx)
        .await
        .context("Failed to delete from forget_password_tokens.")?;

        tx.commit().await?;
        Ok(())
    } else {
        Err(ApiError::BadRequest)
    }
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct AuthRequest {
    code: String,
    state: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DiscordUser {
    id: String,
    avatar: Option<String>,
    username: String,
    discriminator: String,
    email: String,
}

async fn discord_authorize(
    Query(query): Query<AuthRequest>,
    Extension(mut session): Extension<Session>,
    Extension(oauth_client): Extension<BasicClient>,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<(), ApiError> {
    // Get an auth token
    let token = oauth_client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .request_async(async_http_client)
        .await
        .context("Failed to exchange authorization code")?;

    let csrf_token = session
        .get::<CsrfToken>("oauth_csrf_token")
        .ok_or(ApiError::BadRequest)?;

    // Protect Cross Site Request Forgery Attacks
    if csrf_token.secret() != CsrfToken::new(query.state).secret() {
        return Err(ApiError::BadRequest);
    }

    // Cleanup session, we don't need to store csrf_token anymore.
    session.remove("oauth_csrf_token");

    // Fetch user data from discord
    let client = reqwest::Client::new();
    let user_data: DiscordUser = client
        .get("https://discordapp.com/api/users/@me")
        .bearer_auth(token.access_token().secret())
        .send()
        .await?
        .json::<DiscordUser>()
        .await
        .expect("Discord promised");

    let mut tx = conn.begin().await?;

    let user = sqlx::query!(
        r#"
            SELECT user_id FROM users
            WHERE oauth_provider = 'discord' AND oauth_id = $1
        "#,
        user_data.id
    )
    .fetch_optional(&mut tx)
    .await?;

    let user_id = if let Some(u) = user {
        u.user_id
    } else {
        // Assign a random strong password for the user.
        let random_pw = Secret::new(generate_confirmation_token());

        let password_hash =
            crate::utils::spawn_blocking_with_tracing(move || compute_password_hash(random_pw))
                .await
                .context("Failed to hash password")??;
        let user = sqlx::query!(
            r#"
            INSERT INTO users (name, email, confirmed, password_hash, oauth_provider, oauth_id)
            VALUES ($1, $2, 'TRUE', $3, 'discord', $4)
            RETURNING user_id;
            "#,
            user_data.username,
            user_data.email,
            password_hash.expose_secret(),
            user_data.id,
        )
        .fetch_one(&mut tx)
        .await?;
        user.user_id
    };
    tx.commit().await?;

    session.regenerate();
    session
        .insert("user_id", user_id)
        .expect("user_id is serializable");

    Ok(())
}

#[derive(Clone, serde::Serialize)]
struct RedirectUri {
    uri: String,
}

async fn discord_auth(
    Extension(client): Extension<BasicClient>,
    Extension(mut session): Extension<Session>,
) -> Result<Json<RedirectUri>, ApiError> {
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    session
        .insert("oauth_csrf_token", csrf_token)
        .expect("csrf_token is serializable");

    Ok(Json(RedirectUri {
        uri: auth_url.to_string(),
    }))
}
