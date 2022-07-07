use anyhow::Context;
use axum::{extract::Path, routing::get, Json, Router};
use axum_extra::extract::Form;

use crate::{
    error::ApiError,
    extractors::{AuthUser, DatabaseConnection},
};

pub fn recipe_router() -> Router {
    Router::new().route("/:name", get(get_recipe).post(insert_recipe))
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
struct Recipe {
    name: String,
    description: String,
}

async fn get_recipe(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(name): Path<String>,
) -> Result<Json<Recipe>, ApiError> {
    let recipe = sqlx::query_as!(
        Recipe,
        r#"SELECT name, description FROM recipes WHERE name = $1"#,
        name
    )
    .fetch_optional(&mut conn)
    .await
    .context("Failed to query for recipe")?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(recipe))
}

async fn insert_recipe(
    DatabaseConnection(mut conn): DatabaseConnection,
    Form(recipe): Form<Recipe>,
    auth_user: AuthUser,
) -> Result<(), ApiError> {
    sqlx::query!(
        r#"
        INSERT INTO recipes ("name", "description", "creator_id")
        VALUES ($1, $2, $3)
        "#,
        recipe.name,
        recipe.description,
        *auth_user
    )
    .execute(&mut conn)
    .await
    .map_err(|_| ApiError::Conflict)?;

    Ok(())
}
