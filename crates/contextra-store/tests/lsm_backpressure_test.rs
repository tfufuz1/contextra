//! Integration test for `LsmStorage` backpressure evaluation via `pressure_receiver()`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_ports::{StorageEngine, TxId};
use contextra_store::system_pressure::{PressureLevel, WAL_QUEUE_CRITICAL_THRESHOLD};
use contextra_store::{LsmConfig, LsmStorage};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[tokio::test]
async fn test_lsm_storage_backpressure_throttles_writes_and_resumes() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::open(config).await.expect("open storage"));
    let mut pressure_rx = storage.pressure_receiver();

    // 1. Initial level must be Normal
    assert_eq!(pressure_rx.borrow().pressure_level, PressureLevel::Normal);

    // Measure write latency under Normal pressure
    let start_normal = Instant::now();
    for i in 1..=3 {
        let tx = TxId::new(i);
        storage.put(tx, format!("key_{i}").as_bytes(), b"val").await.expect("put");
        storage.commit(tx).await.expect("commit");
    }
    let normal_duration = start_normal.elapsed();

    // 2. Simulate high WAL queue depth to trigger Critical pressure
    storage.set_wal_queue_depth_for_test(WAL_QUEUE_CRITICAL_THRESHOLD + 50);

    // Wait for system pressure monitor task to detect depth and update watch channel
    let mut transitioned = false;
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if pressure_rx.borrow().pressure_level == PressureLevel::Critical {
            transitioned = true;
            break;
        }
        if pressure_rx.changed().await.is_err() {
            break;
        }
    }
    assert!(transitioned, "System pressure monitor should transition pressure_rx to Critical");

    // Measure write latency under Critical pressure
    let start_critical = Instant::now();
    let tx_crit = TxId::new(100);
    storage.put(tx_crit, b"key_crit", b"val_crit").await.expect("put crit");
    storage.commit(tx_crit).await.expect("commit crit");
    let critical_duration = start_critical.elapsed();

    // Throttling under Critical pressure applies 50ms delay per put and per commit (>= 100ms total)
    assert!(
        critical_duration >= Duration::from_millis(80),
        "Write under Critical pressure should be throttled (expected >= 80ms, got {:?})",
        critical_duration
    );

    // 3. Lower WAL queue depth back to 0 -> pressure transitions back to Normal
    storage.set_wal_queue_depth_for_test(0);

    let mut resumed = false;
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if pressure_rx.borrow().pressure_level == PressureLevel::Normal {
            resumed = true;
            break;
        }
        if pressure_rx.changed().await.is_err() {
            break;
        }
    }
    assert!(resumed, "System pressure monitor should transition pressure_rx back to Normal");

    // Measure write latency under Normal pressure again
    let start_resumed = Instant::now();
    let tx_resumed = TxId::new(200);
    storage.put(tx_resumed, b"key_resumed", b"val_resumed").await.expect("put resumed");
    storage.commit(tx_resumed).await.expect("commit resumed");
    let resumed_duration = start_resumed.elapsed();

    assert!(
        resumed_duration < Duration::from_millis(40),
        "Write under Normal pressure after resume should run fast (expected < 40ms, got {:?})",
        resumed_duration
    );

    let _ = normal_duration;
}
