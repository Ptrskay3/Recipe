use crate::{
    config::Settings,
    email::EmailClient,
    routes::{admin, auth, ingredient, recipe},
    sse::{sse_handler, Notification},
    state::AppState,
    upload,
    utils::{oauth_client_discord, oauth_client_google, shutdown_signal},
};
use anyhow::Context;
use axum::{
    http::HeaderValue,
    routing::{get, get_service},
    Extension, Router,
};
use axum_prometheus::PrometheusMetricLayerBuilder;
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, sync::Arc};
use time::Duration;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};

pub async fn application(
    dynamic_cfg: tokio::sync::watch::Receiver<Settings>,
) -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();

    let config = crate::config::get_config().expect("Configuration file is missing");

    let addr = SocketAddr::from((
        config.application_settings.host,
        config.application_settings.port,
    ));

    let discord_oauth_client = oauth_client_discord(&config);
    let google_oauth_client = oauth_client_google(&config);

    let db_conn_str = config.database.connection_string();

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&db_conn_str)
        .await
        .context("failed to connect to database")?;

    sqlx::migrate!().run(&db_pool).await?;

    let redis_url = config.redis.connection_string();

    let pool = RedisPool::new(
        RedisConfig::from_url_centralized(&redis_url).unwrap(),
        None,
        None,
        None,
        6,
    )?;

    let _redis_connection = pool.connect();
    pool.wait_for_connect().await?;
    tracing::debug!("redis connected.");

    let session_store = RedisStore::new(pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(
            std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| String::from("local"))
                == "production",
        )
        .with_expiry(Expiry::OnInactivity(Duration::minutes(10)));

    let email_client = EmailClient::from_config(config.email_client);

    let (metric_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_ignore_pattern("/admin")
        .with_default_metrics()
        .build_pair();

    let (tx, rx) = tokio::sync::broadcast::channel::<Notification>(16);
    let tx = Arc::new(tx);
    let rx = Arc::new(rx);

    let app_state = AppState {
        db_pool,
        config: dynamic_cfg,
        email_client,
        tx,
        rx,
    };

    let app = Router::<AppState>::new()
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .route("/sse", get(sse_handler))
        .nest("/i", ingredient::router(app_state.clone()))
        .nest("/r", recipe::router())
        .nest("/", auth::router())
        .nest("/admin", admin::router(app_state.clone()))
        .nest("/upload", upload::router(app_state.clone()))
        .fallback_service(get_service(ServeDir::new("static")))
        .layer(
            tower::ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(metric_layer)
                .layer(Extension(discord_oauth_client))
                .layer(Extension(google_oauth_client))
                .layer(
                    CorsLayer::very_permissive()
                        .allow_origin(config.frontend_url.parse::<HeaderValue>().unwrap())
                        .allow_credentials(true),
                )
                .layer(session_layer),
        )
        .with_state(app_state);

    let listener = TcpListener::bind(&addr).await.unwrap();

    tracing::debug!(%addr, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Failed to start server")
}
