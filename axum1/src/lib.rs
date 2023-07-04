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
pub mod session;
pub mod session_ext;
pub mod sse;
pub mod startup;
pub mod state;
pub mod upload;
pub mod utils;
pub mod task; 

static RE_USERNAME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^[a-zA-Z0-9 íáéúőóüöűÍÁÉÚŐÓÜÖŰ](\.?[a-zA-Z0-9 íáéúőóüöűÍÁÉÚŐÓÜÖŰ])*$"#).unwrap()
});
static RE_RECIPE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^[a-zA-Z0-9-íáéúőóüöűÍÁÉÚŐÓÜÖŰ](\-?[a-zA-Z0-9 íáéúőóüöűÍÁÉÚŐÓÜÖŰ - \s])*$"#)
        .unwrap()
});
