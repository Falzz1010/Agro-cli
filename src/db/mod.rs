use chrono::Utc;
use sqlx::{Pool, Sqlite, sqlite::SqlitePool};
use thiserror::Error;

use crate::core::{CareType, Plant};

/// Kesalahan yang dapat terjadi selama operasi database.
#[derive(Debug, Error)]
pub enum DbError {
    /// Kesalahan dari pool `SQLx` atau kueri yang mendasarinya.
    #[error("kegagalan kueri basis data: {0}")]
    Sqlx(#[from] sqlx::Error),
    /// Kesalahan saat tanaman tidak ditemukan.
    #[error("tanaman '{0}' tidak ditemukan")]
    PlantNotFound(String),
}

/// Menangani semua logika persistensi untuk taman `AgroCLI`.
#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    /// Membuat instance Database baru dan terhubung ke pool `SQLite`.
    ///
    /// # Argumen
    /// * `database_url` - String koneksi untuk basis data `SQLite`.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError` jika koneksi gagal atau URL tidak valid.
    pub async fn new(database_url: &str) -> Result<Self, DbError> {
        use std::str::FromStr;
        let options = sqlx::sqlite::SqliteConnectOptions::from_str(database_url)?
            .pragma("journal_mode", "WAL")
            .pragma("synchronous", "NORMAL")
            .pragma("temp_store", "MEMORY")
            .pragma("cache_size", "-20000") // 20MB cache
            .busy_timeout(std::time::Duration::from_secs(5));

        let pool = SqlitePool::connect_with(options).await?;
        Ok(Self { pool })
    }

