use axum::{
    async_trait,
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
    Connection, PgExecutor,
};

use crate::{
    error::ApiError,
    extractors::{AuthUser, DatabaseConnection},
    Queryable,
};

pub mod suggestion;
use suggestion::add_ingredient_suggestion;

use self::suggestion::{get_ingredient_suggestion, get_ingredient_suggestions};

#[must_use]
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
        .route("/favorite/:name", post(make_favorite)) // TODO: swap route to `/:name/favorite` maybe for consistency?
        .route("/:name/suggestion", post(add_ingredient_suggestion))
        .route("/:name/suggestion/:id", get(get_ingredient_suggestion))
        .route("/:name/suggestions", get(get_ingredient_suggestions))
}

#[derive(sqlx::Type, Debug, Deserialize, Serialize, Clone)]
#[sqlx(rename_all = "snake_case", type_name = "food_category")]
#[serde(rename_all = "snake_case")]
pub enum FoodCategory {
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
    Uncategorized,
}

impl PgHasArrayType for FoodCategory {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_food_category")
    }
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct Ingredient {
    pub name: String,
    pub calories_per_100g: f32,
    pub category: Vec<FoodCategory>,
    pub g_per_piece: Option<f32>,
    pub protein: f32,
    pub water: f32,
    pub fat: f32,
    pub sugar: f32,
    pub carbohydrate: f32,
    pub fiber: f32,
    pub caffeine: f32,
    pub contains_alcohol: bool,
}

#[async_trait]
impl Queryable for Ingredient {
    type Id = uuid::Uuid;
    type Name = String;

    async fn get_by_id<'c, E>(e: E, id: Self::Id) -> Result<Self, ApiError>
    where
        E: PgExecutor<'c>,
    {
        let query = sqlx::query_as!(
            Self,
            r#"
            SELECT name, calories_per_100g, category as "category: Vec<FoodCategory>", g_per_piece,
            protein, water, fat, sugar, carbohydrate, fiber, caffeine, contains_alcohol
            FROM ingredients
            WHERE id = $1;
            "#,
            id
        )
        .fetch_optional(e)
        .await?
        .ok_or(ApiError::NotFound)?;
        Ok(query)
    }

    async fn get_by_name<'c, E>(e: E, name: Self::Name) -> Result<Self, ApiError>
    where
        E: PgExecutor<'c>,
    {
        let query = sqlx::query_as!(
            Self,
            r#"
            SELECT name, calories_per_100g, category as "category: Vec<FoodCategory>", g_per_piece,
            protein, water, fat, sugar, carbohydrate, fiber, caffeine, contains_alcohol
            FROM ingredients
            WHERE name = $1;
            "#,
            name
        )
        .fetch_optional(e)
        .await?
        .ok_or(ApiError::NotFound)?;
        Ok(query)
    }
}

async fn all_ingredients(
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<Ingredient>>, ApiError> {
    let rows: Vec<_> = sqlx::query_as!(
        Ingredient,
        r#"
        SELECT name, calories_per_100g, category as "category: Vec<FoodCategory>", g_per_piece,
        protein, water, fat, sugar, carbohydrate, fiber, caffeine, contains_alcohol
        FROM ingredients;
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
        SELECT name, calories_per_100g, category as "category: Vec<FoodCategory>", g_per_piece,
        protein, water, fat, sugar, carbohydrate, fiber, caffeine, contains_alcohol
        FROM ingredients
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
    let creator_id = auth_user.map(|u| *u);
    sqlx::query!(
        r#"
        INSERT INTO ingredients (
            name,
            category,
            calories_per_100g,
            g_per_piece,
            protein,
            water,
            fat,
            sugar,
            carbohydrate,
            fiber,
            caffeine,
            contains_alcohol,
            creator_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13);
        "#,
        ingredient.name,
        ingredient.category as _,
        ingredient.calories_per_100g,
        ingredient.g_per_piece,
        ingredient.protein,
        ingredient.water,
        ingredient.fat,
        ingredient.sugar,
        ingredient.carbohydrate,
        ingredient.fiber,
        ingredient.caffeine,
        ingredient.contains_alcohol,
        creator_id
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize, Clone, Default)]
pub struct UpgradeIngredient {
    name: Option<String>,
    calories_per_100g: Option<f32>,
    category: Option<Vec<FoodCategory>>,
    g_per_piece: Option<Option<f32>>,
    protein: Option<f32>,
    water: Option<f32>,
    fat: Option<f32>,
    sugar: Option<f32>,
    carbohydrate: Option<f32>,
    fiber: Option<f32>,
    caffeine: Option<f32>,
    contains_alcohol: Option<bool>,
}

async fn upgrade_ingredient(
    Path(name): Path<String>,
    DatabaseConnection(mut conn): DatabaseConnection,
    Form(ingredient): Form<UpgradeIngredient>,
) -> Result<Json<Ingredient>, ApiError> {
    let mut tx = conn.begin().await?;
    let original = sqlx::query_as::<_, Ingredient>(
        "SELECT name, category, calories_per_100g, g_per_piece,
        protein, water, fat, sugar, carbohydrate, fiber, caffeine, contains_alcohol
        FROM ingredients WHERE name = $1",
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
            category = $3,
            g_per_piece = $4,
            protein = $5,
            water = $6,
            fat = $7,
            sugar = $8,
            carbohydrate = $9,
            fiber = $10,
            caffeine = $11,
            contains_alcohol = $12
        WHERE name = $13
        RETURNING name, category as "category!: Vec<FoodCategory>", calories_per_100g, g_per_piece,
                  protein, water, fat, sugar, carbohydrate, fiber, caffeine, contains_alcohol;
        "#,
        ingredient.name.unwrap_or(original.name),
        ingredient
            .calories_per_100g
            .unwrap_or(original.calories_per_100g),
        ingredient.category.unwrap_or(original.category) as _,
        ingredient.g_per_piece.unwrap_or(original.g_per_piece),
        ingredient.protein.unwrap_or(original.protein),
        ingredient.water.unwrap_or(original.water),
        ingredient.fat.unwrap_or(original.fat),
        ingredient.sugar.unwrap_or(original.sugar),
        ingredient.carbohydrate.unwrap_or(original.carbohydrate),
        ingredient.fiber.unwrap_or(original.fiber),
        ingredient.caffeine.unwrap_or(original.caffeine),
        ingredient
            .contains_alcohol
            .unwrap_or(original.contains_alcohol),
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
    let row = Ingredient::get_by_name(&mut conn, name).await?;

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
        RETURNING name, category as "category!: Vec<FoodCategory>", calories_per_100g, g_per_piece,
                  protein, water, fat, sugar, carbohydrate, fiber, caffeine, contains_alcohol;
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
    auth_user: AuthUser,
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
        *auth_user
    )
    .execute(&mut tx)
    .await?;

    // Don't forget to commit to actually run those queries against the database.
    tx.commit().await?;

    Ok(())
}
