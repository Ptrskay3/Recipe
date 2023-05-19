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

pub fn init_tracing_panic_hook() {
    let next_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let payload = panic_info.payload().downcast_ref::<&str>();
        let location = panic_info.location().map(|loc| loc.to_string());

        tracing::error!(
            panic.payload = payload,
            panic.location = location,
            "Unhandled panic"
        );
        next_hook(panic_info);
    }));
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
                "'{}' task failed to complete",
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

    tracing::warn!("signal received, starting graceful shutdown");
}

pub fn oauth_client_discord(config: &Settings) -> DiscordOAuthClient {
    let client_id = config.oauth.discord.client_id.clone();
    let client_secret = config.oauth.discord.client_secret.clone();
    let redirect_url = config.oauth.discord.redirect_url.clone();
    let auth_url = config.oauth.discord.auth_url.clone();
    let token_url = config.oauth.discord.token_url.clone();
    let revocation_url = config.oauth.discord.revocation_url.clone();

    DiscordOAuthClient(
        BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(auth_url).unwrap(),
            Some(TokenUrl::new(token_url).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
        .set_revocation_uri(
            RevocationUrl::new(revocation_url).expect("Invalid revocation endpoint URL"),
        ),
    )
}

pub fn oauth_client_google(config: &Settings) -> GoogleOAuthClient {
    let client_id = config.oauth.google.client_id.clone();
    let client_secret = config.oauth.google.client_secret.clone();
    let redirect_url = config.oauth.google.redirect_url.clone();
    let auth_url = config.oauth.google.auth_url.clone();
    let token_url = config.oauth.google.token_url.clone();
    let revocation_url = config.oauth.google.revocation_url.clone();

    GoogleOAuthClient(
        BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(auth_url).unwrap(),
            Some(TokenUrl::new(token_url).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
        .set_revocation_uri(
            RevocationUrl::new(revocation_url).expect("Invalid revocation endpoint URL"),
        ),
    )
}

#[derive(Clone, Debug)]
pub struct GoogleOAuthClient(pub BasicClient);

#[derive(Clone, Debug)]
pub struct DiscordOAuthClient(pub BasicClient);
