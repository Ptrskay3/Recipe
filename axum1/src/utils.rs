use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, RevocationUrl, TokenUrl,
};
use tokio::task::{JoinError, JoinHandle};

use std::fmt::{Debug, Display};

use crate::config::Settings;

/// To play nicely with tokio, we must offload our CPU-intensive task to a
/// separate threadpool using `tokio::task::spawn_blocking`. Those threads
/// are reserved for blocking operations and do not interfere with
/// the scheduling of async tasks.
///
/// This function takes care of attaching the current span to the newly spawn
/// thread to have appropriate logging.
pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}

pub fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name);
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            );
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            );
        }
    }
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}

pub fn oauth_client_discord(config: &Settings) -> DiscordOAuthClient {
    let client_id = config.oauth.discord.client_id.clone();
    let client_secret = config.oauth.discord.client_secret.clone();

    // TODO: do not hardcode these here
    let redirect_url = "http://localhost:3001/auth/discord_authorize".to_owned();
    let auth_url = "https://discord.com/api/oauth2/authorize?response_type=code".to_string();
    let token_url = "https://discord.com/api/oauth2/token".to_string();

    DiscordOAuthClient(
        BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(auth_url).unwrap(),
            Some(TokenUrl::new(token_url).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap()),
    )
}

pub fn oauth_client_google(config: &Settings) -> GoogleOAuthClient {
    let client_id = config.oauth.google.client_id.clone();
    let client_secret = config.oauth.google.client_secret.clone();

    // TODO: do not hardcode these here
    let redirect_url = "http://localhost:3001/auth/google_authorize".to_owned();
    let auth_url = "https://accounts.google.com/o/oauth2/v2/auth".to_string();
    let token_url = "https://www.googleapis.com/oauth2/v3/token".to_string();

    GoogleOAuthClient(
        BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(auth_url).unwrap(),
            Some(TokenUrl::new(token_url).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
        .set_revocation_uri(
            RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
                .expect("Invalid revocation endpoint URL"),
        ),
    )
}

#[derive(Clone, Debug)]
pub struct GoogleOAuthClient(pub BasicClient);

#[derive(Clone, Debug)]
pub struct DiscordOAuthClient(pub BasicClient);
