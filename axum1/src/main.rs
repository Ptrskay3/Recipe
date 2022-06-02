use anyhow::Context;
use async_session::MemoryStore;
use axum::{http::StatusCode, routing::{get, delete}, Extension, Json, Router, response::IntoResponse, extract::Query};
use axum1::extractors::{DatabaseConnection, UserIdFromSession, internal_error};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use rand::{distributions::Alphanumeric, Rng};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "axum1=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let db_conn_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432".to_string());

    println!("trying to connect to.. {:?}", db_conn_str);

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_timeout(std::time::Duration::from_secs(3))
        .connect(&db_conn_str)
        .await
        .context("could not connect to database_url")?;

    sqlx::migrate!().run(&db_pool).await?; 

    let store = MemoryStore::new();

    let app = Router::new()
        .route("/health_check", get(|| async { StatusCode::OK }))
        .route("/pg", get(pg_health))
        .route("/users", get(get_users))
        .route("/insert", get(insert_garbage))
        .route("/clean", delete(clean))
        .layer(Extension(store))
        .layer(Extension(db_pool));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[derive(sqlx::FromRow, serde::Deserialize, Debug, serde::Serialize, Clone)]
pub(crate) struct User {
    id: i32,
    name: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn pg_health(
    DatabaseConnection(conn): DatabaseConnection,
) -> Result<String, (StatusCode, String)> {
    let mut conn = conn;
    sqlx::query_scalar("SELECT 'hello world from pg'")
        .fetch_one(&mut conn)
        .await
        .map_err(internal_error)
}

async fn clean(
    DatabaseConnection(conn): DatabaseConnection,
) -> Result<(), (StatusCode, String)> {
    let mut conn = conn;
    sqlx::query("TRUNCATE TABLE users")
        .execute(&mut conn)
        .await
        .map_err(internal_error)?;
        Ok(())
}

async fn get_users(
    _user_id: UserIdFromSession,
    DatabaseConnection(conn): DatabaseConnection,
    Query(params): Query<std::collections::HashMap<String, String>>
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let mut conn = conn;

    let ordering = match params.get_key_value("order_by") {
        Some((_, order)) => match order.to_lowercase().as_ref() {
            "desc" => "desc",
            _ => "asc", 
        }
        _ => "asc"
    };
    let query = format!("SELECT * FROM users u order by u.created_at {ordering}");
    let users = sqlx::query_as::<_, User>(&query)
        .fetch_all(&mut conn)
        .await
        .map_err(internal_error)?;
    Ok(Json(users))
}

async fn insert_garbage(
    DatabaseConnection(conn): DatabaseConnection,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut conn = conn;
    let name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(9)
        .map(char::from)
        .collect();
      let email: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(9)
        .map(char::from)
        .collect();
    sqlx::query!(r#"INSERT INTO users (name, email) VALUES ($1, $2)"#, name, email)
        .execute(&mut conn)
        .await
        .map_err(internal_error)?;
        Ok(())
}
