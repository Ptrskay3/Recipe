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
    routing::{delete, get, post, put},
    Extension, Json, Router, TypedHeader,
};
use axum1::{
    error::{ApiError, ResultExt},
    extractors::{AuthUser, DatabaseConnection, RedisConnection},
    routes::auth_router,
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

    let app = Router::new()
        .route("/", get(index))
        .route("/health_check", get(|| async { StatusCode::OK }))
        .route("/pg", get(pg_health))
        .route("/users", get(get_users))
        .route("/register", post(register))
        .route("/auth", post(authorize))
        .route("/logout", get(logout))
        .route("/protected", get(protected))
        .route("/update_password", put(update_password))
        .route("/clean", delete(clean))
        .nest("/api", auth_router())
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

async fn pg_health(DatabaseConnection(mut conn): DatabaseConnection) -> Result<(), ApiError> {
    sqlx::query_scalar("SELECT 'hello world from pg'")
        .fetch_one(&mut conn)
        .await?;
    Ok(())
}

async fn set_authorization_headers(
    store: RedisSessionStore,
    user_id: uuid::Uuid,
) -> Result<HeaderMap, ApiError> {
    let mut headers = HeaderMap::new();
    let mut session = Session::new();
    // TODO: do not hardcode this here..
    session.expire_in(std::time::Duration::from_secs(1200));
    session.insert("user_id", user_id).unwrap();
    let cookie = store.store_session(session).await?.unwrap();
    let cookie = format!(
        "{}={}; SameSite=Lax; Path=/",
        AXUM_SESSION_COOKIE_NAME, cookie
    );
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    Ok(headers)
}

async fn unset_authorization_headers(
    cookie: Cookie,
    store: RedisSessionStore,
) -> Result<HeaderMap, ApiError> {
    let mut headers = HeaderMap::new();
    let session_cookie = cookie.get(AXUM_SESSION_COOKIE_NAME).unwrap();
    let loaded_session = store
        .load_session(session_cookie.to_owned())
        .await?
        .unwrap();
    store.destroy_session(loaded_session).await.unwrap();

    // Unset cookies at client side
    let cookie = format!("{}={}; Max-Age=0", AXUM_SESSION_COOKIE_NAME, "");

    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    Ok(headers)
}

async fn authorize(
    Form(credentials): Form<Credentials>,
    RedisConnection(store): RedisConnection,
    conn: DatabaseConnection,
) -> Result<(HeaderMap, Redirect), ApiError> {
    let user_id = validate_credentials(credentials, conn).await?;
    let headers = set_authorization_headers(store, user_id).await?;
    Ok((headers, Redirect::to("/")))
}

async fn logout(
    _user: AuthUser,
    RedisConnection(store): RedisConnection,
    TypedHeader(cookie): TypedHeader<Cookie>,
) -> Result<(HeaderMap, Redirect), ApiError> {
    let headers = unset_authorization_headers(cookie, store).await?;
    Ok((headers, Redirect::to("/")))
}

async fn clean(DatabaseConnection(mut conn): DatabaseConnection) -> Result<(), ApiError> {
    sqlx::query("TRUNCATE TABLE users")
        .execute(&mut conn)
        .await?;
    Ok(())
}

async fn get_users(
    _user_id: AuthUser,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<User>>, ApiError> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at")
        .fetch_all(&mut conn)
        .await?;
    Ok(Json(users))
}

#[derive(Debug, serde::Deserialize)]
pub struct Credentials {
    name: String,
    password: Secret<String>,
}

async fn validate_credentials(
    credentials: Credentials,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<uuid::Uuid, ApiError> {
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
        None => {
            return Err(ApiError::unprocessable_entity([(
                "username",
                "no such user",
            )]))
        }
    };

    axum1::utils::spawn_blocking_with_tracing(move || {
        let expected_password_hash = PasswordHash::new(&expected_password_hash)?;
        Argon2::default().verify_password(
            credentials.password.expose_secret().as_bytes(),
            &expected_password_hash,
        )
    })
    .await
    .map_err(|_| {
        ApiError::Anyhow(anyhow::anyhow!(
            "unexpected error happened during password hashing"
        ))
    })?
    .map_err(|_| ApiError::unprocessable_entity([("password", "password is wrong")]))?;
    // FIXME: after the 0.6 release of sqlx, this nonsense can go away
    Ok(uuid::Uuid::from_bytes(*user_id.as_bytes()))
}

#[derive(serde::Deserialize)]
pub struct UpdatePassword {
    name: String,
    password: Secret<String>,
}

#[derive(serde::Deserialize)]
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
        axum1::utils::spawn_blocking_with_tracing(move || compute_password_hash(password))
            .await
            .map_err(|_| ApiError::Anyhow(anyhow::anyhow!("Failed to hash password")))??;

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
    .on_constraint("users_name_key", |_| {
        ApiError::unprocessable_entity([("name", "name already taken")])
    })
    .on_constraint("users_email_key", |_| {
        ApiError::unprocessable_entity([("email", "email already taken")])
    })?;
    Ok(())
}

async fn update_password(
    user_id: AuthUser,
    Form(form): Form<UpdatePassword>,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<(), ApiError> {
    let UpdatePassword { name, password } = form;
    let password_hash =
        axum1::utils::spawn_blocking_with_tracing(move || compute_password_hash(password))
            .await
            .map_err(|_| ApiError::Anyhow(anyhow::anyhow!("Failed to hash password")))??;

    sqlx::query!(
        r#"
        UPDATE users
        SET password_hash = $1
        WHERE name = $2 AND user_id = $3
        "#,
        password_hash.expose_secret(),
        name,
        // FIXME: after the 0.6 release of sqlx, this nonsense can go away
        <sqlx::types::uuid::Uuid as From<_>>::from(user_id),
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
