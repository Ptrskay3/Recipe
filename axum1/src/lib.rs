use axum::async_trait;
use error::ApiError;
use once_cell::sync::Lazy;
use regex::Regex;
use sqlx::PgExecutor;

pub mod config;
pub mod error;
pub mod extractors;
pub mod queue;
pub mod routes;
pub mod search;
pub mod session;
pub mod startup;
pub mod utils;

static RE_USERNAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^[a-zA-Z0-9 áéúőóüöÁÉÚŐÓÜÖ](\.?[a-zA-Z0-9 áéúőóüöÁÉÚŐÓÜÖ])*$"#).unwrap());
static RE_RECIPE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^[a-zA-Z0-9-áéúőóüöÁÉÚŐÓÜÖ](\-?[a-zA-Z0-9 áéúőóüöÁÉÚŐÓÜÖ -,\s])*$"#).unwrap()
});

#[async_trait]
pub trait Queryable: Sized {
    type Id;
    type Name;

    async fn get_by_id<'c, E>(tx: E, id: Self::Id) -> Result<Self, ApiError>
    where
        E: PgExecutor<'c>;

    async fn get_by_name<'c, E>(tx: E, name: Self::Name) -> Result<Self, ApiError>
    where
        E: PgExecutor<'c>;
}
