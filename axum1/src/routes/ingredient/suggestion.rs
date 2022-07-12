use axum::{extract::Path, Json};

use crate::{
    error::ApiError,
    extractors::{AuthUser, DatabaseConnection},
    routes::AdminUser,
};

use super::{FoodCategory, UpgradeIngredient};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct IngredientSuggestion {
    is_delete_vote: Option<bool>,
    update_ingredient: Option<UpgradeIngredient>,
}

impl IngredientSuggestion {
    pub fn is_irrelevant(&self) -> bool {
        self.is_delete_vote.is_none() && self.update_ingredient.is_none()
    }
}

pub async fn add_ingredient_suggestion(
    DatabaseConnection(mut conn): DatabaseConnection,
    Json(ingredient_suggestion): Json<IngredientSuggestion>,
    Path(name): Path<String>,
    auth_user: AuthUser,
) -> Result<(), ApiError> {
    if ingredient_suggestion.is_irrelevant() {
        return Err(ApiError::BadRequest);
    }

    let update_ingredient = ingredient_suggestion.update_ingredient.unwrap_or_default();
    sqlx::query!(
        r#"
        INSERT INTO ingredient_suggestions (
            ingredient_id,
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
            user_id,
            is_delete_vote
        )
        VALUES ((SELECT id FROM ingredients WHERE name = $1), $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15);
        "#,
        name,
        update_ingredient.name,
        update_ingredient.category as _,
        update_ingredient.calories_per_100g,
        update_ingredient.g_per_piece.unwrap_or(None),
        update_ingredient.protein,
        update_ingredient.water,
        update_ingredient.fat,
        update_ingredient.sugar,
        update_ingredient.carbohydrate,
        update_ingredient.fiber,
        update_ingredient.caffeine,
        update_ingredient.contains_alcohol,
        *auth_user,
        ingredient_suggestion.is_delete_vote,
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct SuggestedIngredient {
    id: uuid::Uuid,
    name: Option<String>,
    calories_per_100g: Option<f32>,
    category: Option<Vec<FoodCategory>>,
    g_per_piece: Option<f32>,
    protein: Option<f32>,
    water: Option<f32>,
    fat: Option<f32>,
    sugar: Option<f32>,
    carbohydrate: Option<f32>,
    fiber: Option<f32>,
    caffeine: Option<f32>,
    contains_alcohol: Option<bool>,
    is_delete_vote: Option<bool>,
    suggester: String,
}

pub async fn get_ingredient_suggestions(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(name): Path<String>,
    _user: AdminUser, // FIXME: this is just used as a request guard, so maybe a `route_layer` would suffice..
) -> Result<Json<Vec<SuggestedIngredient>>, ApiError> {
    let suggestions: Vec<_> = sqlx::query_as!(
        SuggestedIngredient,
        r#"
        SELECT
            igs.id,
            COALESCE(igs.name, i.name) AS name,
            COALESCE(igs.category, i.category) AS "category: Vec<FoodCategory>",
            COALESCE(igs.calories_per_100g, i.calories_per_100g) AS calories_per_100g,
            COALESCE(igs.g_per_piece, i.g_per_piece) AS g_per_piece,
            COALESCE(igs.protein, i.protein) AS protein,
            COALESCE(igs.water, i.water) AS water,
            COALESCE(igs.fat, i.fat) AS fat,
            COALESCE(igs.sugar, i.sugar) AS sugar,
            COALESCE(igs.carbohydrate, i.carbohydrate) AS carbohydrate,
            COALESCE(igs.fiber, i.fiber) AS fiber,
            COALESCE(igs.caffeine, i.caffeine) AS caffeine,
            COALESCE(igs.contains_alcohol, i.contains_alcohol) AS contains_alcohol,
            u.name as suggester,
            is_delete_vote
            FROM ingredient_suggestions igs 
        INNER JOIN ingredients i ON igs.ingredient_id = i.id
        INNER JOIN users u ON u.user_id = igs.user_id
        WHERE ingredient_id = (SELECT id FROM ingredients WHERE name = $1)
        "#,
        name
    )
    .fetch_all(&mut conn)
    .await?;
    Ok(Json(suggestions))
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct Suggestion {
    name: Option<String>,
    calories_per_100g: Option<f32>,
    category: Option<Vec<FoodCategory>>,
    g_per_piece: Option<f32>,
    protein: Option<f32>,
    water: Option<f32>,
    fat: Option<f32>,
    sugar: Option<f32>,
    carbohydrate: Option<f32>,
    fiber: Option<f32>,
    caffeine: Option<f32>,
    contains_alcohol: Option<bool>,
    is_delete_vote: Option<bool>,
}

pub async fn get_ingredient_suggestion(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path((name, id)): Path<(String, uuid::Uuid)>,
) -> Result<Json<Suggestion>, ApiError> {
    let suggestion = sqlx::query_as!(
        Suggestion,
        r#"
        SELECT
            COALESCE(igs.name, i.name) AS name,
            COALESCE(igs.category, i.category) AS "category: Vec<FoodCategory>",
            COALESCE(igs.calories_per_100g, i.calories_per_100g) AS calories_per_100g,
            COALESCE(igs.g_per_piece, i.g_per_piece) AS g_per_piece,
            COALESCE(igs.protein, i.protein) AS protein,
            COALESCE(igs.water, i.water) AS water,
            COALESCE(igs.fat, i.fat) AS fat,
            COALESCE(igs.sugar, i.sugar) AS sugar,
            COALESCE(igs.carbohydrate, i.carbohydrate) AS carbohydrate,
            COALESCE(igs.fiber, i.fiber) AS fiber,
            COALESCE(igs.caffeine, i.caffeine) AS caffeine,
            COALESCE(igs.contains_alcohol, i.contains_alcohol) AS contains_alcohol,
            is_delete_vote
        FROM ingredient_suggestions igs
        INNER JOIN ingredients i ON igs.ingredient_id = i.id
        WHERE i.name = $1 AND igs.id = $2;
        "#,
        name,
        id
    )
    .fetch_optional(&mut conn)
    .await?
    .ok_or(ApiError::NotFound)?;
    Ok(Json(suggestion))
}
