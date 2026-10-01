#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::{ReadSet, SequenceLogSsiValidator, SsiValidator, TxId};

#[test]
fn test_b16_reproduction_capacity_total_outage_resolved() {
    let max_keys = 100;
    let validator = SequenceLogSsiValidator::new_with_bounds(max_keys);

    // Fill the register to capacity (100 keys committed across sequence numbers 1..=100)
    for i in 1..=max_keys {
        let key = format!("user:{i:04}");
        validator.record_commit_key(key.as_bytes(), i as u64);
    }

    // Now attempt a transaction that reads key "unrelated:key" at snapshot_seq 200 (> all commit seqs 1..=100).
    // In the fixed code with coarsening, this non-conflicting read MUST succeed (no total outage!).
    let tx = TxId::new(999);
    let mut rs = ReadSet::new();
    rs.record_read(b"unrelated:key", 200);

    let res = validator.validate(tx, &rs);
    assert!(
        res.is_ok(),
        "B-16 resolution check failed: validate returned error for non-conflicting read: {:?}",
        res
    );
}

#[test]
fn test_b16_coarsening_prevents_false_negatives() {
    let max_keys = 10;
    let validator = SequenceLogSsiValidator::new_with_bounds(max_keys);

    // Commit keys at seq 10 to trigger coarsening
    for i in 1..=10 {
        let key = format!("account:{i:03}");
        validator.record_commit_key(key.as_bytes(), 10);
    }

    // Confirm pruning blocker diagnostics shows coarsened buckets
    let blocker = validator.diagnose_pruning_blocker().expect("blocker info");
    assert!(blocker.coarsened_seq_buckets > 0);

    // Test 1: Real conflict MUST STILL BE DETECTED (0% false negatives!)
    // Transaction read key "account:005" at snapshot_seq 5 (< commit_seq 10).
    let tx_conflict = TxId::new(101);
    let mut rs_conflict = ReadSet::new();
    rs_conflict.record_read(b"account:005", 5);

    let res_conflict = validator.validate(tx_conflict, &rs_conflict);
    assert!(
        res_conflict.is_err(),
        "Coarsening flaw: expected conflict to be detected for account:005 read at seq 5 < commit_seq 10"
    );

    // Test 2: Non-conflicting read of "account:005" at snapshot_seq 15 (>= commit_seq 10) succeeds
    let tx_ok = TxId::new(102);
    let mut rs_ok = ReadSet::new();
    rs_ok.record_read(b"account:005", 15);

    assert!(validator.validate(tx_ok, &rs_ok).is_ok());
}

#[test]
fn test_long_lived_snapshot_pin_blocker_alarm() {
    use contextra_mvcc::SnapshotRegistry;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    let registry = Arc::new(SnapshotRegistry::new());
    let now = Instant::now();
    let past_350s = now.checked_sub(Duration::from_secs(350)).unwrap_or(now);

    // Register a snapshot guard at past_350s (350s ago > DEFAULT_MAX_PIN_DURATION of 300s)
    let _guard = registry.register_at(42, past_350s);

    let validator = SequenceLogSsiValidator::new_with_bounds(10)
        .with_snapshot_registry(registry.clone());

    // Record keys to trigger coarsening check / diagnostic check
    for i in 1..=10 {
        let key = format!("data:{i:02}");
        validator.record_commit_key(key.as_bytes(), 50 + i as u64);
    }

    // Query pruning blocker diagnostics at `now`
    let blocker = validator
        .diagnose_pruning_blocker_at(now)
        .expect("pruning blocker info must be present");

    assert_eq!(blocker.longest_active_snapshot_seq, Some(42));
    assert!(blocker.longest_pin_duration.unwrap() >= Duration::from_secs(350));
    assert!(
        blocker.is_pin_expired,
        "Snapshot active for 350s must be marked as expired pin"
    );
}

#[test]
fn test_fail_closed_snapshot_pruned_still_enforced_after_coarsening() {
    let validator = SequenceLogSsiValidator::new_with_bounds(10);

    // Commit keys to fill register
    for i in 1..=10 {
        let key = format!("item:{i:02}");
        validator.record_commit_key(key.as_bytes(), 20);
    }

    // Prune up to sequence 15
    validator.prune_through(15);
    assert_eq!(validator.pruned_through(), 15);

    // Try validating a transaction with snapshot_seq 10 (< pruned_through 15)
    let tx = TxId::new(500);
    let mut rs = ReadSet::new();
    rs.record_read(b"item:01", 10);

    let res = validator.validate(tx, &rs);
    assert!(
        res.is_err(),
        "Fail-closed check failed: snapshot_seq 10 < pruned_through 15 MUST return conflict error"
    );
}
