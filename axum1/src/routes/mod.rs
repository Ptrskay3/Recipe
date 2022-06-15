mod auth;
mod ingredient;

pub use ingredient::{ingredient_router, Ingredient};
// TODO: don't use wildcard
pub use auth::*;
