use axum1::{
    config::get_config, queue::run_worker_until_stopped, search::run_meili_indexer_until_stopped,
    startup::application, utils::report_exit,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = get_config().expect("Failed to read configuration.");
    let application_task = tokio::spawn(application());
    let worker_task = tokio::spawn(run_worker_until_stopped(configuration.clone()));
    let meili_indexing_task = tokio::spawn(run_meili_indexer_until_stopped(configuration));

    tokio::select! {
        f = application_task => report_exit("Server", f),
        f = worker_task =>  report_exit("Background worker", f),
        f = meili_indexing_task =>  report_exit("Meili indexer", f),
    };

    Ok(())
}
