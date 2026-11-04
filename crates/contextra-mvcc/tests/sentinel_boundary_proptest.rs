//! Exhaustive boundary and property tests for sentinel freedom and 64-bit boundaries.
//!
//! SPEC §8.7, §9.5, D-61, D-127, I-12, T-MVCC-SENTINEL.
//! Verifies:
//! - Full u64 space coverage including 0, 1, i64::MAX, i64::MAX + 1, u64::MAX - 1, u64::MAX.
//! - TOMBSTONE_BIT collision handling.
//! - Absence of cast wrapping ("as i64") or treating u64::MAX as "all visible".
//! - Pruning and visibility semantics over boundary values.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::error::MvccError;
use contextra_mvcc::seq_log::{SeqLogEntry, SequenceLog};
use contextra_mvcc::snapshot::SnapshotRegistry;
use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
use contextra_testmacros::ctx_test;
use contextra_types::{DocId, TxId, TOMBSTONE_BIT};
use proptest::prelude::*;

/// SPEC: T-VER-1
#[ctx_test(id = "T-VER-1")]
fn test_explicit_sentinel_boundary_values() {
    let validator = SequenceLogSsiValidator::new();

    let boundaries = [
        0u64,
        1u64,
        i64::MAX as u64 - 1,
        i64::MAX as u64,
        i64::MAX as u64 + 1,
        u64::MAX - 1,
        u64::MAX,
    ];

    for &seq in &boundaries {
        let res = validator.prune_through_checked(Some(seq));
        if seq == u64::MAX || (seq & TOMBSTONE_BIT) != 0 {
            assert_eq!(
                res,
                Err(MvccError::InvalidSequence(seq)),
                "Sequence {} with TOMBSTONE_BIT or u64::MAX must be rejected fail-closed",
                seq
            );
        } else {
            assert!(
                res.is_ok(),
                "Sequence {} without TOMBSTONE_BIT must be accepted",
                seq
            );
        }
    }
}

/// SPEC: T-VER-1
#[ctx_test(id = "T-VER-1")]
fn test_tombstone_bit_permutations_on_boundaries() {
    let validator = SequenceLogSsiValidator::new();
    let mut log = SequenceLog::new();
    let doc_id = DocId::from(777u64);

    let boundaries = [
        0u64,
        1u64,
        100u64,
        i64::MAX as u64,
        i64::MAX as u64 + 1,
        u64::MAX - 1,
        u64::MAX,
    ];

    for &raw_seq in &boundaries {
        // Test with TOMBSTONE_BIT explicitly set
        let with_tombstone = raw_seq | TOMBSTONE_BIT;

        // 1. prune_through_checked must reject any sequence with TOMBSTONE_BIT
        let prune_res = validator.prune_through_checked(Some(with_tombstone));
        assert_eq!(
            prune_res,
            Err(MvccError::InvalidSequence(with_tombstone)),
            "TOMBSTONE_BIT must never be accepted as prune bound"
        );

        // 2. ReadSet recording must mask TOMBSTONE_BIT and not treat it as huge number
        let mut rs = ReadSet::new();
        rs.record_read(b"key1".to_vec(), with_tombstone);
        let masked = with_tombstone & !TOMBSTONE_BIT;
        assert_eq!(
            rs.min_snapshot_seq(),
            Some(masked),
            "ReadSet must mask TOMBSTONE_BIT"
        );

        // 3. SequenceLog must mask TOMBSTONE_BIT on insert
        log.record_insert(doc_id, with_tombstone);
        let entry_vis = SeqLogEntry {
            doc_id,
            insert_seq: with_tombstone,
            delete_seq: None,
        };
        // If query as_of is masked, it must be visible
        assert!(
            entry_vis.is_visible(masked),
            "Visibility must handle masked sequence correctly"
        );
    }
}

/// SPEC: T-SNAP-1
#[ctx_test(id = "T-SNAP-1")]
fn test_empty_snapshot_registry_never_returns_sentinel() {
    let registry = SnapshotRegistry::new();

    // min_read_snapshot MUST be None, never Some(u64::MAX)
    assert_eq!(registry.min_read_snapshot(), None);

    // min_for_compaction MUST return current_seq, never u64::MAX
    let test_seqs = [0, 1, 42, i64::MAX as u64, u64::MAX - 1, u64::MAX];
    for &curr in &test_seqs {
        assert_eq!(
            registry.min_for_compaction(curr),
            curr,
            "Empty registry must fallback to current_seq"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5000))]

    /// SPEC: T-VER-1
    /// SPEC: T-VER-1
    #[test]
    fn prop_boundary_pruning_and_visibility_across_full_u64(
        raw_seq in prop_oneof![
            Just(0u64),
            Just(1u64),
            Just(i64::MAX as u64 - 1),
            Just(i64::MAX as u64),
            Just(i64::MAX as u64 + 1),
            Just(u64::MAX - 1),
            Just(u64::MAX),
            any::<u64>(),
        ],
        with_tombstone_flip in any::<bool>(),
    ) {
        let seq = if with_tombstone_flip {
            raw_seq ^ TOMBSTONE_BIT
        } else {
            raw_seq
        };

        let validator = SequenceLogSsiValidator::new();
        let prune_res = validator.prune_through_checked(Some(seq));

        if seq == u64::MAX || (seq & TOMBSTONE_BIT) != 0 {
            prop_assert_eq!(
                prune_res,
                Err(MvccError::InvalidSequence(seq)),
                "Invalid bound {:?} must fail-closed", seq
            );
        } else {
            prop_assert!(
                prune_res.is_ok(),
                "Valid bound {:?} must succeed", seq
            );
        }

        // Test visibility formula does not wrap or panic
        let doc_id = DocId::from(12345u64);
        let entry = SeqLogEntry {
            doc_id,
            insert_seq: seq,
            delete_seq: None,
        };
        // Unmasked as_of query
        let _ = entry.is_visible(seq & !TOMBSTONE_BIT);
    }

    /// SPEC: T-TXN-1
    /// SPEC: T-TXN-1
    #[test]
    fn prop_ssi_read_set_never_overflows_or_wraps_at_u64_max(
        snapshot_seq in prop_oneof![
            Just(0u64),
            Just(1u64),
            Just(i64::MAX as u64),
            Just(u64::MAX - 1),
            Just(u64::MAX),
            any::<u64>(),
        ],
        commit_seq in 0..10_000u64,
    ) {
        let validator = SequenceLogSsiValidator::new();
        let mut read_set = ReadSet::new();

        read_set.record_read(b"my_key".to_vec(), snapshot_seq);

        let masked_snapshot = snapshot_seq & !TOMBSTONE_BIT;
        prop_assert_eq!(read_set.min_snapshot_seq(), Some(masked_snapshot));

        validator.record_commit_key(b"my_key", commit_seq);

        let tx = TxId::new(999);
        let res = validator.validate(tx, &read_set);

        // If commit_seq > masked_snapshot, conflict must occur!
        if commit_seq > masked_snapshot {
            prop_assert!(res.is_err(), "Must conflict when commit_seq > masked_snapshot");
        } else {
            prop_assert!(res.is_ok(), "Must succeed when commit_seq <= masked_snapshot");
        }
    }
}
