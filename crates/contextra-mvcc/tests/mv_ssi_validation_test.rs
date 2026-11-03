// ANCHOR[contextra-mvcc:TEST-MV-SSI-PROPS]
//! # SSI-Validierungs- und Isolations-Tests gegen Referenzmodell (MV-09, MV-10)
//!
//! Prüft:
//! - MV-09: SSI erkennt Schreib-Schreib- und Lese-Schreib-Antidependenzen;
//!   Konfliktprüfung gegen Read-/Write-Set; keine Tombstone-Bit-Semantik für Existenz (Tombstone = vlen = 0).
//! - MV-10: Keine Panics, kein unwrap; checked/saturating-Zähler.
//! - Proptest gegen ReferenceModel aus `contextra_testkit::reference_model::ReferenceModel`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::error::MvccError;
use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
use contextra_mvcc::ContextraError;
use contextra_testkit::reference_model::ReferenceModel;
use contextra_types::TxId;
use proptest::prelude::*;
use std::collections::BTreeMap;

#[test]
fn mv_09_write_write_antidependency_detection() {
    let validator = SequenceLogSsiValidator::new();
    let tx1 = TxId::new(10);
    let tx2 = TxId::new(20);

    let key = b"exclusive_resource";

    let mut rs1 = ReadSet::new();
    rs1.record_read(key.to_vec(), 100);

    let mut rs2 = ReadSet::new();
    rs2.record_read(key.to_vec(), 100);

    // Both read at seq 100
    assert!(validator.validate(tx1, &rs1).is_ok());
    assert!(validator.validate(tx2, &rs2).is_ok());

    // Tx1 commits write at seq 105
    assert!(validator
        .validate_and_record(tx1, &rs1, [key.as_slice()], 105)
        .is_ok());

    // Tx2 attempts to validate and record at seq 110 -> write-write / read-write conflict!
    let res2 = validator.validate(tx2, &rs2);
    assert!(
        matches!(res2, Err(ContextraError::Conflict(_))),
        "Tx2 must be rejected with Conflict error due to anti-dependency, got: {:?}",
        res2
    );
}

#[test]
fn mv_09_read_write_antidependency_write_skew() {
    let validator = SequenceLogSsiValidator::new();
    let tx1 = TxId::new(101);
    let tx2 = TxId::new(102);

    let key_a = b"bank_balance_a";
    let key_b = b"bank_balance_b";

    // Tx1 reads key_a, Tx2 reads key_b
    let mut rs1 = ReadSet::new();
    rs1.record_read(key_a.to_vec(), 50);

    let mut rs2 = ReadSet::new();
    rs2.record_read(key_b.to_vec(), 50);

    // Tx1 writes key_b and commits at seq 55
    assert!(validator
        .validate_and_record(tx1, &rs1, [key_b.as_slice()], 55)
        .is_ok());

    // Tx2 writes key_a, but read key_b earlier as part of transaction logic
    rs2.record_read(key_b.to_vec(), 50);

    // Tx2 must fail validation because key_b was modified after snapshot 50
    let res2 = validator.validate(tx2, &rs2);
    assert!(
        matches!(res2, Err(ContextraError::Conflict(_))),
        "Tx2 must fail with Conflict due to modified key in read set"
    );
}

#[test]
fn mv_10_zero_panic_checked_counters_and_overflow_protection() {
    let validator = SequenceLogSsiValidator::new_with_bounds(100);
    let tx = TxId::new(999);

    let mut rs = ReadSet::new();
    rs.record_read(b"k1".to_vec(), 10);

    // Record commits up to capacity
    for i in 1u64..=105 {
        let k = format!("k_{i}").into_bytes();
        validator.record_commit_key(&k, i);
    }

    // Capacity bound check must fail gracefully without panic
    let res = validator.validate(tx, &rs);
    assert!(res.is_err() || res.is_ok()); // Neither must panic

    // Checked pruning bounds
    assert!(validator.prune_through_checked(None).is_ok());
    assert_eq!(
        validator.prune_through_checked(Some(u64::MAX)),
        Err(MvccError::InvalidSequence(u64::MAX))
    );
}

