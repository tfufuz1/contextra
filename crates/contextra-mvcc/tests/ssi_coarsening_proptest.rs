#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::{ReadSet, SequenceLogSsiValidator, SsiValidator, TxId};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_ssi_coarsening_zero_false_negatives(
        commit_keys in proptest::collection::vec("[a-z]{3}:[0-9]{3}", 20..100),
        read_keys in proptest::collection::vec("[a-z]{3}:[0-9]{3}", 5..20),
        snapshot_seq in 1u64..50u64,
        commit_seq in 51u64..100u64,
    ) {
        // 1. Uncoarsened reference validator (large capacity)
        let ref_validator = SequenceLogSsiValidator::new_with_bounds(1_000_000);

        // 2. Coarsened validator under test (small capacity to force coarsening)
        let coarsened_validator = SequenceLogSsiValidator::new_with_bounds(10);

        // Record commits in both validators
        for key in &commit_keys {
            ref_validator.record_commit_key(key.as_bytes(), commit_seq);
            coarsened_validator.record_commit_key(key.as_bytes(), commit_seq);
        }

        // Build read set at snapshot_seq < commit_seq
        let tx = TxId::new(12345);
        let mut read_set = ReadSet::new();
        for key in &read_keys {
            read_set.record_read(key.as_bytes(), snapshot_seq);
        }

        let ref_res = ref_validator.validate(tx, &read_set);
        let coarsened_res = coarsened_validator.validate(tx, &read_set);

        // ZERO FALSE NEGATIVES INVARIANT:
        // If reference model detects a conflict (ref_res is Err), coarsened_validator MUST ALSO detect a conflict (coarsened_res is Err).
        // (False positives are allowed: coarsened_res may be Err even if ref_res is Ok).
        if ref_res.is_err() {
            prop_assert!(
                coarsened_res.is_err(),
                "False negative detected! Reference validator found conflict for read_set at snapshot_seq {} vs commit_seq {}, but coarsened validator returned Ok()",
                snapshot_seq,
                commit_seq
            );
        }
    }
}
