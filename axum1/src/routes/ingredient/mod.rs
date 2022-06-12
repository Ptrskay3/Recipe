use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
// Because we need to deserialize a sequence from a form, we need `axum-extra`.
// See: https://github.com/tokio-rs/axum/pull/1031
use axum_extra::extract::Form;
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{PgHasArrayType, PgTypeInfo},
    Connection,
};

use crate::{
    error::ApiError,
    extractors::{AuthUser, DatabaseConnection},
};

pub fn ingredient_router() -> Router {
    Router::new()
        .route("/all", get(all_ingredients))
        .route("/category/:category", get(ingredients_by_category))
        .route("/new", post(add_ingredient))
        .route(
            "/:name",
            get(get_ingredient)
                .delete(delete_ingredient)
                .patch(upgrade_ingredient),
        )
        .route("/favorite/:name", post(make_favorite))
}

#[derive(sqlx::Type, Debug, Deserialize, Serialize)]
#[sqlx(rename_all = "snake_case", type_name = "food_category")]
#[serde(rename_all = "snake_case")]
enum FoodCategory {
    Vegetable,
    Fruit,
    Meat,
    Dairy,
    Grains,
    Legumes,
    Baked,
    Eggs,
    Seafood,
    NutsAndSeeds,
    HerbsAndSpices,
    Garnishes,
    DesertsAndSweets,
    Supplements,
    Beverage,
}

impl PgHasArrayType for FoodCategory {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_food_category")
    }
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
struct Ingredient {
    name: String,
    calories_per_100g: f32,
    category: Vec<FoodCategory>,
}

async fn all_ingredients(
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<Ingredient>>, ApiError> {
    let rows: Vec<_> = sqlx::query_as!(
        Ingredient,
        r#"
        SELECT name, calories_per_100g, category as "category: Vec<FoodCategory>" FROM ingredients;
        "#
    )
    .fetch_all(&mut conn)
    .await?;
    Ok(Json(rows))
}

async fn ingredients_by_category(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(category): Path<FoodCategory>,
) -> Result<Json<Vec<Ingredient>>, ApiError> {
    let rows: Vec<_> = sqlx::query_as!(
        Ingredient,
        r#"
        SELECT name, calories_per_100g, category as "category: Vec<FoodCategory>" FROM ingredients
        WHERE $1 = ANY (category);
        "#,
        category as _
    )
    .fetch_all(&mut conn)
    .await?;
    Ok(Json(rows))
}

async fn add_ingredient(
    DatabaseConnection(mut conn): DatabaseConnection,
    Form(ingredient): Form<Ingredient>,
    auth_user: Option<AuthUser>,
) -> Result<(), ApiError> {
    let creator_id = if let Some(uuid) = auth_user {
        Some(<sqlx::types::uuid::Uuid as From<_>>::from(uuid))
    } else {
        None
    };
    sqlx::query!(
        r#"
        INSERT INTO ingredients (name, category, calories_per_100g, creator_id)
        VALUES ($1, $2, $3, $4);
        "#,
        ingredient.name,
        ingredient.category as _,
        ingredient.calories_per_100g,
        creator_id
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
struct UpgradeIngredient {
    name: Option<String>,
    calories_per_100g: Option<f32>,
    category: Option<Vec<FoodCategory>>,
}

async fn upgrade_ingredient(
    Path(name): Path<String>,
    DatabaseConnection(mut conn): DatabaseConnection,
    Form(ingredient): Form<UpgradeIngredient>,
) -> Result<Json<Ingredient>, ApiError> {
    let mut tx = conn.begin().await?;
    let original = sqlx::query_as::<_, Ingredient>(
        "SELECT name, category, calories_per_100g FROM ingredients WHERE name = $1",
    )
    .bind(name.clone())
    .fetch_optional(&mut tx)
    .await?
    .ok_or(ApiError::NotFound)?;

    let row = sqlx::query_as!(
        Ingredient,
        r#"
        UPDATE ingredients
        SET name = $1,
            calories_per_100g = $2,
            category = $3
        WHERE name = $4
        RETURNING name, category as "category!: Vec<FoodCategory>", calories_per_100g
        "#,
        ingredient.name.unwrap_or(original.name),
        ingredient
            .calories_per_100g
            .unwrap_or(original.calories_per_100g),
        ingredient.category.unwrap_or(original.category) as _,
        name
    )
    .fetch_one(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(Json(row))
}

async fn get_ingredient(
    Path(name): Path<String>,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Ingredient>, ApiError> {
    let row = sqlx::query_as!(
        Ingredient,
        r#"
        SELECT name, category as "category: Vec<FoodCategory>", calories_per_100g
        FROM ingredients
        WHERE name = $1
        "#,
        name
    )
    .fetch_optional(&mut conn)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(row))
}

async fn delete_ingredient(
    Path(name): Path<String>,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Ingredient>, ApiError> {
    let row = sqlx::query_as!(
        Ingredient,
        r#"
        DELETE FROM ingredients
        WHERE name = $1
        RETURNING name, category as "category!: Vec<FoodCategory>", calories_per_100g
        "#,
        name
    )
    .fetch_optional(&mut conn)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(row))
}

#[derive(sqlx::FromRow)]
struct IngredientId {
    id: sqlx::types::uuid::Uuid,
}

async fn make_favorite(
    user_id: AuthUser,
    Path(name): Path<String>,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<(), ApiError> {
    // You can implement this either with a single query using Common Table Expressions (CTEs),
    // or multiple queries with a transaction.
    //
    // The former is likely more performant as it involves only a single round-trip to the database,
    // but the latter is more readable.
    //
    // It's generally a good idea to shoot for readability over raw performance for long-lived
    // projects. You don't want to come back later and be unable to understand what you wrote
    // because you were too clever. You can always improve performance later if the
    // implementation proves to be a bottleneck.
    //
    // Readability is also paramount if you need to onboard more devs to the project.

    // Begin a transaction so we have a consistent view of the database.
    // This has the side-effect of checking out a connection for the whole function,
    // which saves some overhead on subsequent queries.
    //
    // If an error occurs, this transaction will be rolled back on-drop.
    let mut tx = conn.begin().await?;

    let ingredient = sqlx::query_as!(
        IngredientId,
        r#"
        SELECT id
        FROM ingredients
        WHERE name = $1
        "#,
        name
    )
    .fetch_optional(&mut tx)
    .await?
    .ok_or(ApiError::NotFound)?;

    sqlx::query!(
        // If the row already exists, we don't need to do anything.
        r#"INSERT INTO favorite_ingredient(ingredient_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
        ingredient.id,
        <sqlx::types::uuid::Uuid as From<_>>::from(user_id)
    )
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
