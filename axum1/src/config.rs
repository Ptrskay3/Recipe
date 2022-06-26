use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};

use crate::{
    error::ApiError,
    queue::email::{Email, EmailClient},
};

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub application_port: u16,
    pub frontend_url: String,
    pub sentry_dsn: Option<String>,
    pub email_client: EmailClientSettings,
}

#[derive(Deserialize)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: Secret<String>,
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<Email, ApiError> {
        Email::parse(self.sender_email.clone())
    }

    pub fn client(self) -> EmailClient {
        let sender_email = self.sender().expect("Invalid sender email address.");
        let timeout = self.timeout();
        EmailClient::new(
            self.base_url,
            sender_email.into(),
            self.authorization_token,
            timeout,
        )
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

#[derive(Deserialize)]
pub struct RedisSettings {
    pub host: String,
    pub port: u16,
    pub secret_key: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        )
    }

    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}

impl RedisSettings {
    pub fn connection_string(&self) -> String {
        format!("redis://{}:{}", self.host, self.port)
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("configuration"))
        .build()
        .unwrap();
    settings.try_deserialize()
}
