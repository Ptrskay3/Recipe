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
        .route("/", post(insert_barebone_recipe))
        .route("/new-full", post(insert_full_recipe))
        .route("/:name", get(get_recipe_with_ingredients))
        .route("/my-recipes", get(my_recipes))
        .route(
            "/:name/edit",
            post(add_or_update_ingredient_to_recipe).delete(delete_ingredient_from_recipe),
        )
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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct InsertIngredient {
    name: String,
    quantity: String,
    quantity_unit: String,
}

#[tracing::instrument(skip(conn))]
async fn add_or_update_ingredient_to_recipe(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(name): Path<String>,
    Form(ingredient): Form<InsertIngredient>,
) -> Result<(), ApiError> {
    sqlx::query!(
        r#"
        INSERT INTO ingredients_to_recipes (ingredient_id, recipe_id, quantity, quantity_unit)
        VALUES (
            (SELECT id FROM ingredients WHERE name = $1),
            (SELECT id FROM recipes WHERE name = $2),
            $3,
            $4
        ) ON CONFLICT (ingredient_id, recipe_id) DO
        UPDATE SET
            quantity = EXCLUDED.quantity,
            quantity_unit = EXCLUDED.quantity_unit;
        "#,
        ingredient.name,
        name,
        ingredient.quantity,
        ingredient.quantity_unit
    )
    .execute(&mut conn)
    .await
    .map_err(|_| ApiError::BadRequest)?;

    Ok(())
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct NamedIngredient {
    name: String,
}

#[tracing::instrument(skip(conn))]
async fn delete_ingredient_from_recipe(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(name): Path<String>,
    Form(ingredient): Form<NamedIngredient>,
) -> Result<(), ApiError> {
    sqlx::query!(
        r#"
        DELETE FROM ingredients_to_recipes
        WHERE recipe_id = (SELECT id FROM recipes WHERE name = $1)
        AND ingredient_id = (SELECT id from ingredients WHERE name = $2)
        "#,
        name,
        ingredient.name
    )
    .execute(&mut conn)
    .await
    .context("Failed to delete from ingredients_to_recipes")?;

    Ok(())
}

async fn insert_full_recipe(
    DatabaseConnection(mut conn): DatabaseConnection,
    // We want to accept Json input here instead of Form, because the structure
    // of `RecipeWithIngredients` is too complicated to handle with a form.
    Json(recipe_with_ingredients): Json<RecipeWithIngredients>,
    auth_user: AuthUser,
) -> Result<(), ApiError> {
    let RecipeWithIngredients {
        name,
        description,
        ingredients,
    } = recipe_with_ingredients;
    let recipe = sqlx::query!(
        r#"
        INSERT INTO recipes ("name", "description", "creator_id")
        VALUES ($1, $2, $3)
        RETURNING id;
        "#,
        name,
        description,
        *auth_user
    )
    .fetch_one(&mut conn)
    .await
    .map_err(|_| ApiError::Conflict)?;

    for ingredient in ingredients {
        sqlx::query!(
            r#"
        INSERT INTO ingredients_to_recipes (ingredient_id, recipe_id, quantity, quantity_unit)
        VALUES (
            (SELECT id FROM ingredients WHERE name = $1),
            $2,
            $3,
            $4
        ) ON CONFLICT (ingredient_id, recipe_id) DO
        UPDATE SET
            quantity = EXCLUDED.quantity,
            quantity_unit = EXCLUDED.quantity_unit;
        "#,
            ingredient.name,
            recipe.id,
            ingredient.quantity,
            ingredient.quantity_unit
        )
        .execute(&mut conn)
        .await
        .map_err(|_| {
            ApiError::unprocessable_entity([(
                "ingredient-name",
                format!("{} is not an ingredient", ingredient.name),
            )])
        })?;
    }

    Ok(())
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
struct RecipeWithIngredientCount {
    name: String,
    description: String,
    ingredient_count: Option<i64>,
}

async fn my_recipes(
    DatabaseConnection(mut conn): DatabaseConnection,
    auth_user: AuthUser,
) -> Result<Json<Vec<RecipeWithIngredientCount>>, ApiError> {
    let results = sqlx::query_as!(
        RecipeWithIngredientCount,
        r#"
        SELECT DISTINCT r.name,
                r.description,
                COUNT(ir.recipe_id) OVER (PARTITION BY r.id) AS ingredient_count
        FROM recipes r
        LEFT JOIN ingredients_to_recipes ir ON ir.recipe_id = r.id
        WHERE creator_id = $1;
        "#,
        *auth_user
    )
    .fetch_all(&mut conn)
    .await?;

    Ok(Json(results))
}
