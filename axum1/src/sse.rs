use std::{convert::Infallible, sync::Arc};

use axum::{
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    Extension,
};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};

use crate::utils::shutdown_signal;

pub async fn sse_handler(
    Extension(chan): Extension<Arc<tokio::sync::broadcast::Sender<Notification>>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Create an internal channel which transmits all traffic that's coming from our `chan`.
    let (mut tx, rx) = futures::channel::mpsc::channel::<Result<Event, Infallible>>(16);

    let mut sub = chan.subscribe();
    tokio::spawn(async move {
        use futures::SinkExt;

        while let Ok(m) = sub.recv().await {
            let _ = tx.send(Ok(Event::default().json_data(m).unwrap())).await;
        }
    });

    Sse::new(or_until_shutdown(rx)).keep_alive(KeepAlive::default())
}

/// Run a stream until it completes or we receive the shutdown signal.
///
/// Uses the `async-stream` to make things easier to write.
fn or_until_shutdown<S>(stream: S) -> impl Stream<Item = S::Item>
where
    S: Stream,
{
    async_stream::stream! {
        futures::pin_mut!(stream);

        let shutdown_signal = shutdown_signal();
        futures::pin_mut!(shutdown_signal);

        loop {
            tokio::select! {
                Some(item) = stream.next() => {
                    yield item
                }
                _ = &mut shutdown_signal => {
                    break;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum Notification {
    NewRecipe(NewRecipe),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRecipe {
    pub name: String,
}
