//! Hardening and concurrency tests for Serializable Snapshot Isolation (SSI) pruning, limits, and watermarks.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::error::ContextraError;
use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
use contextra_mvcc::tx_buffer::TxBuffer;
use contextra_mvcc::types::TxId;
use std::sync::{Arc, Barrier};
use std::time::Duration;

#[test]
fn test_validate_and_record_atomicity() {
    for _ in 0..500 {
        let validator = SequenceLogSsiValidator::new();
        let barrier = Arc::new(Barrier::new(2));

        // Thread A: reads key_x at seq 10, writes key_y at seq 20
        let v_a = validator.clone();
        let b_a = barrier.clone();
        let handle_a = std::thread::spawn(move || {
            let mut rs_a = ReadSet::new();
            rs_a.record_read(b"key_x".to_vec(), 10);
            b_a.wait();
            v_a.validate_and_record(TxId::new(1), &rs_a, vec![b"key_y".as_slice()], 20)
        });

        // Thread B: reads key_y at seq 10, writes key_x at seq 20
        let v_b = validator.clone();
        let b_b = barrier.clone();
        let handle_b = std::thread::spawn(move || {
            let mut rs_b = ReadSet::new();
            rs_b.record_read(b"key_y".to_vec(), 10);
            b_b.wait();
            v_b.validate_and_record(TxId::new(2), &rs_b, vec![b"key_x".as_slice()], 20)
        });

        let res_a = handle_a.join().expect("thread A join");
        let res_b = handle_b.join().expect("thread B join");

        let ok_count = [res_a.is_ok(), res_b.is_ok()]
            .iter()
            .filter(|&&ok| ok)
            .count();
        assert_eq!(
            ok_count, 1,
            "Exactly one transaction must succeed in write-skew scenario"
        );
    }
}

#[test]
fn test_prune_through_mechanics() {
    let validator = SequenceLogSsiValidator::new();
    validator.record_commit_key(b"key1", 10);
    validator.record_commit_key(b"key2", 20);
    validator.record_commit_key(b"key3", 30);

    assert_eq!(validator.tracked_commit_keys(), 3);
    assert_eq!(validator.pruned_through(), 0);

    let removed = validator.prune_through(20);
    assert_eq!(removed, 2);
    assert_eq!(validator.tracked_commit_keys(), 1);
    assert_eq!(validator.pruned_through(), 20);

    // Monotonicity check: smaller bound does not lower watermark or remove entries
    let removed_2 = validator.prune_through(15);
    assert_eq!(removed_2, 0);
    assert_eq!(validator.pruned_through(), 20);
}

#[test]
fn test_fail_closed_validation() {
    let validator = SequenceLogSsiValidator::new();
    validator.record_commit_key(b"key1", 5);
    validator.prune_through(10);

    let mut old_rs = ReadSet::new();
    old_rs.record_read(b"key1".to_vec(), 5);

    let res_old = validator.validate(TxId::new(1), &old_rs);
    assert!(res_old.is_err());
    if let Err(ContextraError::Conflict(msg)) = res_old {
        assert!(
            msg.contains("snapshot too old"),
            "Expected 'snapshot too old' in error message: {msg}"
        );
    } else {
        panic!("Expected ContextraError::Conflict");
    }

    let mut valid_rs = ReadSet::new();
    valid_rs.record_read(b"key1".to_vec(), 10);
    let res_valid = validator.validate(TxId::new(2), &valid_rs);
    assert!(res_valid.is_ok());
}

#[test]
fn test_min_read_snapshot_across_shards() {
    let buffer = TxBuffer::<String>::new_with_config(64, Duration::from_secs(30));
    assert_eq!(buffer.min_read_snapshot(), None);

    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);

    buffer.begin(tx1);
    buffer.begin(tx2);

    buffer.register_read(tx1, b"k1".to_vec(), 100);
    buffer.register_read(tx2, b"k2".to_vec(), 50);

    assert_eq!(buffer.min_read_snapshot(), Some(50));

    buffer.discard(tx2);
    assert_eq!(buffer.min_read_snapshot(), Some(100));

    buffer.drain(tx1);
    assert_eq!(buffer.min_read_snapshot(), None);
}

#[test]
fn test_try_register_read_limit_and_updates() {
    let buffer = TxBuffer::<String>::new();
    let tx = TxId::new(100);
    buffer.begin(tx);

    assert!(buffer.try_register_read(tx, b"k1".to_vec(), 10, 2).is_ok());
    assert!(buffer.try_register_read(tx, b"k2".to_vec(), 20, 2).is_ok());

    // 3rd new key fails when max_keys is 2
    let res_overflow = buffer.try_register_read(tx, b"k3".to_vec(), 30, 2);
    assert!(matches!(
        res_overflow,
        Err(ContextraError::LimitExceeded { .. })
    ));

    let rs = buffer.get_read_set(tx).unwrap();
    assert_eq!(rs.len(), 2);
    assert_eq!(rs.get(b"k3"), None);

    // Updating existing key k1 is allowed even at max_keys limit
    assert!(buffer.try_register_read(tx, b"k1".to_vec(), 5, 2).is_ok());
    let rs_updated = buffer.get_read_set(tx).unwrap();
    assert_eq!(rs_updated.get(b"k1"), Some(5));
}

proptest::proptest! {
    #[test]
    fn prop_prune_equivalence_above_watermark(
        commits in proptest::collection::vec((proptest::collection::vec(0..255u8, 1..8), 1..100u64), 1..30),
        prune_bound in 1..50u64,
        read_key in proptest::collection::vec(0..255u8, 1..8),
        read_snapshot_offset in 0..50u64
    ) {
        let snapshot_seq = prune_bound + read_snapshot_offset;

        let val_unpruned = SequenceLogSsiValidator::new();
        let val_pruned = SequenceLogSsiValidator::new();

        for (key, seq) in &commits {
            val_unpruned.record_commit_key(key, *seq);
            val_pruned.record_commit_key(key, *seq);
        }

        val_pruned.prune_through(prune_bound);

        let mut rs = ReadSet::new();
        rs.record_read(read_key, snapshot_seq);

        let res_unpruned = val_unpruned.validate(TxId::new(100), &rs);
        let res_pruned = val_pruned.validate(TxId::new(100), &rs);

        match (res_unpruned, res_pruned) {
            (Ok(()), Ok(())) => {},
            (Err(ContextraError::Conflict(_)), Err(ContextraError::Conflict(_))) => {},
            (un, pr) => proptest::prop_assert!(false, "Mismatch above watermark: unpruned={:?}, pruned={:?}", un, pr),
        }
    }
}
