use criterion::{criterion_group, criterion_main, Criterion};
// Crate names with uppercase letters are lowercased and hyphens become underscores
use agrocli::db::Database;
use tokio::runtime::Runtime;

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Initialize DB once outside the hot loop
    let db_path = "data/garden.db";
    let db_url = format!("sqlite://{}", db_path);
    let db = rt.block_on(async { Database::new(&db_url).await.unwrap() });
    
    c.bench_function("db_sensor_history", |b| {
        b.to_async(&rt).iter(|| async {
            // Test sensor history query (common in dashboard)
            let _ = db.sensor_history("Tomato", 24).await;
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
