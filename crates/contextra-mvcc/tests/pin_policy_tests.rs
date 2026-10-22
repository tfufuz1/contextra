#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Tests for Snapshot Pin-Policy (`max_snapshot_age` and `pin_grace`) and `SnapshotExpired` error handling.

use contextra_mvcc::{SnapshotRegistry, DEFAULT_MAX_SNAPSHOT_AGE_TICKS, DEFAULT_PIN_GRACE_TICKS};
use contextra_testkit::ManualClock;
use contextra_types::ContextraError;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_expired_snapshot_returns_snapshot_expired_error() {
    let clock = Arc::new(ManualClock::new(1_000_000_000));
    let registry = Arc::new(SnapshotRegistry::new().with_clock(clock.clone()));

    // Set max snapshot age to 10 seconds (10_000_000_000 ticks)
    registry.set_max_snapshot_age(Duration::from_secs(10));
    assert_eq!(registry.max_snapshot_age(), Duration::from_secs(10));

    // Acquire snapshot lease at current clock time (t = 1s)
    let lease = registry.acquire(|| 100);
    assert_eq!(lease.seq_no(), 100);
    assert_eq!(lease.created_at_ticks(), Some(1_000_000_000));

    // Check validity at t = 5s (age 4s <= 10s max age) -> valid
    assert!(lease.check_valid_at(5_000_000_000).is_ok());

    // Advance clock to t = 15s (age 14s > 10s max age) -> SnapshotExpired
    clock.set_nanos(15_000_000_000);
    let res = lease.check_valid_at(clock.now_nanos());

    assert!(
        matches!(
            res,
            Err(ContextraError::SnapshotExpired {
                seq_no: 100,
                age_ticks: 14_000_000_000,
                max_age_ticks: 10_000_000_000,
            })
        ),
        "Expected SnapshotExpired error, got: {:?}",
        res
    );
}

#[test]
fn test_pin_grace_eviction_advances_retention_floor() {
    let clock = Arc::new(ManualClock::new(0));
    let registry = Arc::new(SnapshotRegistry::new().with_clock(clock.clone()));

    // Set pin grace to 30s
    registry.set_pin_grace(Duration::from_secs(30));
    assert_eq!(registry.pin_grace(), Duration::from_secs(30));

    // Pin seq 50 at t = 0s, and pin seq 100 at t = 20s
    registry.pin_at(50, 0);
    registry.pin_at(100, 20_000_000_000);

    // At t = 25s, min_active_seqno is 50
    assert_eq!(registry.min_active_seqno(), 50);

    // At t = 35s (> 30s pin grace for seq 50 pinned at t=0), evict_grace_expired_pins_at(35s)
    let evicted = registry.evict_grace_expired_pins_at(35_000_000_000);
    assert_eq!(evicted, vec![50]);

    // Retention floor (min_active_seqno) has now advanced to 100
    assert_eq!(registry.min_active_seqno(), 100);

    // Calling min_active_seqno_at(now_ticks) also triggers grace eviction
    assert_eq!(registry.min_active_seqno_at(35_000_000_000), 100);
}

#[test]
fn test_default_pin_policy_constants() {
    let registry = SnapshotRegistry::new();
    assert_eq!(
        registry.max_snapshot_age_ticks(),
        DEFAULT_MAX_SNAPSHOT_AGE_TICKS
    );
    assert_eq!(registry.pin_grace_ticks(), DEFAULT_PIN_GRACE_TICKS);
    assert_eq!(registry.max_snapshot_age(), Duration::from_secs(15 * 60));
    assert_eq!(registry.pin_grace(), Duration::from_secs(30));
}
