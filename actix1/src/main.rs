use std::net::TcpListener;

use actix1::configuration::get_config;
use actix1::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_config().expect("Failed to load configuration");
    let connection = sqlx::PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to database");
    let addr = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(addr)?;
    run(listener, connection)?.await
}
