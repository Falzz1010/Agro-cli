use std::collections::HashMap;
use std::fs;
use std::path::Path;

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const RULES_PATH: &str = "plants.yaml";

/// Errors that can occur in the core logic.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Error when reading the rules file.
    #[error("failed to read rules file: {0}")]
    ReadError(#[from] std::io::Error),
    /// Error when parsing the rules YAML.
    #[error("failed to parse rules YAML: {0}")]
    ParseError(#[from] serde_yaml::Error),
}

/// Represents a type of plant care action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CareType {
    /// Watering the plant.
    Water,
    /// Fertilizing the plant.
    Fertilizer,
}

impl CareType {
    /// Returns the database column name for this care type.
    pub fn column_name(self) -> &'static str {
        match self {
            CareType::Water => "last_watered",
            CareType::Fertilizer => "last_fertilized",
        }
    }
}

/// Fetches the current weather condition and temperature for a city.
///
/// Returns `Some((condition, temp))` on success.
pub async fn weather(city: &str, api_key: &str) -> Option<(String, f32)> {
    let url = format!(
        "http://api.openweathermap.org/data/2.5/weather?q={city}&appid={api_key}&units=metric"
    );

    let response = reqwest::get(&url).await.ok()?;
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await.ok()?;
        let weather = json["weather"][0]["main"].as_str()?.to_string();
        #[allow(clippy::cast_possible_truncation)]
        let temp = json["main"]["temp"].as_f64()? as f32;
        Some((weather, temp))
    } else {
        None
    }
}

/// Defines the care requirements for a specific plant type.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlantRule {
    /// Recommended interval between waterings in days.
    pub water_interval_days: Option<i64>,
    /// Recommended interval between fertilizing in days.
    pub fertilizer_interval_days: Option<i64>,
    /// Amount of water in milliliters.
    pub water_ml: Option<i32>,
    /// Recommended hours of sunlight.
    pub sun_hours: Option<i32>,
    /// Minimum soil moisture level (0-100%).
    pub min_moisture_level: Option<f32>,
}

/// Represents a single plant in the garden.
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
#[allow(clippy::struct_field_names)]
pub struct Plant {
    /// Unique nickname of the plant.
    pub name: String,
    /// The type of plant (must match a key in `plants.yaml`).
    pub plant_type: String,
    /// The date the plant was added to the garden.
    pub planted_date: String,
    /// The date the plant was last watered.
    pub last_watered: String,
    /// The date the plant was last fertilized.
    pub last_fertilized: String,
    /// Custom minimum moisture level.
    pub min_moisture: Option<f32>,
    /// Custom water amount in ml.
    pub water_ml: Option<i32>,
}

/// A specific care task generated for a plant.
#[derive(Debug, Serialize, Deserialize)]
pub struct GardenTask {
    /// Nickname of the plant.
    pub name: String,
    /// The type of plant.
    pub plant_type: String,
    /// Whether the plant needs watering.
    pub needs_water: bool,
    /// Whether watering was skipped due to rain.
    pub skip_watering_due_to_weather: bool,
    /// Amount of water needed in milliliters.
    pub water_ml: i32,
    /// Whether the plant needs fertilizer.
    pub needs_fertilizer: bool,
    /// Required sunlight for the day.
    pub sun_hours: i32,
}

/// Loads the plant care rules from the `plants.yaml` file.
pub fn load_rules() -> Result<HashMap<String, PlantRule>, CoreError> {
    if !Path::new(RULES_PATH).exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(RULES_PATH)?;
    let rules: HashMap<String, PlantRule> = serde_yaml::from_str(&content)?;
    Ok(rules)
}

