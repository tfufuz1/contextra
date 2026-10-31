//! # Vertragstests: MaintenanceScheduler (`tests/maintenance_scheduler_tests.rs`)
//!
//! Validiert die Auslösebedingungen, Planungslogik mit `ManualClock`,
//! Fehlerisolation (Zero-Panic), Grenzwerte und den sauberen Shutdown des Schedulers.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::get_first,
    clippy::panic,
    clippy::disallowed_methods
)]

use contextra_cognition::maintenance_config::MaintenanceConfig;
use contextra_cognition::maintenance_scheduler::MaintenanceScheduler;
use contextra_cognition::memory_consolidation::ConsolidationConfig;
use contextra_engine::collection::Collection;
use contextra_graph::CsrGraph;
use contextra_ports::StorageEngine;
use contextra_store::LsmStorage;
use contextra_testkit::ManualClock;
use contextra_vector::HnswIndex;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

async fn create_test_collection() -> Arc<Collection<LsmStorage, HnswIndex>> {
    let dir = tempdir().unwrap();
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(),
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(),
    );
    Arc::new(Collection::new(
        "test-scheduler".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_engine::Language::English,
    ))
}

#[tokio::test]
async fn test_all_public_methods_direct_coverage() {
    let col = create_test_collection().await;
    let config = MaintenanceConfig {
        tick_interval_secs: 5,
        decay_enabled: false,
        percolation_enabled: false,
        replicator_enabled: false,
        background_consolidation_enabled: false,
        ..Default::default()
    };
    let initial_nanos = 1_000_000_000u64;
    let clock = Arc::new(ManualClock::new(initial_nanos));

    let scheduler = MaintenanceScheduler::new(config, col.clone(), ConsolidationConfig::default())
        .with_clock(clock.clone());

    // Public session counters
    assert_eq!(scheduler.active_agent_sessions(), 0);
    assert_eq!(scheduler.increment_active_sessions(), 1);
    assert_eq!(scheduler.increment_active_sessions(), 2);
    assert_eq!(scheduler.active_agent_sessions(), 2);
    assert_eq!(scheduler.decrement_active_sessions(), 1);
    assert_eq!(scheduler.decrement_active_sessions(), 0);
    assert_eq!(scheduler.decrement_active_sessions(), 0); // saturating sub

    // Direct run_tick
    scheduler.run_tick().await;

    // Check WAL intent written
    let prefix = b"__maintenance_intent:tick".to_vec();
    let entries = col.storage().scan_prefix(&prefix).await.unwrap();
    assert!(!entries.is_empty());
    let val: serde_json::Value = serde_json::from_slice(&entries[0].1).unwrap();
    assert_eq!(val["status"], "completed");
    assert_eq!(val["timestamp_ms"], initial_nanos / 1_000_000);
}

#[tokio::test]
async fn test_scheduler_background_timing_and_cancellation() {
    let col = create_test_collection().await;
    let config = MaintenanceConfig {
        tick_interval_secs: 1, // 1 second interval
        decay_enabled: false,
        percolation_enabled: false,
        replicator_enabled: false,
        background_consolidation_enabled: false,
        ..Default::default()
    };
    let initial_nanos = 10_000_000_000u64; // 10s
    let clock = Arc::new(ManualClock::new(initial_nanos));
    let scheduler = Arc::new(
        MaintenanceScheduler::new(config, col.clone(), ConsolidationConfig::default())
            .with_clock(clock.clone()),
    );

    let cancel_token = CancellationToken::new();
    let handle = scheduler.start(cancel_token.clone());

    let prefix = b"__maintenance_intent:tick".to_vec();

    // First tick fires immediately upon start as tokio::time::interval yields at t=0
    let mut initial_fired = false;
    for _ in 0..20 {
        sleep(Duration::from_millis(50)).await;
        let entries = col.storage().scan_prefix(&prefix).await.unwrap();
        if !entries.is_empty() {
            initial_fired = true;
            let val: serde_json::Value = serde_json::from_slice(&entries[0].1).unwrap();
            assert_eq!(val["status"], "completed");
            assert_eq!(val["timestamp_ms"], initial_nanos / 1_000_000);
            break;
        }
    }
    assert!(initial_fired, "First tick must fire on interval startup");

    // Advance clock time by 50s to simulate large time jump
    clock.advance(Duration::from_secs(50));

    // Next periodic tick fires after the 1 second interval expires
    let mut next_fired = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(100)).await;
        let entries = col.storage().scan_prefix(&prefix).await.unwrap();
        if !entries.is_empty() {
            let val: serde_json::Value = serde_json::from_slice(&entries[0].1).unwrap();
            if val["timestamp_ms"] == (initial_nanos / 1_000_000) + 50_000 {
                next_fired = true;
                break;
            }
        }
    }
    assert!(
        next_fired,
        "Subsequent tick must fire with advanced clock timestamp"
    );

    // Clean shutdown via CancellationToken
    cancel_token.cancel();
    let res = handle.await;
    assert!(res.is_ok(), "Task must shut down cleanly on cancellation");
}

