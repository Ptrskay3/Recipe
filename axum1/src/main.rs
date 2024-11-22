use std::sync::{Arc, Mutex};

use axum1::{
    cli::cli_manager,
    config::get_config,
    queue::run_worker_until_stopped,
    search::run_meili_indexer_until_stopped,
    startup::application,
    task::supervised_task,
    utils::{init_tracing_panic_hook, report_exit},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = get_config().expect("Failed to read configuration.");

    let _guard = sentry::init((
        configuration.clone().sentry_dsn,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum1=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_tracing::layer())
        .init();

    init_tracing_panic_hook();

    let cfg = Arc::new(Mutex::new(configuration.clone()));

    let application_task = tokio::spawn(application(Arc::clone(&cfg)));
    let worker_task = run_worker_until_stopped(Arc::clone(&cfg));
    let meili_indexing_task = run_meili_indexer_until_stopped(Arc::clone(&cfg));

    let (meili_task_spawned, meili_supervisor) = supervised_task(meili_indexing_task);
    let (worker_task_spawned, worker_supervisor) = supervised_task(worker_task);
    let config = Arc::clone(&cfg);

    let cli_manager_task = tokio::spawn(cli_manager(config, meili_supervisor, worker_supervisor));

    tokio::select! {
        f = application_task => report_exit("server", f),
        f = meili_task_spawned => report_exit("meili indexing", f),
        f = worker_task_spawned => report_exit("queue", f),
        f = cli_manager_task => report_exit("CLI Manager", f),
    };

    Ok(())
}
