// FILE-CONTEXT
// STAND: 2026-09-28T20:00:00Z
// ZWECK: Modellbasierte Property-Tests fuer LSM MVCC & Flush Invarianten gegen ReferenceModel.
// DOKUMENTATION: PROPTEST_CASES ist standardmaessig auf 64 gesetzt (optimiert fuer Jules-VM). Per Environment Variable `PROPTEST_CASES=10000` auf 10 000 Cases fuer Nachtlaeufe erhoehbar.

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_testkit::{RefOp, ReferenceModel};
use proptest::prelude::*;
use tempfile::TempDir;

#[derive(Debug, Clone)]
enum ModelOp {
    Put(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
    Commit,
    Rollback,
    Flush,
    Compact,
    SnapshotPin,
    SnapshotRead(Vec<u8>),
}

fn model_op_strategy() -> impl Strategy<Value = ModelOp> {
    prop_oneof![
        (prop::collection::vec(0..4u8, 1..4), prop::collection::vec(0..10u8, 1..8))
            .prop_map(|(k, v)| ModelOp::Put(k, v)),
        prop::collection::vec(0..4u8, 1..4).prop_map(ModelOp::Delete),
        Just(ModelOp::Commit),
        Just(ModelOp::Rollback),
        Just(ModelOp::Flush),
        Just(ModelOp::Compact),
        Just(ModelOp::SnapshotPin),
        prop::collection::vec(0..4u8, 1..4).prop_map(ModelOp::SnapshotRead),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROPTEST_CASES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(64),
        .. ProptestConfig::default()
    })]

    #[test]
    fn test_model_based_lsm_operations(ops in prop::collection::vec(model_op_strategy(), 1..25)) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let runtime = match runtime {
            Ok(rt) => rt,
            Err(e) => return Err(TestCaseError::fail(e.to_string())),
        };

        let result: Result<(), TestCaseError> = runtime.block_on(async move {
            let temp_dir = match TempDir::new() {
                Ok(td) => td,
                Err(e) => return Err(TestCaseError::fail(e.to_string())),
            };
            let config = LsmConfig {
                path: temp_dir.path().to_path_buf(),
                ..Default::default()
            };

            let storage = match LsmStorage::new(config.clone()).await {
                Ok(s) => s,
                Err(e) => return Err(TestCaseError::fail(e.to_string())),
            };
            let mut model = ReferenceModel::new();

            let mut staged_ops = Vec::new();
            let mut current_tx = TxId::new(1);
            let mut pinned_snapshot_seq: Option<u64> = None;

            for op in ops {
                match op {
                    ModelOp::Put(ref k, ref v) => {
                        model.put(k.clone(), v.clone());
                        staged_ops.push(RefOp::Put(k.clone(), v.clone()));
                        if let Err(e) = storage.put(current_tx, k, v).await {
                            return Err(TestCaseError::fail(e.to_string()));
                        }
                    }
                    ModelOp::Delete(ref k) => {
                        model.delete(k.clone());
                        staged_ops.push(RefOp::Delete(k.clone()));
                        if let Err(e) = storage.delete(current_tx, k).await {
                            return Err(TestCaseError::fail(e.to_string()));
                        }
                    }
                    ModelOp::Commit => {
                        if let Err(e) = storage.commit(current_tx).await {
                            return Err(TestCaseError::fail(e.to_string()));
                        }
                        let seq = model.commit();
                        staged_ops.clear();
                        current_tx = TxId::new(current_tx.inner() + 1);

                        // Invariante 1: Snapshot-Abfrage gegen Storage matcht ReferenceModel
                        let keys = model.keys_at(seq);
                        for key in keys {
                            let expected = model.get_at(&key, seq);
                            let actual = match storage.get(&key).await {
                                Ok(val) => val.map(|b| b.to_vec()),
                                Err(e) => return Err(TestCaseError::fail(e.to_string())),
                            };
                            prop_assert_eq!(expected, actual, "Snapshot read mismatch for key {:?}", key);
                        }
                    }
                    ModelOp::Rollback => {
                        if let Err(e) = storage.rollback(current_tx).await {
                            return Err(TestCaseError::fail(e.to_string()));
                        }
                        model.rollback();
                        staged_ops.clear();
                        current_tx = TxId::new(current_tx.inner() + 1);
                    }
                    ModelOp::Flush => {
                        if let Err(e) = storage.force_flush().await {
                            return Err(TestCaseError::fail(e.to_string()));
                        }

                        // Invariante 3: Nach Flush aendern sich alle Lesergebnisse der bisher bestaetigten Commits nicht
                        let current_seq = model.snapshot_seq();
                        if current_seq > 0 {
                            let keys = model.keys_at(current_seq);
                            for key in keys {
                                let expected = model.get_at(&key, current_seq);
                                let actual = match storage.get(&key).await {
                                    Ok(val) => val.map(|b| b.to_vec()),
                                    Err(e) => return Err(TestCaseError::fail(e.to_string())),
                                };
                                prop_assert_eq!(expected, actual, "Post-flush read mismatch for key {:?}", key);
                            }
                        }
                    }
                    ModelOp::Compact => {
                        if let Err(e) = storage.maybe_compact().await {
                            return Err(TestCaseError::fail(e.to_string()));
                        }
                    }
                    ModelOp::SnapshotPin => {
                        pinned_snapshot_seq = Some(model.snapshot_seq());
                    }
                    ModelOp::SnapshotRead(ref key) => {
                        if let Some(pin_seq) = pinned_snapshot_seq {
                            let expected = model.get_at(key, pin_seq);
                            if pin_seq == model.snapshot_seq() {
                                let actual = match storage.get(key).await {
                                    Ok(val) => val.map(|b| b.to_vec()),
                                    Err(e) => return Err(TestCaseError::fail(e.to_string())),
                                };
                                prop_assert_eq!(expected, actual, "Pinned snapshot read mismatch for key {:?}", key);
                            }
                        }
                    }
                }
            }

            // Invariante 2: Reopen check
            drop(storage);
            let reopened_storage = match LsmStorage::new(config).await {
                Ok(s) => s,
                Err(e) => return Err(TestCaseError::fail(e.to_string())),
            };
            let seq = model.snapshot_seq();
            if seq > 0 {
                for key in model.keys_at(seq) {
                    let expected = model.get_at(&key, seq);
                    let actual = match reopened_storage.get(&key).await {
                        Ok(val) => val.map(|b| b.to_vec()),
                        Err(e) => return Err(TestCaseError::fail(e.to_string())),
                    };
                    prop_assert_eq!(expected, actual, "Post-reopen read mismatch for key {:?}", key);
                }
            }

            Ok(())
        });

        result?;
    }
}
