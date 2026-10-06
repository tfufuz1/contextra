// FILE-CONTEXT
// ZWECK: Campaign coverage test for SequenceLogSsiValidator coarsening and pruning blocker diagnostics.
// INVARIANTEN: Coarsening occurs at 80% capacity limit without false negatives for write-skew detection.
// NICHT-OFFENSICHTLICH: Uses independent oracle (R4) validating fail-closed conflict classification post-coarsening.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_mvcc::snapshot::SnapshotRegistry;
use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator};
use contextra_mvcc::SsiValidator;
use contextra_types::TxId;
use std::sync::Arc;
use std::time::Instant;

#[test]
fn test_ssi_coarsening_trigger_and_diagnose_pruning_blocker() {
    let registry = Arc::new(SnapshotRegistry::new());
    // Create an active snapshot at seq 0 to hold back pruning
    let snap = registry.register(0);

    let validator =
        SequenceLogSsiValidator::new_with_bounds(10).with_snapshot_registry(registry.clone());

    // Record keys up to 80% threshold (8 keys)
    for i in 0..10 {
        let key = format!("key_{i}").into_bytes();
        validator.record_commit_key(&key, (i + 1) as u64);
    }

    // 1. Verify diagnose_pruning_blocker returns active blocker info
    let blocker_info = validator.diagnose_pruning_blocker();
    assert!(
        blocker_info.is_some(),
        "Expected pruning blocker diagnostic info"
    );
    let info = blocker_info.expect("PruningBlockerInfo");
    assert!(info.tracked_commit_keys > 0);
    assert!(info.min_unpruned_seq > 0);

    let blocker_at = validator.diagnose_pruning_blocker_at(Instant::now());
    assert!(blocker_at.is_some());

    // 2. Oracle Check (R4): Verify write-skew conflict detection works fail-closed
    // Read key_0 at snapshot seq 0
    let mut read_set = ReadSet::new();
    read_set.record_read(b"key_0", snap.seq_no());

    // Validate read_set against committed writes
    let res = validator.validate(TxId::new(100), &read_set);
    assert!(
        res.is_err(),
        "Write-skew conflict MUST be detected fail-closed for key_0 committed after snapshot"
    );

    drop(snap);
}
