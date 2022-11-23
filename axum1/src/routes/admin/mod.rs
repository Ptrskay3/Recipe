mod middleware;
pub use middleware::AdminUser;

use axum::{http::StatusCode, middleware::from_extractor_with_state, routing::get, Json, Router};

use crate::{
    error::ApiError,
    extractors::{DatabaseConnection, RedisConnection},
    state::AppState,
};

pub fn admin_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/pg", get(pg_health))
        .route("/redis", get(redis_health))
        .route_layer(from_extractor_with_state::<AdminUser, _>(state))
        .route("/health_check", get(|| async { StatusCode::OK }))
}

async fn pg_health(DatabaseConnection(mut conn): DatabaseConnection) -> Result<(), ApiError> {
    let _ = sqlx::query_scalar!("SELECT 1 + 1")
        .fetch_one(&mut conn)
        .await?;
    Ok(())
}

async fn redis_health(RedisConnection(conn): RedisConnection) -> Result<Json<usize>, ApiError> {
    Ok(Json(conn.count().await?))
}
