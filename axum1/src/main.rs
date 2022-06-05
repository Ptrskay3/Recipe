use anyhow::Context;
use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Redirect},
    routing::{delete, get},
    Extension, Json, Router, TypedHeader, headers::Cookie,
};
use axum1::{
    extractors::{internal_error, AuthUser, DatabaseConnection, RedisConnection},
    AXUM_SESSION_COOKIE_NAME,
};
use rand::{distributions::Alphanumeric, Rng};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_timeout(std::time::Duration::from_secs(3))
        .connect(&db_conn_str)
        .await
        .context("could not connect to database_url")?;

    sqlx::migrate!().run(&db_pool).await?;

    let redis_conn_str =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let store = RedisSessionStore::new(&*redis_conn_str).context("failed to connect redis")?;

    let app = Router::new()
    .route("/", get(index))
        .route("/health_check", get(|| async { StatusCode::OK }))
        .route("/pg", get(pg_health))
        .route("/users", get(get_users))
        .route("/insert", get(insert_garbage))
        .route("/auth", get(authorize))
        .route("/logout", get(logout))
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

async fn index(user: Option<AuthUser>) -> impl IntoResponse {
    match user {
        Some(_) => "Hello User, you are logged in!",
        _ => "Hi stranger!"
    }
}

async fn pg_health(
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("SELECT 'hello world from pg'")
        .fetch_one(&mut conn)
        .await
        .map_err(internal_error)
}

async fn authorize(
    RedisConnection(store): RedisConnection,
    // DatabaseConnection(mut conn): DatabaseConnection
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();

    let user_id = AuthUser::new();
    let mut session = Session::new();
    session.insert("user_id", user_id).unwrap();
    let cookie = store.store_session(session).await.unwrap().unwrap();

    let cookie = format!(
        "{}={}; SameSite=Lax; Path=/",
        AXUM_SESSION_COOKIE_NAME, cookie
    );

    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    (headers, Redirect::to("/"))
}

async fn logout(
    _user: AuthUser,
    RedisConnection(store): RedisConnection,
    TypedHeader(cookie): TypedHeader<Cookie>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    // TODO: Remove the absurd amount of `unwrap` calls.
    let session_cookie = cookie.get(AXUM_SESSION_COOKIE_NAME).unwrap();
    let loaded_session = store.load_session(session_cookie.to_owned()).await.unwrap().unwrap();
    store.destroy_session(loaded_session).await.unwrap();

    // Unset cookies at client side
    let cookie = format!(
        "{}={}; Max-Age=0",
        AXUM_SESSION_COOKIE_NAME, ""
    );

    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    (headers, Redirect::to("/pg"))
}

async fn clean(DatabaseConnection(conn): DatabaseConnection) -> Result<(), (StatusCode, String)> {
    let mut conn = conn;
    sqlx::query("TRUNCATE TABLE users")
        .execute(&mut conn)
        .await
        .map_err(internal_error)?;
    Ok(())
}

async fn get_users(
    user_id: AuthUser,
    DatabaseConnection(mut conn): DatabaseConnection,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    println!("decoded user_id is {:?}", user_id);
    let ordering = match params.get_key_value("order_by") {
        Some((_, order)) => match order.to_lowercase().as_ref() {
            "desc" => "desc",
            _ => "asc",
        },
        _ => "asc",
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
    sqlx::query!(
        r#"INSERT INTO users (name, email) VALUES ($1, $2)"#,
        name,
        email
    )
    .execute(&mut conn)
    .await
    .map_err(internal_error)?;
    Ok(())
}
