use axum::async_trait;
use error::ApiError;
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

lazy_static::lazy_static! {
    static ref RE_USERNAME: Regex =
        Regex::new(r#"^[a-z0-9](\.?[a-z0-9])*$"#).unwrap();
}

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
