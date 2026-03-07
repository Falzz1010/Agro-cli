use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{Duration, sleep};

/// Simulasi membaca tingkat kelembaban tanah dari sensor fisik.
///
/// Mengembalikan tingkat kelembaban antara 30.0% dan 70.0%.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn read_soil_moisture(_plant_name: &str) -> f32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    // Deterministic but varying mock data: 30% to 70%
    30.0 + (nanos % 40) as f32
}

/// Simulasi membaca suhu ambien dari sensor fisik.
///
/// Mengembalikan suhu dalam Celsius antara 24.0 dan 28.0.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn read_temperature() -> f32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    // 24 to 28 degrees
    24.0 + (nanos % 5) as f32
}

/// Simulasi membaca kelembaban ambien dari sensor fisik.
///
/// Mengembalikan persentase kelembaban antara 60.0 dan 80.0.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn read_humidity() -> f32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    // 60% to 80%
    60.0 + (nanos % 21) as f32
}

/// Mengaktifkan pompa air fisik untuk durasi tertentu.
///
/// Ini adalah simulasi asinkron dari tindakan perangkat keras.
///
/// # Argumen
/// * `name` - Nama panggilan tanaman yang akan disiram.
/// * `duration_seconds` - Berapa lama pompa tetap aktif.
pub async fn water_plant(name: &str, duration_seconds: u64) {
    println!(
        "🚿 [HARDWARE] Activating pump for {name} ({duration_seconds}s)..."
    );
    sleep(Duration::from_secs(duration_seconds)).await;
    println!("✅ [HARDWARE] Pump deactivated.");
}
