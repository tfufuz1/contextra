// ANCHOR[contextra-mvcc:TEST-B16-ROOT]
//! # Regressionstests für Befund B-16 (Kapazitätserschöpfung und Coarsening)
//!
//! Verifiziert, dass das Erreichen von `DEFAULT_MAX_TRACKED_COMMIT_KEYS` nicht mehr
//! zu einem Totalausfall führt, sondern ältere Buckets konservativ vergröbert werden,
//! während 0 % False Negatives garantiert bleiben.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::{ReadSet, SequenceLogSsiValidator, SsiValidator, TxId};

// Prüft, dass das Erreichen der maximalen Schlüsselkapazität durch Coarsening gelöst wird und nicht blockiert.
#[test]
fn test_b16_reproduction_capacity_total_outage_resolved() {
    let max_keys = 100;
    let validator = SequenceLogSsiValidator::new_with_bounds(max_keys);

    // Fill the register to capacity (100 keys committed across sequence numbers 1..=100)
    for i in 1..=max_keys {
        let key = format!("user:{i:04}");
        validator.record_commit_key(key.as_bytes(), u64::try_from(i).expect("valid u64"));
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

// Prüft, dass vergröberte Buckets echte Schreibkonflikte weiterhin zuverlässig (ohne False Negatives) abfangen.
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

// Prüft, dass langlaufende gepinnte Snapshots, die das Pruning behindern, als Diagnosealarm erkannt werden.
#[test]
fn test_long_lived_snapshot_pin_blocker_alarm() {
    use contextra_mvcc::SnapshotRegistry;
    use contextra_testkit::ManualClock;
    use std::sync::Arc;
    use std::time::Duration;

    let clock = Arc::new(ManualClock::new(400_000_000_000));
    let registry = Arc::new(SnapshotRegistry::with_clock(clock.clone()));

    let now_nanos = clock.now_nanos();
    let past_350s_nanos = now_nanos.saturating_sub(350_000_000_000);

    // Register a snapshot guard at 350s ago (> DEFAULT_MAX_PIN_DURATION of 300s)
    let _guard = registry.register_at(42, past_350s_nanos);

    let validator = SequenceLogSsiValidator::new_with_bounds(10)
        .with_clock(clock.clone())
        .with_snapshot_registry(registry.clone());

    // Record keys to trigger coarsening check / diagnostic check
    for i in 1..=10 {
        let key = format!("data:{i:02}");
        validator.record_commit_key(key.as_bytes(), 50 + u64::try_from(i).expect("valid u64"));
    }

    // Query pruning blocker diagnostics
    let blocker = validator
        .diagnose_pruning_blocker_at(now_nanos)
        .expect("pruning blocker info must be present");

    assert_eq!(blocker.longest_active_snapshot_seq, Some(42));
    assert!(blocker.longest_pin_duration.unwrap() >= Duration::from_secs(350));
    assert!(
        blocker.is_pin_expired,
        "Snapshot active for 350s must be marked as expired pin"
    );
}

// Prüft, dass das Fail-Closed-Verhalten für Lese-Snapshots älter als pruned_through auch nach Coarsening greift.
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
