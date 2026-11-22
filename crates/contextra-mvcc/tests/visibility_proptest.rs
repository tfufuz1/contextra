#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::{DocId, SeqLogEntry, SequenceLog};
use proptest::prelude::*;

/// Trivial reference implementation of the MVCC visibility formula:
/// visible(e, as_of) <==> e.insert_seq <= as_of AND (e.delete_seq == None OR e.delete_seq > as_of)
fn reference_visibility(insert_seq: u64, delete_seq: Option<u64>, as_of: u64) -> bool {
    insert_seq <= as_of && delete_seq.is_none_or(|del| del > as_of)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn prop_mvcc_visibility_matches_reference_10k(
        insert_seq in 0..10_000u64,
        delete_opt in prop_oneof![
            Just(None),
            (0..10_000u64).prop_map(Some)
        ],
        as_of in 0..10_000u64,
    ) {
        let doc_id = DocId::from(42u64);
        let entry = SeqLogEntry {
            doc_id,
            insert_seq,
            delete_seq: delete_opt,
        };

        // 1. Check direct SeqLogEntry::is_visible
        let actual_entry_vis = entry.is_visible(as_of);
        let expected_vis = reference_visibility(insert_seq, delete_opt, as_of);

        prop_assert_eq!(
            actual_entry_vis,
            expected_vis,
            "SeqLogEntry::is_visible mismatch for insert_seq={}, delete_seq={:?}, as_of={}",
            insert_seq,
            delete_opt,
            as_of
        );

        // 2. Check via SequenceLog if delete_seq >= insert_seq
        if delete_opt.is_none_or(|del| del >= insert_seq) {
            let mut log = SequenceLog::new();
            log.record_insert(doc_id, insert_seq);
            if let Some(del) = delete_opt {
                log.record_delete(doc_id, del);
            }

            let actual_log_vis = log.is_visible(doc_id, as_of);
            prop_assert_eq!(
                actual_log_vis,
                expected_vis,
                "SequenceLog::is_visible mismatch for insert_seq={}, delete_seq={:?}, as_of={}",
                insert_seq,
                delete_opt,
                as_of
            );
        }
    }

    /// Explicit boundary testing: as_of == insert_seq, as_of == delete_seq, delete_seq == None
    #[test]
    fn prop_mvcc_visibility_explicit_boundary_cases(
        seq_a in 0..5_000u64,
        seq_b in 5_000..10_000u64,
        scenario in 0..4u8,
    ) {
        let (insert_seq, delete_opt, as_of) = match scenario {
            // Case 0: as_of == insert_seq, delete_seq == None -> MUST be visible
            0 => (seq_a, None, seq_a),
            // Case 1: as_of == delete_seq -> MUST NOT be visible (delete_seq > as_of is strict)
            1 => (seq_a, Some(seq_b), seq_b),
            // Case 2: as_of == insert_seq with future delete_seq -> MUST be visible
            2 => (seq_a, Some(seq_b), seq_a),
            // Case 3: delete_seq == None, as_of > insert_seq -> MUST be visible
            _ => (seq_a, None, seq_b),
        };

        let doc_id = DocId::from(100u64);
        let entry = SeqLogEntry {
            doc_id,
            insert_seq,
            delete_seq: delete_opt,
        };

        let expected = reference_visibility(insert_seq, delete_opt, as_of);
        prop_assert_eq!(entry.is_visible(as_of), expected);

        if scenario == 1 {
            // Explicit check: delete_seq == as_of -> MUST be false
            prop_assert!(!entry.is_visible(as_of), "as_of == delete_seq must evaluate to NOT visible");
        } else if scenario == 0 || scenario == 2 {
            // Explicit check: as_of == insert_seq -> MUST be true
            prop_assert!(entry.is_visible(as_of), "as_of == insert_seq must evaluate to visible");
        }
    }
}