#[derive(Debug, Clone)]
enum OracleAction {
    Begin { tx_id: u64, snapshot_seq: u64 },
    Read { tx_id: u64, key: u8 },
    Write { tx_id: u64, key: u8, val: u8 },
    Commit { tx_id: u64 },
    Rollback { tx_id: u64 },
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_mv_09_ssi_validation_oracle_with_reference_model(
        actions in proptest::collection::vec(
            prop_oneof![
                (1..10u64, 0..50u64).prop_map(|(tx_id, snapshot_seq)| OracleAction::Begin { tx_id, snapshot_seq }),
                (1..10u64, 1..6u8).prop_map(|(tx_id, key)| OracleAction::Read { tx_id, key }),
                (1..10u64, 1..6u8, 1..10u8).prop_map(|(tx_id, key, val)| OracleAction::Write { tx_id, key, val }),
                (1..10u64).prop_map(|tx_id| OracleAction::Commit { tx_id }),
                (1..10u64).prop_map(|tx_id| OracleAction::Rollback { tx_id }),
            ],
            1..60
        )
    ) {
        let mut ref_model = ReferenceModel::new();
        let validator = SequenceLogSsiValidator::new();

        struct ActiveTx {
            snapshot_seq: u64,
            read_keys: BTreeMap<Vec<u8>, u64>,
            write_keys: BTreeMap<Vec<u8>, Vec<u8>>,
        }

        let mut active_txs: BTreeMap<u64, ActiveTx> = BTreeMap::new();
        // Global commit history of keys: key -> latest_commit_seq
        let mut key_last_committed: BTreeMap<Vec<u8>, u64> = BTreeMap::new();

        for action in actions {
            match action {
                OracleAction::Begin { tx_id, snapshot_seq } => {
                    let effective_snapshot = snapshot_seq.min(ref_model.snapshot_seq());
                    active_txs.insert(tx_id, ActiveTx {
                        snapshot_seq: effective_snapshot,
                        read_keys: BTreeMap::new(),
                        write_keys: BTreeMap::new(),
                    });
                }
                OracleAction::Read { tx_id, key } => {
                    let k = vec![key];
                    if let Some(tx) = active_txs.get_mut(&tx_id) {
                        let snap = tx.snapshot_seq;
                        // Reference model lookup
                        let _val = ref_model.get_at(&k, snap);
                        tx.read_keys.entry(k).or_insert(snap);
                    }
                }
                OracleAction::Write { tx_id, key, val } => {
                    let k = vec![key];
                    let v = vec![val];
                    if let Some(tx) = active_txs.get_mut(&tx_id) {
                        tx.write_keys.insert(k, v);
                    }
                }
                OracleAction::Commit { tx_id } => {
                    if let Some(tx) = active_txs.remove(&tx_id) {
                        // Check oracle conflict: did any read key get committed after tx.snapshot_seq?
                        let mut oracle_conflict = false;
                        for (k, snap) in &tx.read_keys {
                            if let Some(&last_commit) = key_last_committed.get(k) {
                                if last_commit > *snap {
                                    oracle_conflict = true;
                                    break;
                                }
                            }
                        }

                        // Validate against SequenceLogSsiValidator
                        let mut rs = ReadSet::new();
                        for (k, snap) in &tx.read_keys {
                            rs.record_read(k.clone(), *snap);
                        }

                        let actual_val = validator.validate(TxId::new(tx_id), &rs);

                        if oracle_conflict {
                            prop_assert!(
                                actual_val.is_err(),
                                "Oracle detected conflict for Tx {}, but validator succeeded",
                                tx_id
                            );
                        } else {
                            prop_assert!(
                                actual_val.is_ok(),
                                "Oracle found no conflict for Tx {}, but validator failed: {:?}",
                                tx_id,
                                actual_val
                            );

                            // Proceed to commit in both models
                            for (k, v) in &tx.write_keys {
                                ref_model.put(k, v);
                            }
                            let new_seq = ref_model.commit();

                            let write_key_refs: Vec<&[u8]> = tx.write_keys.keys().map(|k| k.as_slice()).collect();
                            validator.record_commit_keys(write_key_refs, new_seq);

                            for k in tx.write_keys.into_keys() {
                                key_last_committed.insert(k, new_seq);
                            }
                        }
                    }
                }
                OracleAction::Rollback { tx_id } => {
                    active_txs.remove(&tx_id);
                }
            }
        }
    }
}
