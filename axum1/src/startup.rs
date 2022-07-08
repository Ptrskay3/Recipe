use crate::{
    queue::email::EmailClient,
    routes::{admin_router, auth_router, ingredient_router, recipe_router},
    session::SessionLayer,
    utils::shutdown_signal,
};
use anyhow::Context;
use async_redis_session::RedisSessionStore;
use axum::{
    http::{HeaderValue, Method},
    response::IntoResponse,
    routing::get_service,
    Extension, Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn application() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

    let config = crate::config::get_config().expect("Configuration file is missing");

    // TODO: move this outside of the application worker
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
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&db_conn_str)
        .await
        .context("failed to connect to database")?;

    sqlx::migrate!().run(&db_pool).await?;

    let redis_conn_str = config.redis.connection_string();

    let store =
        RedisSessionStore::new(redis_conn_str.as_ref()).context("failed to connect redis")?;

    if let Some(sentry_dsn) = config.sentry_dsn {
        let _guard = sentry::init((
            sentry_dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            },
        ));
    }

    let email_client = EmailClient::from_config(config.email_client);

    let app = Router::new()
        .nest("/i", ingredient_router())
        .nest("/r", recipe_router())
        .nest("/", auth_router())
        .nest("/admin", admin_router())
        .fallback(get_service(ServeDir::new("./static")).handle_error(handle_asset_error))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(db_pool))
        .layer(Extension(store.clone()))
        .layer(SessionLayer::new(store, config.redis.secret_key.as_bytes()))
        .layer(Extension(email_client.clone()))
        .layer(
            CorsLayer::new()
                .allow_origin(config.frontend_url.parse::<HeaderValue>().unwrap())
                .allow_methods([
                    Method::GET,
                    Method::PUT,
                    Method::POST,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_credentials(true),
        );

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Failed to start server")
}

async fn handle_asset_error(_err: std::io::Error) -> impl IntoResponse {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "something went wrong",
    )
}
