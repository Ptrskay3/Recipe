use crate::{
    email::EmailClient,
    routes::{admin, auth, ingredient, recipe},
    session::SessionLayer,
    sse::{sse_handler, Notification},
    state::AppState,
    upload,
    utils::{oauth_client_discord, oauth_client_google, shutdown_signal},
};
use anyhow::Context;
use async_redis_session::RedisSessionStore;
use axum::{
    http::HeaderValue,
    routing::{get, get_service},
    Extension, Router,
};
use axum_prometheus::PrometheusMetricLayerBuilder;
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

pub async fn application() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

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

    let redis_conn_str = config.redis.connection_string();

    let redis_store =
        RedisSessionStore::new(redis_conn_str.as_ref()).context("failed to connect redis")?;

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
        config: config.application_settings,
        email_client,
        tx,
        rx,
        redis_store: redis_store.clone(),
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
                    SessionLayer::new(redis_store, config.redis.secret_key.as_bytes())
                        .with_secure(
                            std::env::var("APP_ENVIRONMENT")
                                .unwrap_or_else(|_| String::from("local"))
                                == "production",
                        )
                        .with_persistence(crate::session::Persistence::Always),
                )
                .layer(
                    CorsLayer::very_permissive()
                        .allow_origin(config.frontend_url.parse::<HeaderValue>().unwrap())
                        .allow_credentials(true),
                ),
        )
        .with_state(app_state);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Failed to start server")
}
