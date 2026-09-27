#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::{ReadSet, SequenceLogSsiValidator, SsiValidator, TxId};
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum TxEvent {
    RecordRead {
        tx_id: u64,
        key_idx: usize,
        snapshot_seq: u64,
    },
    RecordCommit {
        key_idx: usize,
        commit_seq: u64,
    },
    ValidateTx {
        tx_id: u64,
    },
}

fn simulate_sequence(
    keys: &[Vec<u8>],
    events: &[TxEvent],
) -> Vec<(u64, Result<(), String>)> {
    let validator = SequenceLogSsiValidator::new();
    let mut tx_read_sets = std::collections::HashMap::<u64, ReadSet>::new();
    let mut results = Vec::new();

    for event in events {
        match event {
            TxEvent::RecordRead {
                tx_id,
                key_idx,
                snapshot_seq,
            } => {
                let key = &keys[*key_idx % keys.len()];
                tx_read_sets
                    .entry(*tx_id)
                    .or_default()
                    .record_read(key.clone(), *snapshot_seq);
            }
            TxEvent::RecordCommit {
                key_idx,
                commit_seq,
            } => {
                let key = &keys[*key_idx % keys.len()];
                validator.record_commit_key(key, *commit_seq);
            }
            TxEvent::ValidateTx { tx_id } => {
                let read_set = tx_read_sets.get(tx_id).cloned().unwrap_or_default();
                let res = validator
                    .validate(TxId::new(*tx_id), &read_set)
                    .map_err(|e| e.to_string());
                results.push((*tx_id, res));
            }
        }
    }

    results
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_ssi_validation_is_100_percent_deterministic(
        seed_keys in proptest::collection::vec(proptest::collection::vec(0..255u8, 1..16), 1..10),
        events in proptest::collection::vec(
            prop_oneof![
                (1..10u64, 0..10usize, 1..1000u64).prop_map(|(tx_id, key_idx, snapshot_seq)| {
                    TxEvent::RecordRead { tx_id, key_idx, snapshot_seq }
                }),
                (0..10usize, 1..1000u64).prop_map(|(key_idx, commit_seq)| {
                    TxEvent::RecordCommit { key_idx, commit_seq }
                }),
                (1..10u64).prop_map(|tx_id| {
                    TxEvent::ValidateTx { tx_id }
                }),
            ],
            1..100
        )
    ) {
        // Run simulation run 1
        let run1_results = simulate_sequence(&seed_keys, &events);

        // Run simulation run 2 with identical inputs
        let run2_results = simulate_sequence(&seed_keys, &events);

        // Outcomes must be 100% identical
        prop_assert_eq!(
            run1_results,
            run2_results,
            "SSI validation must produce 100% deterministic results across repeated runs"
        );
    }
}
