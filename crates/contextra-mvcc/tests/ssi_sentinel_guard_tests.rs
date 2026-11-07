//! Regressionstest für Sentinel-Werte (u64::MAX / i64::MAX) und Tombstone-Bit in prune_through.
//!
//! SPEC 8.1, D-61, D-124, V-MVCC, J09.

use contextra_mvcc::error::ContextraError;
use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
use contextra_mvcc::types::{TxId, TOMBSTONE_BIT};

#[test]
fn test_ssi_prune_through_rejects_sentinel_and_tombstone() {
    let validator = SequenceLogSsiValidator::new();

    validator.record_commit_key(b"key1", 100);
    validator.record_commit_key(b"key2", 200);

    // Initial watermark
    assert_eq!(validator.pruned_through(), 0);

    // Passing u64::MAX must be ignored / return 0, without updating pruned_through
    let removed_max = validator.prune_through(u64::MAX);
    assert_eq!(removed_max, 0, "u64::MAX must not prune any keys");
    assert_eq!(
        validator.pruned_through(),
        0,
        "pruned_through must not be set to sentinel or i64::MAX"
    );

    // Passing bound with TOMBSTONE_BIT set must be ignored
    let removed_tombstone = validator.prune_through(100 | TOMBSTONE_BIT);
    assert_eq!(
        removed_tombstone, 0,
        "bound with tombstone bit must be rejected"
    );
    assert_eq!(validator.pruned_through(), 0);

    // Passing bound >= i64::MAX must be ignored
    let removed_i64_max = validator.prune_through(i64::MAX as u64);
    assert_eq!(removed_i64_max, 0, "bound >= i64::MAX must be rejected");
    assert_eq!(validator.pruned_through(), 0);

    // Valid pruning with sequence 150 must succeed
    let removed_valid = validator.prune_through(150);
    assert_eq!(removed_valid, 1, "Only key1 (commit_seq 100 <= 150) pruned");
    assert_eq!(validator.pruned_through(), 150);

    // Subsequent transaction at snapshot_seq 180 should validate key2 without false SnapshotTooOld
    let tx = TxId::new(42);
    let mut rs = ReadSet::new();
    rs.record_read(b"key2", 180);
    // Key2 was committed at 200 > snapshot_seq 180, so conflict is detected properly, but NOT SnapshotTooOld
    let res = validator.validate(tx, &rs);
    assert!(matches!(res, Err(ContextraError::Conflict(_))));
}
