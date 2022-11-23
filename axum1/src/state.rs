use std::sync::Arc;

use async_redis_session::RedisSessionStore;
use sqlx::PgPool;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::{config::ApplicationSettings, email::EmailClient, sse::Notification};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub config: ApplicationSettings,
    pub redis_store: RedisSessionStore,
    pub tx: Arc<Sender<Notification>>,
    pub rx: Arc<Receiver<Notification>>,
    pub email_client: EmailClient,
}