#[tokio::test]
async fn test_scheduler_step_fault_isolation_and_zero_panic() {
    let col = create_test_collection().await;
    // Enable all steps including decay
    let config = MaintenanceConfig {
        tick_interval_secs: 1,
        decay_enabled: true,
        percolation_enabled: true,
        replicator_enabled: false,
        background_consolidation_enabled: true,
        background_consolidation_episode_threshold: 1,
        ..Default::default()
    };
    let clock = Arc::new(ManualClock::new(5_000_000_000));
    let scheduler = Arc::new(
        MaintenanceScheduler::new(config, col.clone(), ConsolidationConfig::default())
            .with_clock(clock),
    );

    // Run tick on empty collection with background consolidation enabled
    // Should not panic, all errors in sub-steps are caught and WAL completion intent is still committed
    scheduler.run_tick().await;

    let prefix = b"__maintenance_intent:tick".to_vec();
    let entries = col.storage().scan_prefix(&prefix).await.unwrap();
    assert_eq!(entries.len(), 1);
    let val: serde_json::Value = serde_json::from_slice(&entries[0].1).unwrap();
    assert_eq!(val["status"], "completed");
}

#[tokio::test]
async fn test_boundary_values_zero_and_large_intervals() {
    let col = create_test_collection().await;

    // Interval 0 should be guarded with max(1) internally without panic
    let zero_config = MaintenanceConfig {
        tick_interval_secs: 0,
        ..Default::default()
    };
    let scheduler_zero = Arc::new(MaintenanceScheduler::new(
        zero_config,
        col.clone(),
        ConsolidationConfig::default(),
    ));

    let token_zero = CancellationToken::new();
    let handle_zero = scheduler_zero.start(token_zero.clone());
    token_zero.cancel();
    let res_zero = handle_zero.await;
    assert!(
        res_zero.is_ok(),
        "Zero interval scheduler starts and stops safely"
    );

    // Very large interval: 1 year in seconds (overflow-safe)
    let large_config = MaintenanceConfig {
        tick_interval_secs: 31_536_000,
        ..Default::default()
    };
    let scheduler_large = Arc::new(MaintenanceScheduler::new(
        large_config,
        col,
        ConsolidationConfig::default(),
    ));

    let token_large = CancellationToken::new();
    let handle_large = scheduler_large.start(token_large.clone());
    token_large.cancel();
    let res_large = handle_large.await;
    assert!(
        res_large.is_ok(),
        "Large interval scheduler starts and stops safely"
    );
}

#[tokio::test]
async fn test_session_gating_blocks_consolidation_when_active() {
    let col = create_test_collection().await;
    let config = MaintenanceConfig {
        tick_interval_secs: 1,
        background_consolidation_enabled: true,
        percolation_enabled: true,
        ..Default::default()
    };
    let scheduler = MaintenanceScheduler::new(config, col, ConsolidationConfig::default());

    // When sessions > 0, background consolidation and percolation are bypassed
    scheduler.increment_active_sessions();
    assert_eq!(scheduler.active_agent_sessions(), 1);

    // run_tick completes without triggering locked background tasks
    scheduler.run_tick().await;

    scheduler.decrement_active_sessions();
    assert_eq!(scheduler.active_agent_sessions(), 0);
}
