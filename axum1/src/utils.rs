use tokio::task::JoinHandle;

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
