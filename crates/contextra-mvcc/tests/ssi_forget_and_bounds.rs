use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
use contextra_types::{ContextraError, TxId};

#[test]
fn forget_from_removes_keys_of_failed_batch() {
    let validator = SequenceLogSsiValidator::new();
    let tx1 = TxId::new(1);

    validator.record_commit_key(b"key1", 5);
    validator.record_commit_key(b"key2", 10);
    validator.record_commit_key(b"key3", 12);

    assert_eq!(validator.tracked_commit_keys(), 3);

    // Forget from seq 10 onwards (keys recorded for seq 10 and 12 should be removed)
    let removed = validator.forget_from(10);
    assert_eq!(removed, 2);
    assert_eq!(validator.tracked_commit_keys(), 1);

    // Validation for key1 at snapshot_seq 4 should still conflict (committed at 5)
    let mut rs1 = ReadSet::new();
    rs1.record_read(b"key1", 4);
    assert!(validator.validate(tx1, &rs1).is_err());

    // Validation for key2 at snapshot_seq 8 should succeed because key2 was removed by forget_from
    let mut rs2 = ReadSet::new();
    rs2.record_read(b"key2", 8);
    assert!(validator.validate(tx1, &rs2).is_ok());
}

#[test]
fn committed_writes_bounded() {
    let max_keys = 5;
    let validator = SequenceLogSsiValidator::new_with_bounds(max_keys);

    for i in 0..1_000_000 {
        validator.record_commit_key(format!("key_{i}").as_bytes(), i as u64 + 1);
        if (i + 1) % 5 == 0 {
            let removed = validator.prune_through((i + 1) as u64);
            assert!(removed > 0);
        }
    }

    // When capacity is reached without pruning:
    let bounded_val = SequenceLogSsiValidator::new_with_bounds(2);
    bounded_val.record_commit_key(b"k1", 10);
    bounded_val.record_commit_key(b"k2", 11);

    let tx = TxId::new(42);
    let mut rs = ReadSet::new();
    rs.record_read(b"non_existent_key", 100);

    // Because tracked_commit_keys >= max_keys (2 >= 2), validate must fail closed
    let res = bounded_val.validate(tx, &rs);
    assert!(matches!(res, Err(ContextraError::Conflict(_))));
}

#[test]
fn phantom_prefix_conflict() {
    let validator = SequenceLogSsiValidator::new();
    let tx_a = TxId::new(100);

    // Tx A reads prefix "user:" at snapshot_seq 10
    let mut rs_a = ReadSet::new();
    rs_a.record_prefix(b"user:".to_vec(), 10);

    // Tx B commits key "user:42" at commit_seq 15 (> snapshot_seq 10)
    validator.record_commit_key(b"user:42", 15);

    // Tx A validation must detect phantom conflict
    let res = validator.validate(tx_a, &rs_a);
    assert!(matches!(res, Err(ContextraError::Conflict(_))));

    // A commit without prefix match (e.g. "order:1") at commit_seq 15 should not conflict with prefix "user:"
    let validator2 = SequenceLogSsiValidator::new();
    validator2.record_commit_key(b"order:1", 15);
    assert!(validator2.validate(tx_a, &rs_a).is_ok());
}
