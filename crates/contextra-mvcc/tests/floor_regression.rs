//! Floor regression tests for GcFloor and SnapshotLease.

use contextra_mvcc::snapshot::{GcFloor, SnapshotRegistry};
use contextra_mvcc::tx_buffer::TxBuffer;
use contextra_types::TxId;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[test]
fn test_floor_regression_put_50_tombstone_90_reader_80() {
    // Scenario: Put key at Seq 50, Tombstone at Seq 90, Reader starts at Seq 80
    // floor() must not exceed 80 as long as the reader is alive.
    let registry = Arc::new(SnapshotRegistry::new());
    let last_applied = Arc::new(AtomicU64::new(100));
    let tx_buffer = Arc::new(TxBuffer::<Vec<u8>>::new().with_snapshot_registry(registry.clone()));

    let floor_calc = GcFloor::new(
        registry.clone(),
        Some(tx_buffer.clone()),
        last_applied.clone(),
    );

    // Initial floor with no reader active: floor == last_applied (100)
    assert_eq!(floor_calc.floor(), 100);

    // Reader acquires snapshot at seq 80
    let lease = registry.acquire(|| 80);
    assert_eq!(floor_calc.floor(), 80);

    // Advance last_applied to 120
    last_applied.store(120, Ordering::Release);
    // Floor must remain capped at 80 while reader lease is held
    assert_eq!(floor_calc.floor(), 80);

    // Dropping the lease releases the floor back to last_applied (120)
    drop(lease);
    assert_eq!(floor_calc.floor(), 120);
}

#[test]
fn test_reader_started_no_read_yet() {
    // Reader started via TxBuffer (no read performed yet): floor() <= Reader-Seq
    let registry = Arc::new(SnapshotRegistry::new());
    let last_applied = Arc::new(AtomicU64::new(100));
    let tx_buffer = Arc::new(
        TxBuffer::<Vec<u8>>::new()
            .with_snapshot_registry(registry.clone())
            .with_last_applied(last_applied.clone()),
    );

    let floor_calc = GcFloor::new(
        registry.clone(),
        Some(tx_buffer.clone()),
        last_applied.clone(),
    );

    let reader_tx = TxId::new(42);
    // Begin transaction at snapshot seq 75 before any read operation
    tx_buffer.begin_with_seq(reader_tx, 75);

    // Even with 0 tracked reads in TxBuffer, the acquired SnapshotLease caps floor() at 75
    assert!(tx_buffer.min_read_snapshot().is_none());
    assert_eq!(floor_calc.floor(), 75);

    // Draining the transaction drops the lease and releases floor()
    tx_buffer.drain(reader_tx);
    assert_eq!(floor_calc.floor(), 100);
}

#[test]
fn test_lease_drop_releases_floor() {
    let registry = Arc::new(SnapshotRegistry::new());
    let last_applied = Arc::new(AtomicU64::new(200));

    let floor_calc = GcFloor::<Vec<u8>>::new(registry.clone(), None, last_applied.clone());

    let lease1 = registry.acquire(|| 150);
    let lease2 = registry.acquire(|| 120);

    assert_eq!(floor_calc.floor(), 120);

    drop(lease2);
    assert_eq!(floor_calc.floor(), 150);

    drop(lease1);
    assert_eq!(floor_calc.floor(), 200);
}
