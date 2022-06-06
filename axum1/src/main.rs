use anyhow::Context;
use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::{
    extract::Form,
    headers::Cookie,
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{delete, get, post},
    Extension, Json, Router, TypedHeader,
};
use axum1::{
    error::ApiError,
    extractors::{internal_error, AuthUser, DatabaseConnection, RedisConnection},
    AXUM_SESSION_COOKIE_NAME,
};
use secrecy::{ExposeSecret, Secret};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
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

    let store =
        RedisSessionStore::new(redis_conn_str.as_ref()).context("failed to connect redis")?;

    let app = Router::new()
        .route("/", get(index))
        .route("/health_check", get(|| async { StatusCode::OK }))
        .route("/pg", get(pg_health))
        .route("/users", get(get_users))
        .route("/register", post(register))
        .route("/auth", get(authorize))
        .route("/logout", get(logout))
        .route("/protected", get(protected))
        .route("/clean", delete(clean))
        .layer(TraceLayer::new_for_http())
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
        _ => "Hi stranger!",
    }
}

async fn protected(user: Option<AuthUser>) -> Result<String, ApiError> {
    match user {
        Some(user) => Ok(format!(
            "This is the protected area, here is your data: {:?}",
            user
        )),
        _ => Err(ApiError::Unauthorized),
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
    // TODO: Remove the absurd amount of `unwrap` calls.
    let mut headers = HeaderMap::new();
    // TODO: currently this always succeeds, but we'll need to build authentication.
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
    let loaded_session = store
        .load_session(session_cookie.to_owned())
        .await
        .unwrap()
        .unwrap();
    store.destroy_session(loaded_session).await.unwrap();

    // Unset cookies at client side
    let cookie = format!("{}={}; Max-Age=0", AXUM_SESSION_COOKIE_NAME, "");

    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    (headers, Redirect::to("/"))
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
    _user_id: AuthUser,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let query = format!("SELECT * FROM users u order by u.created_at");
    let users = sqlx::query_as::<_, User>(&query)
        .fetch_all(&mut conn)
        .await
        .map_err(internal_error)?;
    Ok(Json(users))
}

pub struct Credentials {
    name: String,
    password: Secret<String>,
}

async fn validate_credentials(
    credentials: Credentials,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<sqlx::types::uuid::Uuid, ApiError> {
    let row: Option<_> = sqlx::query!(
        r#"
        SELECT user_id, password_hash
        FROM users
        WHERE name = $1
        "#,
        credentials.name,
    )
    .fetch_optional(&mut conn)
    .await
    .context("Failed to perform a query to retrieve stored credentials.")?;

    let (expected_password_hash, user_id) = match row {
        Some(row) => (row.password_hash, row.user_id),
        None => return Err(ApiError::Unauthorized),
    };

    let expected_password_hash = PasswordHash::new(&expected_password_hash)
        .context("Failed to parse hash in PHC string format.")
        .map_err(|_| ApiError::Unauthorized)?;

    Argon2::default()
        .verify_password(
            credentials.password.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .map_err(|_| ApiError::Anyhow(anyhow::anyhow!("Failed to validate credentials")))?;
    Ok(user_id)
}

#[derive(serde::Deserialize, Debug)]
pub struct UpdatePassword {
    name: String,
    password: Secret<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Register {
    name: String,
    email: String,
    password: Secret<String>,
}

async fn register(
    Form(form): Form<Register>,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<(), ApiError> {
    let Register {
        name,
        email,
        password,
    } = form;
    let password_hash =
        axum1::utils::spawn_blocking_with_tracing(move || compute_password_hash(password.clone()))
            .await
            .map_err(|_| ApiError::Anyhow(anyhow::anyhow!("Failed to hash password")))?
            .context("Failed to hash password")?;

    sqlx::query!(
        r#"
        INSERT INTO users (name, email, password_hash)
        VALUES ($1, $2, $3)
        "#,
        name,
        email,
        password_hash.expose_secret(),
    )
    .execute(&mut conn)
    .await
    .context("Failed to insert new user.")?;
    Ok(())
}

async fn update_password(
    Form(form): Form<UpdatePassword>,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<(), ApiError> {
    let UpdatePassword { name, password } = form;
    let password_hash =
        axum1::utils::spawn_blocking_with_tracing(move || compute_password_hash(password.clone()))
            .await
            .map_err(|_| ApiError::Anyhow(anyhow::anyhow!("Failed to hash password")))?
            .context("Failed to hash password")?;

    sqlx::query!(
        r#"
        UPDATE users
        SET password_hash = $1
        WHERE name = $2
        "#,
        password_hash.expose_secret(),
        name
    )
    .execute(&mut conn)
    .await
    .context("Failed to change user's password in the database.")?;
    Ok(())
}

fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)?
    .to_string();
    Ok(Secret::new(password_hash))
}
