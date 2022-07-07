use anyhow::Context;
use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::Form;
use sqlx::Acquire;

use crate::{
    error::ApiError,
    extractors::{AuthUser, DatabaseConnection},
};

pub fn recipe_router() -> Router {
    Router::new()
        .route("/:name", get(get_recipe_with_ingredients))
        .route("/", post(insert_barebone_recipe))
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
struct RecipeWithIngredients {
    name: String,
    description: String,
    ingredients: Vec<DetailedIngredient>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow)]

struct DetailedIngredient {
    name: String,
    quantity: String,
    quantity_unit: String,
}

#[tracing::instrument(skip(conn))]
async fn get_recipe_with_ingredients(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(name): Path<String>,
) -> Result<Json<RecipeWithIngredients>, ApiError> {
    let mut tx = conn.begin().await?;

    // A little bit clunky, but better be safe than overly smart.
    let recipe = sqlx::query_as!(
        Recipe,
        r#"
        SELECT name, description
        FROM recipes
        WHERE name = $1
        "#,
        name
    )
    .fetch_optional(&mut tx)
    .await
    .context("Failed to query recipe")?
    .ok_or(ApiError::NotFound)?;

    let ingredients: Vec<DetailedIngredient> = sqlx::query_as!(
        DetailedIngredient,
        r#"
        SELECT i.name, ir.quantity, ir.quantity_unit FROM recipes r
        INNER JOIN ingredients_to_recipes ir
        ON r.id = ir.recipe_id
        INNER JOIN ingredients i
        ON i.id = ir.ingredient_id
        WHERE r.name = $1;
        "#,
        name
    )
    .fetch_all(&mut tx)
    .await
    .context("Failed to query recipe ingredients")?;
    tx.commit().await?;

    Ok(Json(RecipeWithIngredients {
        ingredients,
        name: recipe.name,
        description: recipe.description,
    }))
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
struct Recipe {
    name: String,
    description: String,
}

async fn insert_barebone_recipe(
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