/// Calculates the care tasks needed for today based on plant state and weather.
///
/// # Arguments
/// * `active_plants` - A slice of currently active plants to evaluate.
/// * `weather_condition` - Optional current weather condition (e.g., "Rain").
/// * `real_time_moisture` - Optional current soil moisture reading.
///
/// # Returns
/// A vector of `GardenTask` objects representing actions required today.
pub fn calculate_today_tasks(
    active_plants: &[Plant],
    weather_condition: Option<&str>,
    real_time_moisture: Option<f32>,
) -> Vec<GardenTask> {
    let Ok(rules) = load_rules() else {
        return Vec::new();
    };

    let mut tasks = Vec::new();
    let today = Local::now().date_naive();

    let rain_keywords = ["rain", "drizzle", "thunderstorm", "shower"];
    let mut skip_watering_due_to_weather = false;

    if let Some(weather) = weather_condition {
        let weather_lower = weather.to_lowercase();
        if rain_keywords.iter().any(|&kw| weather_lower.contains(kw)) {
            skip_watering_due_to_weather = true;
        }
    }

    for plant in active_plants {
        let plant_type_lower = plant.plant_type.to_lowercase();
        let Some(rule) = rules.get(&plant_type_lower) else {
            continue;
        };

        let mut needs_water = false;

        // Determine effective moisture threshold (plant setting > rule)
        let min_moisture_threshold = plant.min_moisture.or(rule.min_moisture_level);
        let effective_water_ml = plant.water_ml.unwrap_or_else(|| rule.water_ml.unwrap_or(0));

        // Moisture override
        if let Some(moisture) = real_time_moisture {
            if min_moisture_threshold.is_some_and(|min_m| moisture < min_m) {
                needs_water = true;
            }
        } else {
            // Fallback to date-based
            if let Ok(last_watered_date) =
                NaiveDate::parse_from_str(&plant.last_watered, "%Y-%m-%d")
            {
                let days_since = (today - last_watered_date).num_days();
                let interval = rule.water_interval_days.unwrap_or(1);
                if days_since >= interval {
                    needs_water = true;
                }
            }
        }

        // Weather override
        if needs_water && skip_watering_due_to_weather {
            needs_water = false;
        }

        // Fertilizer
        let mut needs_fertilizer = false;
        if let Ok(last_fert_date) = NaiveDate::parse_from_str(&plant.last_fertilized, "%Y-%m-%d") {
            let days_since_fert = (today - last_fert_date).num_days();
            let fert_interval = rule.fertilizer_interval_days.unwrap_or(14);
            if days_since_fert >= fert_interval {
                needs_fertilizer = true;
            }
        }

        if needs_water || needs_fertilizer {
            tasks.push(GardenTask {
                name: plant.name.clone(),
                plant_type: plant.plant_type.clone(),
                needs_water,
                skip_watering_due_to_weather,
                water_ml: effective_water_ml,
                needs_fertilizer,
                sun_hours: rule.sun_hours.unwrap_or(0),
            });
        }
    }

    tasks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_plant(name: &str, plant_type: &str) -> Plant {
        let today = Local::now().format("%Y-%m-%d").to_string();
        Plant {
            name: name.to_string(),
            plant_type: plant_type.to_string(),
            planted_date: today.clone(),
            last_watered: today.clone(),
            last_fertilized: today,
            min_moisture: None,
            water_ml: None,
        }
    }

    #[test]
    fn calculate_today_tasks_returns_empty_when_no_plants() {
        let tasks = calculate_today_tasks(&[], None, None);
        assert!(tasks.is_empty(), "Expected no tasks for empty plant list");
    }

    #[test]
    fn calculate_today_tasks_returns_empty_for_unknown_plant_type() {
        let plants = vec![make_plant("TestPlant", "unknown_type_xyz")];
        let tasks = calculate_today_tasks(&plants, None, None);
        assert!(
            tasks.is_empty(),
            "Expected no tasks for unknown plant type"
        );
    }

    #[test]
    fn load_rules_returns_ok() {
        let result = load_rules();
        assert!(result.is_ok(), "load_rules should not error");
    }

    #[test]
    fn care_type_column_name_returns_correct_value() {
        assert_eq!(CareType::Water.column_name(), "last_watered");
        assert_eq!(CareType::Fertilizer.column_name(), "last_fertilized");
    }
}
