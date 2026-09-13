use memfuse_db::{CollectionConfig, MemFuse, MemFuseConfig};
use memfuse_store::{PressureLevel, SystemPressure};
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_insert_backpressure_when_pressure_critical() {
    let tmp = tempfile::TempDir::new().expect("temp dir");
    let config = MemFuseConfig {
        dimension: 4,
        ..Default::default()
    };

    let db = MemFuse::open_with_config(tmp.path(), config)
        .await
        .expect("open db");

    let (pressure_tx, pressure_rx) = tokio::sync::watch::channel(SystemPressure {
        wal_queue_depth: 0,
        blocking_thread_utilization: 0.0,
        embedding_queue_depth: 0,
        pressure_level: PressureLevel::Normal,
    });

    let col = db.collection("bp_test").await.expect("collection");
    col.set_config(CollectionConfig {
        backpressure_delay_ms: Some(100),
    });
    col.set_pressure_receiver(pressure_rx);

    // 1. Normal pressure level -> insert should complete quickly (< 80ms)
    let start_normal = Instant::now();
    col.insert("doc_normal", &[1.0, 0.0, 0.0, 0.0], None)
        .await
        .expect("insert normal");
    let elapsed_normal = start_normal.elapsed();
    assert!(
        elapsed_normal < Duration::from_millis(80),
        "Normal pressure insert took {:?}, expected < 80ms",
        elapsed_normal
    );

    // 2. Set pressure level to Critical -> insert should be delayed by ~100ms
    pressure_tx
        .send(SystemPressure {
            wal_queue_depth: 600,
            blocking_thread_utilization: 0.9,
            embedding_queue_depth: 0,
            pressure_level: PressureLevel::Critical,
        })
        .expect("send critical pressure");

    let start_critical = Instant::now();
    col.insert("doc_critical", &[0.0, 1.0, 0.0, 0.0], None)
        .await
        .expect("insert critical");
    let elapsed_critical = start_critical.elapsed();
    assert!(
        elapsed_critical >= Duration::from_millis(90),
        "Critical pressure insert took {:?}, expected >= 90ms backpressure delay",
        elapsed_critical
    );

    // 3. Return pressure level to Normal -> insert should complete quickly without delay again
    pressure_tx
        .send(SystemPressure {
            wal_queue_depth: 10,
            blocking_thread_utilization: 0.1,
            embedding_queue_depth: 5,
            pressure_level: PressureLevel::Normal,
        })
        .expect("send normal pressure");

    let start_restored = Instant::now();
    col.insert("doc_restored", &[0.0, 0.0, 1.0, 0.0], None)
        .await
        .expect("insert restored");
    let elapsed_restored = start_restored.elapsed();
    assert!(
        elapsed_restored < Duration::from_millis(80),
        "Restored normal pressure insert took {:?}, expected < 80ms",
        elapsed_restored
    );

    // Verify all docs were successfully inserted and persisted (no hard abort)
    assert!(col.get("doc_normal").await.expect("get").is_some());
    assert!(col.get("doc_critical").await.expect("get").is_some());
    assert!(col.get("doc_restored").await.expect("get").is_some());
}
