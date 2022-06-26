use axum::async_trait;
use error::ApiError;
use sqlx::PgExecutor;

pub mod config;
pub mod error;
pub mod extractors;
pub mod queue;
pub mod routes;
pub mod session;
pub mod startup;
pub mod utils;

pub const AXUM_SESSION_COOKIE_NAME: &str = "axum_session";

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
