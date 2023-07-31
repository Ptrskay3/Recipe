use async_session::async_trait;
use axum::{
    extract::{FromRef, FromRequestParts, Path},
    http::request::Parts,
    Extension,
};

use crate::{error::ApiError, extractors::DatabaseConnection, state::AppState};

#[derive(Debug)]
pub struct RecipeCreator(uuid::Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for RecipeCreator
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(session) =
            Extension::<crate::session_ext::Session>::from_request_parts(parts, state)
                .await
                .expect("`SessionLayer` should be added");

        let Path(recipe_name) = Path::<String>::from_request_parts(parts, state)
            .await
            .expect("Recipe name is missing from the URL");

        let user_id = session
            .get::<uuid::Uuid>("user_id")
            .ok_or(ApiError::Forbidden)?;

        let DatabaseConnection(mut conn) = DatabaseConnection::from_request_parts(parts, state)
            .await
            .expect("Database extension is missing");

        sqlx::query!(
            "SELECT 1 AS _e FROM recipes WHERE creator_id = $1 AND name = $2",
            user_id,
            recipe_name
        )
        .fetch_optional(&mut *conn)
        .await?
        .ok_or(ApiError::Forbidden)?;

        Ok(Self(user_id))
    }
}
