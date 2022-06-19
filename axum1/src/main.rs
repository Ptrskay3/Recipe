use anyhow::Context;
use async_redis_session::RedisSessionStore;
use axum::{
    http::{HeaderValue, Method},
    Extension, Router,
};
use axum1::{
    routes::{admin_router, auth_router, ingredient_router},
};
use axum_extra::extract::cookie::Key;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let config = axum1::config::get_config().expect("Configuration file is missing");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "axum1=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = SocketAddr::from(([127, 0, 0, 1], config.application_port));

    let db_conn_str = config.database.connection_string();

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_timeout(std::time::Duration::from_secs(3))
        .connect(&db_conn_str)
        .await
        .context("could not connect to database_url")?;

    sqlx::migrate!().run(&db_pool).await?;

    let redis_conn_str = config.redis.connection_string();

    let store =
        RedisSessionStore::new(redis_conn_str.as_ref()).context("failed to connect redis")?;

    let key = Key::generate();

    if let Some(sentry_dsn) = config.sentry_dsn {
        let _guard = sentry::init((
            sentry_dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            },
        ));
    }

    let app = Router::new()
        .nest("/i", ingredient_router())
        .nest("/", auth_router())
        .nest("/admin", admin_router())
        .layer(TraceLayer::new_for_http())
        .layer(Extension(store))
        .layer(Extension(db_pool))
        .layer(Extension(key))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3001".parse::<HeaderValue>().unwrap())
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::DELETE,
                    Method::PATCH,
                    Method::PUT,
                ])
                .allow_credentials(true),
        );

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
