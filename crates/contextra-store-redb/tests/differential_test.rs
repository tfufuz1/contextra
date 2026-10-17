//! Differential tests comparing `RedbStorageEngine` against reference storage implementation.

use std::collections::BTreeMap;
use tempfile::TempDir;

use contextra_ports::StorageEngine;
use contextra_store_redb::RedbStorageEngine;
use contextra_types::TxId;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

#[derive(Debug, Clone)]
enum Op {
    Put(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
    Get(Vec<u8>),
    ScanPrefix(Vec<u8>),
    Commit,
    Rollback,
}

fn op_strategy() -> impl Strategy<Value = Op> {
    let key_strat = prop::collection::vec(any::<u8>(), 1..=16);
    let val_strat = prop::collection::vec(any::<u8>(), 1..=64);

    prop_oneof![
        (key_strat.clone(), val_strat).prop_map(|(k, v)| Op::Put(k, v)),
        key_strat.clone().prop_map(Op::Delete),
        key_strat.clone().prop_map(Op::Get),
        key_strat.clone().prop_map(Op::ScanPrefix),
        Just(Op::Commit),
        Just(Op::Rollback),
    ]
}

async fn run_differential_ops(ops: Vec<Op>) -> Result<(), TestCaseError> {
    let dir = TempDir::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
    let db_path = dir.path().join("diff.redb");

    let store = RedbStorageEngine::open(&db_path).map_err(|e| TestCaseError::fail(e.to_string()))?;

    let mut ref_committed: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
    let mut ref_staged: BTreeMap<Vec<u8>, Option<Vec<u8>>> = BTreeMap::new();

    let mut current_tx = TxId(1);

    for op in ops {
        match op {
            Op::Put(k, v) => {
                store
                    .put(current_tx, &k, &v)
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                ref_staged.insert(k, Some(v));
            }
            Op::Delete(k) => {
                store
                    .delete(current_tx, &k)
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                ref_staged.insert(k, None);
            }
            Op::Get(k) => {
                let store_val = store.get(&k).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                let ref_val = ref_committed.get(&k).cloned().map(bytes::Bytes::from);

                if store_val != ref_val {
                    return Err(TestCaseError::fail(format!(
                        "Get divergence for key {:?}: store = {:?}, ref = {:?}",
                        k, store_val, ref_val
                    )));
                }
            }
            Op::ScanPrefix(prefix) => {
                let store_scans = store
                    .scan_prefix(&prefix)
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;

                let ref_scans: Vec<(Vec<u8>, Vec<u8>)> = ref_committed
                    .iter()
                    .filter(|(k, _)| k.starts_with(&prefix))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                if store_scans != ref_scans {
                    return Err(TestCaseError::fail(format!(
                        "ScanPrefix divergence for prefix {:?}: store = {:?}, ref = {:?}",
                        prefix, store_scans, ref_scans
                    )));
                }
            }
            Op::Commit => {
                store.commit(current_tx).await.map_err(|e| TestCaseError::fail(e.to_string()))?;

                for (k, v_opt) in std::mem::take(&mut ref_staged) {
                    match v_opt {
                        Some(v) => {
                            ref_committed.insert(k, v);
                        }
                        None => {
                            ref_committed.remove(&k);
                        }
                    }
                }

                current_tx = TxId(current_tx.0 + 1);
            }
            Op::Rollback => {
                store
                    .rollback(current_tx)
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                ref_staged.clear();
            }
        }
    }

    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn differential_redb_vs_reference_model(
        ops in prop::collection::vec(op_strategy(), 1..100)
    ) {
        let rt = tokio::runtime::Runtime::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
        rt.block_on(async {
            run_differential_ops(ops).await
        })?;
    }
}
