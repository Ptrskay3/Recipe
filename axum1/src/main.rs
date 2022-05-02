use axum::{http::StatusCode, routing::get, Extension, Json, Router};
use axum1::db::*;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let db_conn_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/hummus".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_timeout(std::time::Duration::from_secs(3))
        .connect(&db_conn_str)
        .await
        .expect("pg is unreachable");

    let app = Router::new()
        .route("/", get(|| async { "Hello world!" }))
        .route("/pg", get(conn_extractor))
        .route("/users", get(get_users))
    .layer(Extension(pool));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn conn_extractor(
    DatabaseConnection(conn): DatabaseConnection,
) -> Result<String, (StatusCode, String)> {
    let mut conn = conn;
    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&mut conn)
        .await
        .map_err(internal_error)
}

#[derive(sqlx::FromRow, serde::Deserialize, Debug, serde::Serialize, Clone)]
pub(crate) struct User {
    id: i64,
    name: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn get_users(
    DatabaseConnection(conn): DatabaseConnection,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let mut conn = conn;
    let users = sqlx::query_as::<_, User>(r#"SELECT * FROM users"#)
        .fetch_all(&mut conn)
        .await
        .map_err(internal_error)?;
    Ok(Json(users))
}
