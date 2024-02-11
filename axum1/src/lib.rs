use once_cell::sync::Lazy;
use regex::Regex;

pub mod cli;
pub mod config;
pub mod email;
pub mod error;
pub mod extractors;
pub mod queue;
pub mod routes;
pub mod search;
pub mod sse;
pub mod startup;
pub mod state;
pub mod task;
pub mod upload;
pub mod utils;

static RE_USERNAME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9 íáéúőóüöűÍÁÉÚŐÓÜÖŰ](\.?[a-zA-Z0-9 íáéúőóüöűÍÁÉÚŐÓÜÖŰ])*$").unwrap()
});

static RE_RECIPE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9-íáéúőóüöűÍÁÉÚŐÓÜÖŰ](\-?[a-zA-Z0-9 íáéúőóüöűÍÁÉÚŐÓÜÖŰ - \s])*$")
        .unwrap()
});
