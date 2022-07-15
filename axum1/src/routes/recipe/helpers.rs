use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgHasArrayType, PgTypeInfo};

#[derive(sqlx::Type, Debug, Deserialize, Serialize, Clone)]
#[sqlx(rename_all = "snake_case", type_name = "difficulty_level")]
#[serde(rename_all = "snake_case")]
pub enum DifficultyLevel {
    Easy,
    Moderate,
    Medium,
    Challenging,
    Hard,
    Extreme,
    DoNotAttempt,
}

impl PgHasArrayType for DifficultyLevel {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_difficulty_level")
    }
}

#[derive(sqlx::Type, Debug, Deserialize, Serialize, Clone)]
#[sqlx(rename_all = "snake_case", type_name = "type_by_time")]
#[serde(rename_all = "snake_case")]
pub enum TypeByTime {
    Breakfast,
    Lunch,
    Dinner,
    Other,
}

impl PgHasArrayType for TypeByTime {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_type_by_time")
    }
}
