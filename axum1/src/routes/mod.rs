mod admin;
mod auth;
mod ingredient;
mod recipe;

pub use ingredient::{ingredient_router, FoodCategory, Ingredient};
// TODO: don't use wildcard
pub use admin::*;
pub use auth::*;
pub use recipe::*;
