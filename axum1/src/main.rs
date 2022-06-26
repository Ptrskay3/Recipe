use axum1::{
    config::get_config, queue::run_worker_until_stopped, startup::application, utils::report_exit,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = get_config().expect("Failed to read configuration.");
    let application_task = tokio::spawn(application());
    let worker_task = tokio::spawn(run_worker_until_stopped(configuration));

    tokio::select! {
        f = application_task => report_exit("Server", f),
        f = worker_task =>  report_exit("Background worker", f),
    };

    Ok(())
}
