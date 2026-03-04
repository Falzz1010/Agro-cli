use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{Duration, sleep};

/// Simulates reading soil moisture from a physical sensor.
///
/// Returns a moisture level between 30.0% and 70.0%.
#[allow(clippy::cast_precision_loss)]
pub fn read_soil_moisture(_plant_name: &str) -> f32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    // Deterministic but varying mock data: 30% to 70%
    30.0 + (nanos % 40) as f32
}

/// Simulates reading ambient temperature from a physical sensor.
///
/// Returns temperature in Celsius between 24.0 and 28.0.
#[allow(clippy::cast_precision_loss)]
pub fn read_temperature() -> f32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    // 24 to 28 degrees
    24.0 + (nanos % 5) as f32
}

/// Simulates reading ambient humidity from a physical sensor.
///
/// Returns humidity percentage between 60.0 and 80.0.
#[allow(clippy::cast_precision_loss)]
pub fn read_humidity() -> f32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    // 60% to 80%
    60.0 + (nanos % 21) as f32
}

/// Activates a physical water pump for a specified duration.
///
/// This is an asynchronous simulation of a hardware action.
///
/// # Arguments
/// * `name` - The nickname of the plant to water.
/// * `duration_seconds` - How long to keep the pump active.
pub async fn water_plant(name: &str, duration_seconds: u64) {
    println!(
        "🚿 [HARDWARE] Activating pump for {name} ({duration_seconds}s)..."
    );
    sleep(Duration::from_secs(duration_seconds)).await;
    println!("✅ [HARDWARE] Pump deactivated.");
}
