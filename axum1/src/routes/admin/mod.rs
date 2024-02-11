mod middleware;
pub use middleware::AdminUser;

use axum::{http::StatusCode, middleware::from_extractor_with_state, routing::get, Router};

use crate::{error::ApiError, extractors::DatabaseConnection, state::AppState};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/pg", get(pg_health))
        .route_layer(from_extractor_with_state::<AdminUser, _>(state))
        .route("/health_check", get(|| async { StatusCode::OK }))
}

async fn pg_health(DatabaseConnection(mut conn): DatabaseConnection) -> Result<(), ApiError> {
    let _ = sqlx::query_scalar!("SELECT 1 + 1")
        .fetch_one(&mut *conn)
        .await?;
    Ok(())
}
