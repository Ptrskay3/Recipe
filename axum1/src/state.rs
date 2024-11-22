use std::sync::{Arc, Mutex};

use sqlx::PgPool;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::{config::Settings, email::EmailClient, sse::Notification};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub config: Arc<Mutex<Settings>>,
    pub tx: Arc<Sender<Notification>>,
    pub rx: Arc<Receiver<Notification>>,
    pub email_client: EmailClient,
}
