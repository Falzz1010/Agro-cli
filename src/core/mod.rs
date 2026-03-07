use std::collections::HashMap;
use tokio::fs;
use std::path::Path;

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

const RULES_PATH: &str = "plants.yaml";

/// Kesalahan yang dapat terjadi pada logika inti.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Kesalahan saat membaca file aturan.
    #[error("gagal membaca file aturan: {0}")]
    ReadError(#[from] std::io::Error),
    /// Kesalahan saat mengurai YAML aturan.
    #[error("gagal mengurai YAML aturan: {0}")]
    ParseError(#[from] serde_yaml::Error),
}

/// Mewakili jenis tindakan perawatan tanaman.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CareType {
    /// Menyiram tanaman.
    Water,
    /// Memupuk tanaman.
    Fertilizer,
}

impl CareType {
    /// Mengembalikan nama kolom database untuk jenis perawatan ini.
    #[must_use]
    pub fn column_name(self) -> &'static str {
        match self {
            CareType::Water => "last_watered",
            CareType::Fertilizer => "last_fertilized",
        }
    }
}

/// Mengambil kondisi cuaca dan suhu saat ini untuk sebuah kota.
///
/// # Errors
///
/// Mengembalikan kesalahan jika permintaan jaringan gagal, API key tidak valid, atau isi respons cacat.
pub async fn weather(city: &str, api_key: &str) -> anyhow::Result<(String, f32)> {
    let url = format!(
        "http://api.openweathermap.org/data/2.5/weather?q={city}&appid={api_key}&units=metric"
    );

    let response = reqwest::get(&url).await?;
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        let weather = json["weather"][0]["main"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing weather main field"))?
            .to_string();
        #[allow(clippy::cast_possible_truncation)]
        let temp = json["main"]["temp"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Missing temperature field"))? as f32;
        Ok((weather, temp))
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(anyhow::anyhow!("Weather API error {status}: {body}"))
    }
}

/// Menentukan persyaratan perawatan untuk jenis tanaman tertentu.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlantRule {
    /// Interval yang disarankan antara penyiraman dalam hari.
    pub water_interval_days: Option<i64>,
    /// Interval yang disarankan antara pemupukan dalam hari.
    pub fertilizer_interval_days: Option<i64>,
    /// Jumlah air dalam mililiter.
    pub water_ml: Option<i32>,
    /// Jam sinar matahari yang disarankan.
    pub sun_hours: Option<i32>,
    /// Tingkat kelembaban tanah minimum (0-100%).
    pub min_moisture_level: Option<f32>,
}

/// Mewakili satu tanaman di taman.
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
#[allow(clippy::struct_field_names)]
pub struct Plant {
    /// Nama panggilan unik tanaman.
    pub name: String,
    /// Jenis tanaman (harus sesuai dengan kunci di `plants.yaml`).
    pub plant_type: String,
    /// Tanggal tanaman ditambahkan ke taman.
    pub planted_date: String,
    /// Tanggal terakhir tanaman disiram.
    pub last_watered: String,
    /// Tanggal terakhir tanaman dipupuk.
    pub last_fertilized: String,
    /// Tingkat kelembaban minimum kustom.
    pub min_moisture: Option<f32>,
    /// Jumlah air kustom dalam ml.
    pub water_ml: Option<i32>,
}

/// Tugas perawatan spesifik yang dihasilkan untuk tanaman.
#[derive(Debug, Serialize, Deserialize)]
pub struct GardenTask {
    /// Nama panggilan tanaman.
    pub name: String,
    /// Jenis tanaman.
    pub plant_type: String,
    /// Apakah tanaman perlu disiram.
    pub needs_water: bool,
    /// Apakah penyiraman dilewati karena hujan.
    pub skip_watering_due_to_weather: bool,
    /// Jumlah air yang dibutuhkan dalam mililiter.
    pub water_ml: i32,
    /// Apakah tanaman perlu dipupuk.
    pub needs_fertilizer: bool,
    /// Sinar matahari yang dibutuhkan untuk hari ini.
    pub sun_hours: i32,
}

/// Memuat aturan perawatan tanaman dari file `plants.yaml`.
///
/// # Errors
///
/// Mengembalikan kesalahan jika file ada tetapi tidak dapat dibaca atau diurai sebagai YAML yang valid.
pub async fn load_rules() -> Result<HashMap<String, PlantRule>, CoreError> {
    if !Path::new(RULES_PATH).exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(RULES_PATH).await?;
    let rules: HashMap<String, PlantRule> = serde_yaml::from_str(&content)?;
    Ok(rules)
}

/// Menghitung tugas perawatan yang diperlukan untuk hari ini berdasarkan status tanaman dan cuaca.
///
/// # Argumen
/// * `active_plants` - Slice tanaman aktif yang akan dievaluasi.
/// * `weather_condition` - Kondisi cuaca saat ini yang opsional (misalnya, "Rain").
/// * `real_time_moisture` - Pembacaan kelembaban tanah saat ini yang opsional.
///
/// # Returns
/// Vektor objek `GardenTask` yang mewakili tindakan yang diperlukan hari ini.
pub async fn calculate_today_tasks(
    active_plants: &[Plant],
    weather_condition: Option<&str>,
    real_time_moisture: Option<f32>,
) -> Vec<GardenTask> {
    let rules = match load_rules().await {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to load plant rules: {e}");
            return Vec::new();
        }
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

    #[tokio::test]
    async fn calculate_today_tasks_returns_empty_when_no_plants() {
        let tasks = calculate_today_tasks(&[], None, None).await;
        assert!(tasks.is_empty(), "Expected no tasks for empty plant list");
    }

    #[tokio::test]
    async fn calculate_today_tasks_returns_empty_for_unknown_plant_type() {
        let plants = vec![make_plant("TestPlant", "unknown_type_xyz")];
        let tasks = calculate_today_tasks(&plants, None, None).await;
        assert!(
            tasks.is_empty(),
            "Expected no tasks for unknown plant type"
        );
    }

    #[tokio::test]
    async fn load_rules_returns_ok() {
        let result = load_rules().await;
        assert!(result.is_ok(), "load_rules should not error");
    }

    #[test]
    fn care_type_column_name_returns_correct_value() {
        assert_eq!(CareType::Water.column_name(), "last_watered");
        assert_eq!(CareType::Fertilizer.column_name(), "last_fertilized");
    }
}
