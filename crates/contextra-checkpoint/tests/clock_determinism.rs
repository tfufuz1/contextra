use contextra_checkpoint::{CheckpointGuard, PersistentCheckpointStore, PinGuard};
use contextra_testkit::{InMemoryStorageEngine, ManualClock};
use contextra_types::TxId;
use std::sync::Arc;

#[tokio::test]
async fn test_checkpoint_store_clock_determinism() {
    let start_nanos = 1_700_000_000_000_000_000; // 1,700,000,000 ms

    // Run 1
    let clock1 = Arc::new(ManualClock::new(start_nanos));
    let storage1 = Arc::new(InMemoryStorageEngine::new());
    let store1 = PersistentCheckpointStore::open(storage1.clone(), "test_ns_1")
        .await
        .unwrap()
        .with_clock(clock1.clone());

    let meta1 = store1
        .create_checkpoint(
            "cp_det",
            "coll1",
            100,
            TxId::new(1),
            serde_json::json!({"test": "value"}),
        )
        .await
        .unwrap();

    let guard1 = store1.create_guard(TxId::new(2)).unwrap();
    let state_cp1 = guard1.checkpoint().unwrap().clone();

    // Pin seq and drop PinGuard without explicit unpin
    let pin_guard1 = PinGuard::pin(storage1.clone(), 500, store1.orphan_registry().clone())
        .await
        .unwrap();
    drop(pin_guard1);

    let orphan_pins1 = store1.orphan_registry().get_orphan_pins();

    // Run 2 (identical start time on clock)
    let clock2 = Arc::new(ManualClock::new(start_nanos));
    let storage2 = Arc::new(InMemoryStorageEngine::new());
    let store2 = PersistentCheckpointStore::open(storage2.clone(), "test_ns_2")
        .await
        .unwrap()
        .with_clock(clock2.clone());

    let meta2 = store2
        .create_checkpoint(
            "cp_det",
            "coll1",
            100,
            TxId::new(1),
            serde_json::json!({"test": "value"}),
        )
        .await
        .unwrap();

    let guard2 = store2.create_guard(TxId::new(2)).unwrap();
    let state_cp2 = guard2.checkpoint().unwrap().clone();

    let pin_guard2 = PinGuard::pin(storage2.clone(), 500, store2.orphan_registry().clone())
        .await
        .unwrap();
    drop(pin_guard2);

    let orphan_pins2 = store2.orphan_registry().get_orphan_pins();

    // Assertions: Byte-exact equality across runs
    assert_eq!(
        meta1.created_at, meta2.created_at,
        "created_at must be deterministic with ManualClock"
    );
    assert_eq!(
        meta1.created_at,
        start_nanos / 1_000_000,
        "created_at must match clock millis"
    );
    assert_eq!(
        state_cp1.timestamp_ms, state_cp2.timestamp_ms,
        "StateCheckpoint timestamp_ms must be deterministic"
    );
    assert_eq!(
        state_cp1.timestamp_ms,
        start_nanos / 1_000_000,
        "timestamp_ms must match clock millis"
    );

    assert_eq!(orphan_pins1.len(), 1);
    assert_eq!(orphan_pins2.len(), 1);
    assert_eq!(
        orphan_pins1[0].timestamp_ms, orphan_pins2[0].timestamp_ms,
        "PinnedSeqNoOrphan timestamp_ms must be deterministic"
    );
    assert_eq!(
        orphan_pins1[0].timestamp_ms,
        start_nanos / 1_000_000,
        "PinnedSeqNoOrphan timestamp_ms must match clock millis recorded at pin time"
    );
}

#[tokio::test]
async fn test_agent_step_with_clock_determinism() {
    let start_nanos = 1_800_000_000_000_000_000;
    let clock = Arc::new(ManualClock::new(start_nanos));
    let storage = Arc::new(InMemoryStorageEngine::new());

    let guard = CheckpointGuard::for_agent_step_with_clock(storage, TxId::new(42), clock)
        .await
        .unwrap();

    let cp = guard.checkpoint().unwrap();
    assert_eq!(cp.timestamp_ms, start_nanos / 1_000_000);
}
