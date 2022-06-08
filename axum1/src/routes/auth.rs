use axum::{middleware::from_extractor, routing::get, Json, Router};

use crate::{
    error::ApiError,
    extractors::{AuthUser, DatabaseConnection},
};

pub fn auth_router() -> Router {
    Router::new()
        .route("/me", get(profile))
        .route_layer(from_extractor::<AuthUser>())
}

#[derive(serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct UserProfile {
    name: String,
    email: String,
}

async fn profile(
    DatabaseConnection(mut conn): DatabaseConnection,
    user_id: AuthUser,
) -> Result<Json<UserProfile>, ApiError> {
    let row = sqlx::query_as!(
        UserProfile,
        r#"
        SELECT name, email
        FROM users
        WHERE user_id = $1;
        "#,
        <sqlx::types::uuid::Uuid as From<_>>::from(user_id),
    )
    .fetch_one(&mut conn)
    .await?;
    Ok(Json(row))
}
