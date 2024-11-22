use serde::{Deserialize, Serialize};

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

#[derive(sqlx::Type, Debug, Deserialize, Serialize, Clone)]
#[sqlx(rename_all = "snake_case", type_name = "type_by_time")]
#[serde(rename_all = "snake_case")]
pub enum TypeByTime {
    Breakfast,
    Lunch,
    Dinner,
    Other,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
#[allow(non_camel_case_types)]
pub enum QuantityUnit {
    g,
    mg,
    kg,
    #[default]
    Empty,
}

impl<'a> TryFrom<&'a str> for QuantityUnit {
    type Error = validator::ValidationError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "g" | "G" => Ok(Self::g),
            "kg" | "Kg" | "KG" | "kG" => Ok(Self::kg),
            "mg" | "Mg" | "MG" | "mG" => Ok(Self::mg),
            other if other.trim() == "" => Ok(Self::Empty),
            other => {
                let mut err = validator::ValidationError::new("quantity_unit");
                err.message = Some(std::borrow::Cow::from(
                    "Cannot parse unit, only `g`, `mg` and `kg` are allowed.",
                ));
                err.add_param(std::borrow::Cow::from("quantity_unit"), &other);
                Err(err)
            }
        }
    }
}

impl QuantityUnit {
    pub fn get_multiplier_for_g(&self) -> f32 {
        match self {
            QuantityUnit::g | QuantityUnit::Empty => 1.0, // TODO: How do we handle empty?
            QuantityUnit::mg => 0.01,
            QuantityUnit::kg => 100.0,
        }
    }
}
