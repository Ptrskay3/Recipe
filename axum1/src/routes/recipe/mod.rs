use anyhow::Context;
use axum::{
    extract::{Json, Path, Query},
    http::StatusCode,
    routing::{get, post},
    Extension, Router,
};
use axum_extra::extract::Form;
use sqlx::{types::BigDecimal, Acquire};
use validator::Validate;

use crate::{
    error::{ApiError, ResultExt},
    extractors::{AuthUser, DatabaseConnection, MaybeAuthUser},
    RE_RECIPE,
};

mod helpers;
use helpers::{DifficultyLevel, TypeByTime};

use self::helpers::QuantityUnit;

pub fn recipe_router() -> Router {
    let action_router = Router::new()
        .route("/my-recipes", get(my_recipes))
        .route("/favorites", get(my_favorite_recipes))
        .route("/popular", get(most_popular_recipes));

    Router::new()
        .route("/", post(insert_full_recipe))
        .route("/:name", get(get_recipe_with_ingredients))
        .route("/:name/favorite", post(toggle_favorite_recipe))
        .route(
            "/:name/ingredient",
            post(add_or_update_ingredient_to_recipe).delete(delete_ingredient_from_recipe),
        )
        .nest("/action", action_router)
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
struct RecipeDetailedWithFav {
    name: String,
    description: String,
    prep_time: i32,
    cook_time: i32,
    difficulty: DifficultyLevel,
    steps: Vec<String>,
    cuisine: String,
    meal_type: TypeByTime,
    ingredients: Vec<DetailedIngredient>,
    full_calories: f32,
    favorited: bool,
}
#[derive(
    Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow, validator::Validate,
)]
struct RecipeWithIngredients {
    #[validate(
        length(
            min = 2,
            max = 250,
            message = "should be at least 2 characters, but no more than 250"
        ),
        regex(
            path = "RE_RECIPE",
            message = "only letters, digits, and non-leading and non-trailing dashes are allowed"
        )
    )]
    name: String,
    #[validate(length(
        min = 2,
        max = 250,
        message = "should be at least 2 characters, but no more than 250"
    ))]
    description: String,
    prep_time: i32,
    cook_time: i32,
    difficulty: DifficultyLevel,
    steps: Vec<String>,
    cuisine: String,
    meal_type: TypeByTime,
    ingredients: Vec<DetailedIngredient>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
struct DetailedIngredient {
    name: String,
    quantity: String,
    quantity_unit: String,
    calories_per_100g: f32,
}

