use axum::async_trait;
use error::ApiError;
use once_cell::sync::Lazy;
use regex::Regex;
use sqlx::PgExecutor;

pub mod config;
pub mod email;
pub mod error;
pub mod extractors;
pub mod queue;
pub mod routes;
pub mod search;
pub mod session;
pub mod session_ext;
pub mod startup;
pub mod upload;
pub mod utils;

static RE_USERNAME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^[a-zA-Z0-9 íáéúőóüöűÍÁÉÚŐÓÜÖŰ](\.?[a-zA-Z0-9 íáéúőóüöűÍÁÉÚŐÓÜÖŰ])*$"#).unwrap()
});
static RE_RECIPE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^[a-zA-Z0-9-íáéúőóüöűÍÁÉÚŐÓÜÖŰ](\-?[a-zA-Z0-9 íáéúőóüöűÍÁÉÚŐÓÜÖŰ - \s])*$"#)
        .unwrap()
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
