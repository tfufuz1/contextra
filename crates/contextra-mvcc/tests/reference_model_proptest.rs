// ANCHOR[contextra-mvcc:TEST-PROPTEST-REFERENCE-MODEL]
//! # Property-Tests gegen ein unabhängiges Referenzmodell
//!
//! Prüft Snapshot-Isolation und SSI-Konflikterkennung (Write-Skew) gegen ein
//! einfaches BTreeMap-basiertes Referenzmodell.

use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
use contextra_types::TxId;
use proptest::prelude::*;
use std::collections::BTreeMap;

/// Unabhängiges In-Memory-Referenzmodell für Transaktionen und SSI.
#[derive(Default)]
struct ReferenceModel {
    /// key -> list of (commit_seq, value)
    store: BTreeMap<Vec<u8>, Vec<(u64, Vec<u8>)>>,
    /// key -> latest commit_seq
    latest_commit: BTreeMap<Vec<u8>, u64>,
}

impl ReferenceModel {
    fn commit(&mut self, seq: u64, writes: &[(Vec<u8>, Vec<u8>)]) {
        for (k, v) in writes {
            self.store
                .entry(k.clone())
                .or_default()
                .push((seq, v.clone()));
            self.latest_commit.insert(k.clone(), seq);
        }
    }

    /// SSI-Validierungs-Orakel: Prüft, ob ein gelesener Schlüssel nach snapshot_seq committet wurde.
    fn would_conflict(&self, reads: &[(Vec<u8>, u64)]) -> bool {
        for (k, snapshot_seq) in reads {
            if let Some(&latest) = self.latest_commit.get(k) {
                if latest > *snapshot_seq {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
enum TxAction {
    Read {
        key: Vec<u8>,
        snapshot_seq: u64,
    },
    Commit {
        seq: u64,
        key: Vec<u8>,
        val: Vec<u8>,
    },
}

proptest! {
    #[test]
    fn prop_ssi_validation_matches_reference_model(
        actions in proptest::collection::vec(
            prop_oneof![
                (1..100u64, 1..5u8).prop_map(|(seq, k)| TxAction::Read {
                    key: vec![k],
                    snapshot_seq: seq,
                }),
                (1..100u64, 1..5u8, 1..10u8).prop_map(|(seq, k, v)| TxAction::Commit {
                    seq,
                    key: vec![k],
                    val: vec![v],
                }),
            ],
            1..40
        )
    ) {
        let mut model = ReferenceModel::default();
        let validator = SequenceLogSsiValidator::new();

        // Separate past commits from current transaction reads
        let mut pending_reads = Vec::new();
        let mut current_read_set = ReadSet::new();

        for action in actions {
            match action {
                TxAction::Commit { seq, key, val } => {
                    model.commit(seq, &[(key.clone(), val)]);
                    validator.record_commit_key(&key, seq);
                }
                TxAction::Read { key, snapshot_seq } => {
                    pending_reads.push((key.clone(), snapshot_seq));
                    current_read_set.record_read(key, snapshot_seq);
                }
            }
        }

        let ref_conflict = model.would_conflict(&pending_reads);
        let actual_result = validator.validate(TxId::new(42), &current_read_set);

        if ref_conflict {
            prop_assert!(
                actual_result.is_err(),
                "Reference model detected write-skew conflict, but validator succeeded"
            );
        } else {
            prop_assert!(
                actual_result.is_ok(),
                "Reference model found no conflict, but validator failed: {:?}",
                actual_result
            );
        }
    }

    #[test]
    fn prop_prune_through_checked_invariants(
        commits in proptest::collection::vec(1..500u64, 1..30),
        prune_target in 0..1000u64,
    ) {
        let validator = SequenceLogSsiValidator::new();
        let mut max_commit = 0;

        for &c in &commits {
            validator.record_commit_key(b"k", c);
            if c > max_commit {
                max_commit = c;
            }
        }

        let res = validator.prune_through_checked(Some(prune_target));
        prop_assert!(res.is_ok());

        let pt = validator.pruned_through();
        // Invariant: pruned_through <= max_commit
        prop_assert!(
            pt <= max_commit,
            "pruned_through ({}) must never exceed max_commit ({})",
            pt,
            max_commit
        );
        // Invariant: pruned_through <= prune_target
        prop_assert!(
            pt <= prune_target,
            "pruned_through ({}) must never exceed prune_target ({})",
            pt,
            prune_target
        );
    }
}
