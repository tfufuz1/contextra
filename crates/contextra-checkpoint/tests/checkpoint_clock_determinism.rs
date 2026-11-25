#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_checkpoint::{CheckpointGuard, PersistentCheckpointStore};
use contextra_testkit::{InMemoryStorageEngine, ManualClock};
use contextra_types::TxId;
use std::sync::Arc;

#[tokio::test]
async fn test_a_stores_with_same_clock_produce_identical_timestamps() {
    let initial_nanos = 1_700_000_000_000_000_000; // 1,700,000,000 ms

    // Store 1 setup
    let clock1 = Arc::new(ManualClock::new(initial_nanos));
    let storage1 = Arc::new(InMemoryStorageEngine::new());
    let store1 = PersistentCheckpointStore::open(storage1.clone(), "det_ns_a1")
        .await
        .expect("open store 1")
        .with_clock(clock1.clone());

    let meta1 = store1
        .create_checkpoint(
            "cp_alpha",
            "coll_a",
            10,
            TxId::new(100),
            serde_json::json!({"state": "A"}),
        )
        .await
        .expect("create checkpoint 1");

    let guard1 = store1.create_guard(TxId::new(101)).expect("create guard 1");
    let state_cp1 = guard1.checkpoint().expect("get state cp 1").clone();
    let _ = guard1.commit();

    // Store 2 setup (identical clock start)
    let clock2 = Arc::new(ManualClock::new(initial_nanos));
    let storage2 = Arc::new(InMemoryStorageEngine::new());
    let store2 = PersistentCheckpointStore::open(storage2.clone(), "det_ns_a2")
        .await
        .expect("open store 2")
        .with_clock(clock2.clone());

    let meta2 = store2
        .create_checkpoint(
            "cp_alpha",
            "coll_a",
            10,
            TxId::new(100),
            serde_json::json!({"state": "A"}),
        )
        .await
        .expect("create checkpoint 2");

    let guard2 = store2.create_guard(TxId::new(101)).expect("create guard 2");
    let state_cp2 = guard2.checkpoint().expect("get state cp 2").clone();
    let _ = guard2.commit();

    // Verification
    assert_eq!(
        meta1.created_at, meta2.created_at,
        "created_at must be identical across both stores"
    );
    assert_eq!(
        meta1.created_at,
        initial_nanos / 1_000_000,
        "created_at must match exact clock milliseconds"
    );
    assert_eq!(
        state_cp1.timestamp_ms, state_cp2.timestamp_ms,
        "Guard state checkpoint timestamp_ms must be identical across both stores"
    );
    assert_eq!(
        state_cp1.timestamp_ms,
        initial_nanos / 1_000_000,
        "timestamp_ms must match exact clock milliseconds"
    );
}

#[tokio::test]
async fn test_b_clock_backward_jump_preserves_monotonicity() {
    let clock = Arc::new(ManualClock::new(2_000_000_000_000)); // 2,000,000 ms
    let storage = Arc::new(InMemoryStorageEngine::new());
    let store = PersistentCheckpointStore::open(storage, "det_ns_b")
        .await
        .expect("open store")
        .with_clock(clock.clone());

    let ts1 = store.monotonic_timestamp_ms();
    assert_eq!(ts1, 2_000_000);

    // Jump clock backward to 1,000,000 ms
    clock.set_nanos(1_000_000_000_000);

    let ts2 = store.monotonic_timestamp_ms();
    assert!(
        ts2 >= ts1,
        "Monotonic timestamp must not decrease when clock jumps backward (ts1={ts1}, ts2={ts2})"
    );
    assert_eq!(ts2, 2_000_000);
}

#[tokio::test]
async fn test_c_unfinalized_guard_drop_records_orphan_with_test_clock() {
    let clock_nanos = 3_456_000_000_000_000; // 3,456,000 ms
    let clock = Arc::new(ManualClock::new(clock_nanos));
    let storage = Arc::new(InMemoryStorageEngine::new());

    let cp = contextra_checkpoint::StateCheckpoint {
        tx_id: TxId::new(999),
        timestamp_ms: clock_nanos / 1_000_000,
        namespace: Some("test_orphan_drop".to_string()),
    };

    let orphan_reg =
        Arc::new(contextra_checkpoint::InstanceOrphanRegistry::new("").with_clock(clock));

    let guard = CheckpointGuard::with_registry(cp, storage, "test_orphan_drop", orphan_reg.clone());

    // Unfinalized drop triggers orphan registration in registry
    drop(guard);

    let orphans = orphan_reg.get_orphaned_checkpoints();

    assert_eq!(orphans.len(), 1, "Expected 1 orphaned checkpoint recorded");
    assert_eq!(
        orphans[0].timestamp_ms,
        clock_nanos / 1_000_000,
        "Orphaned checkpoint timestamp_ms must match test clock timestamp"
    );
}

#[tokio::test]
async fn test_d_default_constructors_smoke_test_with_system_clock() {
    let storage = Arc::new(InMemoryStorageEngine::new());
    let store = PersistentCheckpointStore::open(storage.clone(), "smoke_ns")
        .await
        .expect("open store with default SystemClock");

    let meta = store
        .create_checkpoint(
            "smoke_cp",
            "smoke_coll",
            1,
            TxId::new(1),
            serde_json::json!({}),
        )
        .await
        .expect("create checkpoint with SystemClock");

    assert!(
        meta.created_at > 0,
        "created_at from SystemClock must be greater than zero"
    );

    let guard = CheckpointGuard::for_agent_step(storage, TxId::new(2))
        .await
        .expect("for_agent_step with default SystemClock");

    let cp = guard.checkpoint().expect("get agent step cp");
    assert!(
        cp.timestamp_ms > 0,
        "timestamp_ms from SystemClock must be greater than zero"
    );
    let _ = guard.commit();
}

#[tokio::test]
async fn test_e_pin_guard_with_clock_determinism() {
    let pin_clock_nanos = 5_000_000_000_000_000; // 5,000,000 ms
    let clock = Arc::new(ManualClock::new(pin_clock_nanos));
    let storage = Arc::new(InMemoryStorageEngine::new());
    let orphan_reg = Arc::new(contextra_checkpoint::InstanceOrphanRegistry::new(""));

    let pin_guard =
        contextra_checkpoint::PinGuard::pin_with_clock(storage, 777, orphan_reg.clone(), clock)
            .await
            .expect("pin_with_clock");

    drop(pin_guard);

    let orphan_pins = orphan_reg.get_orphan_pins();
    assert_eq!(orphan_pins.len(), 1);
    assert_eq!(orphan_pins[0].seq_no, 777);
    assert_eq!(
        orphan_pins[0].timestamp_ms,
        pin_clock_nanos / 1_000_000,
        "PinGuard orphan timestamp must match clock injected via pin_with_clock"
    );
}
