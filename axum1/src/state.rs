use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::{
    broadcast,
    watch,
};

use crate::{config::Settings, email::EmailClient, sse::Notification};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub config: watch::Receiver<Settings>,
    pub tx: Arc<broadcast::Sender<Notification>>,
    pub rx: Arc<broadcast::Receiver<Notification>>,
    pub email_client: EmailClient,
}
