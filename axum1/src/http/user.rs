// use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Deserialize, Serialize, Clone, Debug)]
pub(crate) struct User {
    email: String,
    token: String,
    username: String,
    bio: String,
    image: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct NewUser {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub(crate) struct LoginUser {
    email: String,
    password: String,
}

async fn create_user() {}
