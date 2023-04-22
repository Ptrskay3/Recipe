use meilisearch_sdk::client::Client;
use sqlx::{Pool, Postgres};

use crate::{config::Settings, queue::get_connection_pool, routes::ingredient::FoodCategory};

pub async fn run_meili_indexer_until_stopped(config: Settings) -> Result<(), anyhow::Error> {
    let meili_client = Client::new(config.meili.url, Some(config.meili.master_key));
    let pool = get_connection_pool(&config.database);
    // TODO: These defaults are hidden here, maybe there's a better place for them?
    let retry_seconds = config.meili.retry_seconds.unwrap_or(60);
    let max_retries = config.meili.max_retries.unwrap_or(5);
    let indexing_interval_seconds = config.meili.indexing_interval_seconds.unwrap_or(3600);
    let mut current_retries = 0;
    // I don't really know whether this is a good idea yet. Maybe the whole MeiliSearch indexing should be its own crate.
    loop {
        match run_meili_indexer(&pool, &meili_client).await {
            Ok(_) => {
                tokio::time::sleep(std::time::Duration::from_secs(indexing_interval_seconds)).await;
            }
            Err(e) => {
                if max_retries > current_retries {
                    current_retries += 1;
                    let left = max_retries - current_retries;
                    tracing::error!(error.message = %e, "Failed search indexing, retrying in {retry_seconds} seconds ({left} retries left).");
                    tokio::time::sleep(std::time::Duration::from_secs(retry_seconds)).await;
                    continue;
                } else {
                    return Err(e);
                }
            }
        }
    }
}

pub async fn run_meili_indexer(
    pool: &Pool<Postgres>,
    meili_client: &Client,
) -> Result<(), anyhow::Error> {
    // TODO: This is getting repetitive
    let ingredient_records = get_ingredient_records(&pool).await?;
    let cuisine_records = get_cuisine_records(&pool).await?;
    let recipe_records = get_recipe_records(&pool).await?;

    tracing::info!("Started indexing ingredients");
    let task = meili_client
        .index("ingredients")
        .add_documents(&ingredient_records, None)
        .await?
        .wait_for_completion(&meili_client, None, None)
        .await?;
    tracing::info!("indexing finished, success: {}", task.is_success());

    tracing::info!("Started indexing cuisines");
    let task = meili_client
        .index("cuisines")
        .add_documents(&cuisine_records, None)
        .await?
        .wait_for_completion(&meili_client, None, None)
        .await?;
    tracing::info!("indexing finished, success: {}", task.is_success());

    tracing::info!("Started indexing recipes");
    let task = meili_client
        .index("recipes")
        .add_documents(&recipe_records, None)
        .await?
        .wait_for_completion(&meili_client, None, None)
        .await?;
    tracing::info!("indexing finished, success: {}", task.is_success());

    Ok(())
}

async fn get_ingredient_records(pool: &Pool<Postgres>) -> anyhow::Result<Vec<Ingredient>> {
    let mut tx = pool.begin().await?;
    let records = sqlx::query_as!(
        Ingredient,
        r#"
        SELECT id, name, calories_per_100g, category as "category: Vec<FoodCategory>", g_per_piece,
         protein, water, fat, sugar, carbohydrate, fiber, caffeine, contains_alcohol
        FROM ingredients
        "#
    )
    .fetch_all(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(records)
}

async fn get_recipe_records(pool: &Pool<Postgres>) -> anyhow::Result<Vec<RecipeSearchSimple>> {
    let mut tx = pool.begin().await?;
    let records = sqlx::query_as!(
        RecipeSearchSimple,
        r#"
        SELECT id, name, description FROM recipes
        "#
    )
    .fetch_all(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(records)
}

async fn get_cuisine_records(pool: &Pool<Postgres>) -> anyhow::Result<Vec<Cuisine>> {
    let mut tx = pool.begin().await?;
    let records = sqlx::query_as!(
        Cuisine,
        r#"
        SELECT id, name FROM cuisines;
        "#
    )
    .fetch_all(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(records)
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct Cuisine {
    name: String,
    id: uuid::Uuid,
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct RecipeSearchSimple {
    id: uuid::Uuid,
    name: String,
    description: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Ingredient {
    id: uuid::Uuid,
    name: String,
    calories_per_100g: f32,
    category: Vec<FoodCategory>,
    g_per_piece: Option<f32>,
    protein: f32,
    water: f32,
    fat: f32,
    sugar: f32,
    carbohydrate: f32,
    fiber: f32,
    caffeine: f32,
    contains_alcohol: bool,
}