    /// Menginisialisasi tabel database jika belum ada.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError` jika pembuatan tabel atau indeks gagal.
    pub async fn init(&self) -> Result<(), DbError> {
        // Core table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS plants (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                plant_type TEXT NOT NULL,
                planted_date TEXT NOT NULL,
                last_watered TEXT NOT NULL,
                last_fertilized TEXT NOT NULL,
                min_moisture REAL,
                water_ml INTEGER,
                status TEXT NOT NULL DEFAULT 'active'
            )",
        )
        .execute(&self.pool)
        .await?;

        // Migration: Add min_moisture if missing
        // SQLite doesn't easily support ADD COLUMN IF NOT EXISTS.
        // We ignore errors here as the columns might already exist.
        let _ = sqlx::query("ALTER TABLE plants ADD COLUMN min_moisture REAL")
            .execute(&self.pool)
            .await;

        let _ = sqlx::query("ALTER TABLE plants ADD COLUMN water_ml INTEGER")
            .execute(&self.pool)
            .await;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sensor_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                ambient_temp REAL NOT NULL,
                ambient_humidity REAL NOT NULL,
                plant_name TEXT NOT NULL,
                soil_moisture REAL NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        // Index for faster history and latest data lookups
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sensor_logs_plant_timestamp ON sensor_logs (plant_name, timestamp DESC)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS ai_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                query TEXT NOT NULL,
                response TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        // IoT Reliability: Pending alerts for offline-first
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS pending_alerts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                message TEXT NOT NULL,
                retry_count INTEGER DEFAULT 0
            )",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Menambahkan tanaman baru ke taman. Mengembalikan `true` jika berhasil,
    /// atau `false` jika tanaman dengan nama yang sama sudah ada.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError` jika penyisipan basis data gagal.
    pub async fn add_plant(&self, plant_type: &str, name: &str) -> Result<bool, DbError> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let result = sqlx::query(
            "INSERT INTO plants (name, plant_type, planted_date, last_watered, last_fertilized)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(name)
        .bind(plant_type)
        .bind(&today)
        .bind(&today)
        .bind(&today)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.as_database_error()
                    .is_some_and(sqlx::error::DatabaseError::is_unique_violation)
                {
                    return Ok(false);
                }
                Err(DbError::Sqlx(e))
            }
        }
    }

    /// Mengambil semua tanaman yang saat ini ditandai sebagai 'active'.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError` jika kueri basis data gagal.
    pub async fn active_plants(&self) -> Result<Vec<Plant>, DbError> {
        let rows = sqlx::query_as::<_, Plant>(
            "SELECT name, plant_type, planted_date, last_watered, last_fertilized, min_moisture, water_ml 
             FROM plants WHERE status = 'active'",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Memperbarui tanggal `last_watered` atau `last_fertilized` untuk sebuah tanaman.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError::PlantNotFound` jika tanaman tidak aktif, atau `DbError::Sqlx` untuk kegagalan kueri lainnya.
    pub async fn update_care(&self, name: &str, care_type: CareType) -> Result<(), DbError> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let query = format!(
            "UPDATE plants SET {} = ? WHERE name = ? AND status = 'active'",
            care_type.column_name()
        );

        let result = sqlx::query(&query)
            .bind(today)
            .bind(name)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::PlantNotFound(name.to_string()));
        }
        Ok(())
    }

    /// Mengembalikan ringkasan statistik taman.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError` jika kueri agregasi gagal.
    pub async fn garden_stats(&self) -> Result<serde_json::Value, DbError> {
        let active_count: i32 =
            sqlx::query_scalar("SELECT COUNT(*) FROM plants WHERE status = 'active'")
                .fetch_one(&self.pool)
                .await?;

        let harvested_count: i32 =
            sqlx::query_scalar("SELECT COUNT(*) FROM plants WHERE status = 'harvested'")
                .fetch_one(&self.pool)
                .await?;

        Ok(serde_json::json!({
            "active_plants": active_count,
            "harvested_plants": harvested_count,
        }))
    }

    /// Mencatat serangkaian pembacaan sensor ke basis data.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError` jika penyisipan gagal.
    pub async fn log_sensor_data(
        &self,
        plant_name: &str,
        moisture: f32,
        temp: f32,
        humidity: f32,
    ) -> Result<(), DbError> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        sqlx::query(
            "INSERT INTO sensor_logs (timestamp, ambient_temp, ambient_humidity, plant_name, soil_moisture)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(now)
        .bind(temp)
        .bind(humidity)
        .bind(plant_name)
        .bind(moisture)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Menandai tanaman sebagai dipanen (diarsipkan).
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError::PlantNotFound` jika tanaman tidak aktif, atau `DbError::Sqlx` untuk kegagalan kueri lainnya.
    pub async fn harvest_plant(&self, name: &str) -> Result<(), DbError> {
        let result = sqlx::query(
            "UPDATE plants SET status = 'harvested' WHERE name = ? AND status = 'active'",
        )
            .bind(name)
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::PlantNotFound(name.to_string()));
        }
        Ok(())
    }

    /// Memperbarui pengaturan kustom untuk sebuah tanaman.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError::PlantNotFound` jika tanaman tidak aktif, atau `DbError::Sqlx` untuk kegagalan kueri lainnya.
    pub async fn update_plant_settings(
        &self,
        name: &str,
        min_moisture: f32,
        water_ml: i32,
    ) -> Result<(), DbError> {
        let result = sqlx::query(
            "UPDATE plants SET min_moisture = ?, water_ml = ? WHERE name = ? AND status = 'active'",
        )
            .bind(min_moisture)
            .bind(water_ml)
            .bind(name)
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::PlantNotFound(name.to_string()));
        }
        Ok(())
    }

    /// Mengambil pembacaan sensor terbaru untuk setiap tanaman yang aktif.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError` jika kueri join gagal.
    #[allow(clippy::cast_possible_truncation)]
    pub async fn latest_sensor_data(&self) -> Result<Vec<crate::web::SensorData>, DbError> {
        // This query gets the latest log for each plant that is currently active.
        let rows = sqlx::query(
            r"
            SELECT 
                p.name, 
                p.min_moisture, 
                p.water_ml,
                sl.soil_moisture, 
                sl.ambient_temp, 
                sl.ambient_humidity, 
                sl.timestamp
            FROM plants p
            LEFT JOIN (
                SELECT plant_name, soil_moisture, ambient_temp, ambient_humidity, timestamp,
                       ROW_NUMBER() OVER (PARTITION BY plant_name ORDER BY timestamp DESC) as rn
                FROM sensor_logs
            ) sl ON p.name = sl.plant_name AND sl.rn = 1
            WHERE p.status = 'active'
            ORDER BY p.name ASC
            "
        )
        .fetch_all(&self.pool)
        .await?;

        let mut data = Vec::new();
        for row in rows {
            use sqlx::Row;
            let name: String = row.get("name");
            let moisture: f64 = row.get::<Option<f64>, _>("soil_moisture").unwrap_or(0.0);
            let temp: f64 = row.get::<Option<f64>, _>("ambient_temp").unwrap_or(0.0);
            let humidity: f64 = row.get::<Option<f64>, _>("ambient_humidity").unwrap_or(0.0);
            let timestamp: Option<String> = row.get("timestamp");
            let min_moisture: Option<f64> = row.get("min_moisture");
            let water_ml: Option<i32> = row.get("water_ml");

            let moisture_f32 = moisture as f32;
            let temp_f32 = temp as f32;
            let humidity_f32 = humidity as f32;

            data.push(crate::web::SensorData {
                plant_name: name,
                moisture: moisture_f32,
                temperature: temp_f32,
                humidity: humidity_f32,
                timestamp: timestamp
                    .unwrap_or_else(|| "Never".to_string())
                    .split(' ')
                    .next_back()
                    .unwrap_or("Never")
                    .to_string(),
                min_moisture: min_moisture.map(|m| m as f32),
                water_ml,
            });
        }

        Ok(data)
    }

    /// Mencatat interaksi AI.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError` jika penyisipan gagal.
    #[allow(dead_code)]
    pub async fn log_ai_interaction(&self, query: &str, response: &str) -> Result<(), DbError> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        sqlx::query("INSERT INTO ai_logs (timestamp, query, response) VALUES (?, ?, ?)")
            .bind(now)
            .bind(query)
            .bind(response)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Mengambil log AI terbaru.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError` jika kueri gagal.
    #[allow(dead_code)]
    pub async fn recent_ai_logs(&self, limit: i32) -> Result<Vec<crate::web::AiLog>, DbError> {
        let rows = sqlx::query(
            "SELECT timestamp, query, response FROM ai_logs ORDER BY id DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let logs = rows
            .into_iter()
            .map(|row| {
                use sqlx::Row;
                crate::web::AiLog {
                    timestamp: row
                        .get::<String, _>("timestamp")
                        .split(' ')
                        .next_back()
                        .unwrap_or("Never")
                        .to_string(),
                    query: row.get("query"),
                    response: row.get("response"),
                }
            })
            .collect();

        Ok(logs)
    }

    /// Mengambil data sensor historis untuk tanaman tertentu.
    ///
    /// # Argumen
    /// * `plant_name` - Nama tanaman yang akan dikueri.
    /// * `hours` - Jumlah jam riwayat yang akan diambil (misalnya, 24 untuk 1 hari, 168 untuk 7 hari).
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError` jika kueri gagal.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub async fn sensor_history(
        &self,
        plant_name: &str,
        hours: i32,
    ) -> Result<Vec<crate::web::SensorHistoryPoint>, DbError> {
        tracing::debug!(plant = %plant_name, hours = %hours, "Fetching sensor history");

        let rows = sqlx::query(
            r"
            SELECT timestamp, soil_moisture, ambient_temp, ambient_humidity
            FROM sensor_logs
            WHERE plant_name = ?
              AND timestamp >= datetime('now', '-' || ? || ' hours')
            ORDER BY timestamp ASC
            "
        )
        .bind(plant_name)
        .bind(hours)
        .fetch_all(&self.pool)
        .await?;

        tracing::debug!("Found {} history points", rows.len());

        let data = rows
            .into_iter()
            .map(|row| {
                use sqlx::Row;
                // Use robust decoding: try f64, then i64 as f64 if that fails, otherwise 0.0
                let moisture = row.try_get::<f64, _>("soil_moisture")
                    .or_else(|_| row.try_get::<i64, _>("soil_moisture").map(|v| v as f64))
                    .unwrap_or(0.0);
                
                let temp = row.try_get::<f64, _>("ambient_temp")
                    .or_else(|_| row.try_get::<i64, _>("ambient_temp").map(|v| v as f64))
                    .unwrap_or(0.0);
                
                let humidity = row.try_get::<f64, _>("ambient_humidity")
                    .or_else(|_| row.try_get::<i64, _>("ambient_humidity").map(|v| v as f64))
                    .unwrap_or(0.0);

                let moisture_f32 = moisture as f32;
                let temp_f32 = temp as f32;
                let humidity_f32 = humidity as f32;

                crate::web::SensorHistoryPoint {
                    timestamp: row.get::<String, _>("timestamp"),
                    moisture: moisture_f32,
                    temperature: temp_f32,
                    humidity: humidity_f32,
                }
            })
            .collect();

        Ok(data)
    }

    /// Menghapus tanaman dan semua log sensor terkait secara permanen.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError::PlantNotFound` jika tanaman tidak ada, atau `DbError::Sqlx` untuk kegagalan lainnya.
    pub async fn delete_plant(&self, name: &str) -> Result<(), DbError> {
        // Delete sensor logs first (referential cleanup)
        sqlx::query("DELETE FROM sensor_logs WHERE plant_name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;

        let result = sqlx::query("DELETE FROM plants WHERE name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::PlantNotFound(name.to_string()));
        }
        Ok(())
    }

    // ── IoT Reliability: Alert Queueing ──────────────────────────────

    /// Mengantrekan peringatan untuk pengiriman nanti.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError` jika penyisipan gagal.
    pub async fn queue_alert(&self, message: &str) -> Result<(), DbError> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        sqlx::query("INSERT INTO pending_alerts (timestamp, message) VALUES (?, ?)")
            .bind(now)
            .bind(message)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Mengambil semua peringatan yang tertunda.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError` jika kueri gagal.
    pub async fn pending_alerts(&self) -> Result<Vec<(i64, String)>, DbError> {
        use sqlx::Row;
        let rows = sqlx::query("SELECT id, message FROM pending_alerts ORDER BY id ASC")
            .fetch_all(&self.pool)
            .await?;
        let alerts = rows
            .into_iter()
            .map(|row| (row.get("id"), row.get("message")))
            .collect();

        Ok(alerts)
    }

    /// Menghapus peringatan yang tertunda setelah berhasil dikirim.
    ///
    /// # Errors
    ///
    /// Mengembalikan `DbError` jika penghapusan gagal.
    pub async fn delete_alert(&self, id: i64) -> Result<(), DbError> {
        sqlx::query("DELETE FROM pending_alerts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
