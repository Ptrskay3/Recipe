use meilisearch_sdk::Client;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;

use crate::config::Settings;
use crate::queue::get_connection_pool;
use crate::search::run_meili_indexer;
use crate::utils::report_exit;
use crate::PausableFutureSupervisor;

pub async fn cli_manager(
    config: Settings,
    mut supervisor: PausableFutureSupervisor,
) -> Result<(), anyhow::Error> {
    let socket_path = config
        .application_settings
        .cli_unix_socket
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("cli_unix_socket is not set"))?;

    // Clean the tempfile on startup
    if let Err(e) = std::fs::remove_file(socket_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(e.into());
        }
    }

    let listener = UnixListener::bind(socket_path)?;

    tracing::info!("Listening CLI on {:?}", socket_path);

    loop {
        let (mut socket, _) = listener.accept().await?;

        tracing::info!("Accepted connection from {socket:?}");

        let mut buf = [0; 1024];

        let n = socket.read(&mut buf).await?;

        let msg = String::from_utf8_lossy(&buf[..n]);

        // TODO: Parse the strings into enum.
        match msg.as_ref() {
            "index" => {
                tracing::warn!("Requested MeiliSearch indexing through CLI..");
                let result = tokio::spawn(run_meili_indexer_once(config.clone())).await;
                report_exit("Meili indexing", result);
                socket.write_all(b"ok").await?;
                supervisor.pause();
            }
            "resume" => {
                tracing::warn!("Resuming MeiliSearch indexing from CLI..");
                supervisor.resume();
                socket.write_all(b"ok").await?;
            }
            "pause" => {
                tracing::warn!("Pausing MeiliSearch indexing from CLI..");
                supervisor.pause();
                socket.write_all(b"ok").await?;
            }
            cmd => {
                tracing::warn!("Received command '{cmd}', which is not valid in this context.");
                socket.write_all(b"error").await?;
            }
        }
    }
}

// TODO: it's really similar to other functions in `search.rs`. I'm sure there's a better way.
// Currently this is only to work around `tokio::spawn` 'static bound.
pub async fn run_meili_indexer_once(config: Settings) -> Result<(), anyhow::Error> {
    let meili_client = Client::new(config.meili.url, Some(config.meili.master_key));
    let pool = get_connection_pool(&config.database);
    run_meili_indexer(&pool, &meili_client).await
}
