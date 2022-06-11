use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
// https://github.com/tokio-rs/axum/pull/1031
use axum_extra::extract::Form;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgHasArrayType, PgTypeInfo};

use crate::{
    error::ApiError,
    extractors::{AuthUser, DatabaseConnection}, 
};

pub fn ingredient_router() -> Router {
    Router::new()
        .route("/all", get(all_ingredients))
        .route("/category/:category", get(ingredients_by_category))
        .route("/new", post(add_ingredient))
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
