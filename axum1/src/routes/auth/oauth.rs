use anyhow::Context;
use async_session::Session;
use axum::{extract::Query, Extension, Json};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, CsrfToken, Scope,
    TokenResponse,
};
use secrecy::{ExposeSecret, Secret};
use sqlx::Acquire;

use crate::{error::ApiError, extractors::DatabaseConnection};

use super::{confirm::generate_confirmation_token, password::compute_password_hash};

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
pub(super) struct AuthRequest {
    code: String,
    state: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct DiscordUser {
    id: String,
    avatar: Option<String>,
    username: String,
    discriminator: String,
    email: String,
}

pub(super) async fn discord_authorize(
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
pub(super) struct RedirectUri {
    uri: String,
}

pub(super) async fn discord_auth(
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
