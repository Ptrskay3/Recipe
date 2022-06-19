mod admin;
mod auth;
mod ingredient;

pub use ingredient::{ingredient_router, Ingredient};
// TODO: don't use wildcard
pub use admin::*;
pub use auth::*;
