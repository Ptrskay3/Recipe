use std::fmt::{Debug, Display};

use axum1::{config::get_config, queue::run_worker_until_stopped, startup::application};
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = get_config().expect("Failed to read configuration.");
    let application_task = tokio::spawn(application());
    let worker_task = tokio::spawn(run_worker_until_stopped(configuration));

    tokio::select! {
        f = application_task => report_exit("API", f),
        f = worker_task =>  report_exit("Background worker", f),
    };

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}