#[tracing::instrument(skip(conn, maybe_auth_user))]
async fn get_recipe_with_ingredients(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(name): Path<String>,
    maybe_auth_user: MaybeAuthUser,
) -> Result<Json<RecipeDetailedWithFav>, ApiError> {
    let mut tx = conn.begin().await?;

    // A little bit clunky, but better be safe than overly smart.
    let recipe = sqlx::query_as!(
        RecipeFull,
        r#"
        SELECT r.name, description, prep_time, cook_time, difficulty as "difficulty: DifficultyLevel",
        steps, c.name as cuisine, meal_type as "meal_type: TypeByTime"
        FROM recipes r
        INNER JOIN cuisines c ON c.id = r.cuisine_id
        WHERE r.name = $1
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
        SELECT i.name, i.calories_per_100g, ir.quantity, ir.quantity_unit FROM recipes r
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

    let full_calories = ingredients.iter().fold(0.0, |acc, ingredient| {
        let multiplier = QuantityUnit::try_from(ingredient.quantity_unit.as_str())
            .unwrap_or_default()
            .get_multiplier_for_g();
        acc + (ingredient.calories_per_100g * multiplier * ingredient.quantity.parse::<f32>().unwrap_or(0.0) // We ignore non-numeric quantities
            / 100.0)
    });

    let favorited = if let Some(user_id) = maybe_auth_user.into_inner() {
        sqlx::query!(
            r#"
        SELECT 1 as _e FROM favorite_recipe
        WHERE user_id = $1 AND recipe_id = (SELECT id FROM recipes WHERE name = $2)"#,
            *user_id,
            recipe.name
        )
        .fetch_optional(&mut tx)
        .await
        .context("failed to query for favorite recipes")?
        .is_some()
    } else {
        false
    };

    tx.commit().await?;

    Ok(Json(RecipeDetailedWithFav {
        ingredients,
        name: recipe.name,
        description: recipe.description,
        prep_time: recipe.prep_time,
        cook_time: recipe.cook_time,
        difficulty: recipe.difficulty,
        steps: recipe.steps,
        cuisine: recipe.cuisine,
        meal_type: recipe.meal_type,
        full_calories,
        favorited,
    }))
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
struct Recipe {
    name: String,
    description: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
struct RecipeFull {
    name: String,
    description: String,
    prep_time: i32,
    cook_time: i32,
    difficulty: DifficultyLevel,
    steps: Vec<String>,
    cuisine: String,
    meal_type: TypeByTime,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, validator::Validate)]
struct InsertIngredient {
    #[validate(length(min = 2, message = "must be at least 2 character(s)"))]
    name: String,
    #[validate(length(min = 1, message = "must be at least 1 character(s)"))]
    quantity: String,
    quantity_unit: String,
}

#[tracing::instrument(skip(conn))]
async fn add_or_update_ingredient_to_recipe(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(name): Path<String>,
    Form(ingredient): Form<InsertIngredient>,
) -> Result<(), ApiError> {
    ingredient
        .validate()
        .map_err(ApiError::unprocessable_entity_from_validation_errors)?;

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

#[tracing::instrument(skip(conn, auth_user))]
async fn insert_full_recipe(
    Extension(channel): Extension<std::sync::Arc<tokio::sync::broadcast::Sender<String>>>,
    DatabaseConnection(mut conn): DatabaseConnection,
    // We want to accept Json input here instead of Form, because the structure
    // of `RecipeWithIngredients` is too complicated to handle with a form.
    auth_user: AuthUser,
    Json(recipe_with_ingredients): Json<RecipeWithIngredients>,
) -> Result<(), ApiError> {
    recipe_with_ingredients
        .validate()
        .map_err(ApiError::unprocessable_entity_from_validation_errors)?;

    let mut tx = conn.begin().await?;

    let RecipeWithIngredients {
        name,
        description,
        prep_time,
        cook_time,
        difficulty,
        steps,
        cuisine,
        meal_type,
        ingredients,
    } = recipe_with_ingredients;

    let recipe = sqlx::query!(
        r#"
        INSERT INTO recipes (
            "name",
            "description",
            "creator_id",
            "prep_time",
            "cook_time",
            "difficulty",
            "steps",
            "cuisine_id",
            "meal_type"
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, (SELECT id FROM cuisines WHERE name = $8), $9)
        RETURNING id;
        "#,
        name,
        description,
        *auth_user,
        prep_time,
        cook_time,
        difficulty as _,
        &steps,
        cuisine,
        meal_type as _,
    )
    .fetch_one(&mut tx)
    .await
    .on_code("23502", |_| {
        ApiError::unprocessable_entity([("cuisine", "does not exist")])
    })
    .on_constraint("recipes_name_key", |_| ApiError::Conflict)?;

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
        .execute(&mut tx)
        .await
        .map_err(|_| {
            ApiError::unprocessable_entity([(
                "ingredient-name",
                format!("{} is not an ingredient", ingredient.name),
            )])
        })?;
    }

    tx.commit().await?;

    channel
        .send("Someone just added a new recipe".into())
        .unwrap();
    Ok(())
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
struct RecipeWithIngredientCount {
    name: String,
    description: String,
    ingredient_count: Option<i64>,
}

#[tracing::instrument(skip_all)]
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

#[tracing::instrument(skip(conn, auth_user))]
async fn toggle_favorite_recipe(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(name): Path<String>,
    auth_user: AuthUser,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query!(
        // This is a helper function written in the `create_favorite_recipe` migration.
        // It helps to easily manage a 'toggle' functionality for marking favorites.
        "SELECT toggle_favorite_recipe($1, (SELECT id FROM recipes WHERE name = $2))",
        *auth_user,
        name,
    )
    .fetch_one(&mut conn)
    .await
    .map_err(|_| ApiError::BadRequest)?;

    if result.toggle_favorite_recipe == Some(BigDecimal::from(0)) {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::CREATED)
    }
}

#[tracing::instrument(skip_all)]
async fn my_favorite_recipes(
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
        INNER JOIN favorite_recipe fr ON fr.recipe_id = r.id AND fr.user_id = $1;
        "#,
        *auth_user
    )
    .fetch_all(&mut conn)
    .await?;

    Ok(Json(results))
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
struct RecipeWithFavoriteCount {
    name: String,
    count: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LimitedQuery {
    limit: Option<i64>,
}

#[tracing::instrument(skip(conn))]
async fn most_popular_recipes(
    DatabaseConnection(mut conn): DatabaseConnection,
    Query(query): Query<LimitedQuery>,
) -> Result<Json<Vec<RecipeWithFavoriteCount>>, ApiError> {
    let limit = match query.limit {
        Some(limit) if limit >= 0 => limit,
        _ => 10,
    };
    let results = sqlx::query_as!(
        RecipeWithFavoriteCount,
        r#"
        SELECT r.name, COUNT(fr.recipe_id) FROM recipes r
        INNER JOIN favorite_recipe fr ON r.id = fr.recipe_id
        GROUP BY r.name
        ORDER BY count DESC
        LIMIT $1;
        "#,
        limit
    )
    .fetch_all(&mut conn)
    .await?;

    Ok(Json(results))
}
