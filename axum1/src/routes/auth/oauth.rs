use anyhow::Context;
use async_session::Session;
use axum::{extract::Query, Extension, Json};
use oauth2::{
    reqwest::async_http_client, AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    Scope, StandardRevocableToken, TokenResponse,
};
use secrecy::{ExposeSecret, Secret};
use sqlx::Acquire;

use crate::{
    error::{ApiError, ResultExt},
    extractors::DatabaseConnection,
    session::SessionExt,
    utils::{DiscordOAuthClient, GoogleOAuthClient},
};

use super::{confirm::generate_confirmation_token, password::compute_password_hash};

#[derive(Debug, serde::Deserialize)]
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
    Extension(DiscordOAuthClient(oauth_client)): Extension<DiscordOAuthClient>,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<(), ApiError> {
    let verifier = session
        .get::<PkceCodeVerifier>("pkce_verifier")
        .ok_or(ApiError::BadRequest)?;

    let csrf_token = session
        .get::<CsrfToken>("oauth_csrf_token")
        .ok_or(ApiError::BadRequest)?;

    // Protect Cross-site Request Forgery Attacks
    if csrf_token.secret() != CsrfToken::new(query.state).secret() {
        return Err(ApiError::BadRequest);
    }

    // Cleanup session, we don't need to store csrf_token anymore.
    session.remove("oauth_csrf_token");
    session.remove("pkce_verifier");

    // Get an auth token
    let token = oauth_client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .set_pkce_verifier(verifier)
        .request_async(async_http_client)
        .await
        .map_err(|_| ApiError::BadRequest)?;

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
        .await
        .on_constraint("users_email_key", |_| {
            ApiError::unprocessable_entity([(
                "email",
                "email already exists as a regular (non-oauth) user",
            )])
        })?;
        user.user_id
    };
    tx.commit().await?;

    let token_to_revoke: StandardRevocableToken = match token.refresh_token() {
        Some(token) => token.into(),
        None => token.access_token().into(),
    };

    oauth_client
        .revoke_token(token_to_revoke)
        .unwrap()
        .request_async(async_http_client)
        .await
        .ok();

    session.mark_for_regenerate();
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
    Extension(DiscordOAuthClient(client)): Extension<DiscordOAuthClient>,
    Extension(mut session): Extension<Session>,
) -> Result<Json<RedirectUri>, ApiError> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    session
        .insert("pkce_verifier", pkce_verifier)
        .expect("pkce_verifier is serializable");

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    session
        .insert("oauth_csrf_token", csrf_token)
        .expect("csrf_token is serializable");

    Ok(Json(RedirectUri {
        uri: auth_url.to_string(),
    }))
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct GoogleUser {
    email: String,
    sub: String,
    name: String,
}

pub(super) async fn google_auth(
    Extension(GoogleOAuthClient(client)): Extension<GoogleOAuthClient>,
    Extension(mut session): Extension<Session>,
) -> Result<Json<RedirectUri>, ApiError> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    session
        .insert("pkce_verifier", pkce_verifier)
        .expect("pkce_verifier is serializable");

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    session
        .insert("oauth_csrf_token", csrf_token)
        .expect("csrf_token is serializable");

    Ok(Json(RedirectUri {
        uri: auth_url.to_string(),
    }))
}

pub(super) async fn google_authorize(
    Query(query): Query<AuthRequest>,
    Extension(mut session): Extension<Session>,
    Extension(GoogleOAuthClient(oauth_client)): Extension<GoogleOAuthClient>,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<(), ApiError> {
    let verifier = session
        .get::<PkceCodeVerifier>("pkce_verifier")
        .ok_or(ApiError::BadRequest)?;

    let csrf_token = session
        .get::<CsrfToken>("oauth_csrf_token")
        .ok_or(ApiError::BadRequest)?;

    // Protect Cross-site Request Forgery Attacks
    if csrf_token.secret() != CsrfToken::new(query.state).secret() {
        return Err(ApiError::BadRequest);
    }

    // Cleanup session, we don't need to store these anymore.
    session.remove("oauth_csrf_token");
    session.remove("pkce_verifier");

    // Get an auth token
    let token = oauth_client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .set_pkce_verifier(verifier)
        .request_async(async_http_client)
        .await
        .map_err(|_| ApiError::BadRequest)?;

    // Fetch user data from Google
    let client = reqwest::Client::new();
    let user_data: GoogleUser = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(token.access_token().secret())
        .send()
        .await?
        .json::<GoogleUser>()
        .await
        .expect("Google promised");

    let mut tx = conn.begin().await?;

    let user = sqlx::query!(
        r#"
        SELECT user_id FROM users
        WHERE oauth_provider = 'google' AND oauth_id = $1
        "#,
        user_data.sub
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
            VALUES ($1, $2, 'TRUE', $3, 'google', $4)
            RETURNING user_id;
            "#,
            user_data.name,
            user_data.email,
            password_hash.expose_secret(),
            user_data.sub,
        )
        .fetch_one(&mut tx)
        .await
        .on_constraint("users_email_key", |_| {
            ApiError::unprocessable_entity([(
                "email",
                "email already exists as a regular (non-oauth) user",
            )])
        })?;
        user.user_id
    };
    tx.commit().await?;

    let token_to_revoke: StandardRevocableToken = match token.refresh_token() {
        Some(token) => token.into(),
        None => token.access_token().into(),
    };

    oauth_client
        .revoke_token(token_to_revoke)
        .unwrap()
        .request_async(async_http_client)
        .await
        .ok();

    session.mark_for_regenerate();
    session
        .insert("user_id", user_id)
        .expect("user_id is serializable");

    Ok(())
}
