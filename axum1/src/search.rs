use meilisearch_sdk::client::Client;
use sqlx::{Pool, Postgres};

use crate::{config::Settings, queue::get_connection_pool, routes::ingredient::FoodCategory};

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

pub async fn run_meili_indexer_until_stopped(config: Settings) -> Result<(), anyhow::Error> {
    let meili_client = Client::new(config.meili.url, Some(config.meili.master_key));
    let pool = get_connection_pool(&config.database);
    run_meili_indexer(pool, meili_client).await
}

async fn run_meili_indexer(
    pool: Pool<Postgres>,
    meili_client: Client,
) -> Result<(), anyhow::Error> {
    loop {
        let ingredient_records = get_ingredient_records(&pool).await?;
        let cuisine_records = get_cuisine_records(&pool).await?;

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
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    }
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
