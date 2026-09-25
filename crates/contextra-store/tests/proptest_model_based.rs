use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use std::collections::BTreeMap;
use std::ops::Bound;
use tempfile::TempDir;

#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_failing_proptest_sequence() -> contextra_core::Result<()> {
    let tmp = TempDir::new().unwrap();
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024,
        ..Default::default()
    };

    let storage = LsmStorage::new(config.clone()).await?;

    // 1. Delete(0)
    let tx1 = TxId::new(1);
    storage.delete(tx1, b"prop_k_0").await?;
    storage.commit(tx1).await?;

    // 2. Put(1, 0)
    let tx2 = TxId::new(2);
    storage.put(tx2, b"prop_k_1", b"prop_v_0").await?;
    storage.commit(tx2).await?;

    // 3. Put(1, 0)
    let tx3 = TxId::new(3);
    storage.put(tx3, b"prop_k_1", b"prop_v_0").await?;
    storage.commit(tx3).await?;

    // 4. Flush
    storage.force_flush().await?;

    // 5. Restart
    drop(storage);
    let storage = LsmStorage::new(config.clone()).await?;

    let val_k1 = storage.get(b"prop_k_1").await?;
    assert_eq!(
        val_k1,
        Some(bytes::Bytes::from_static(b"prop_v_0")),
        "prop_k_1 must equal prop_v_0 after restart"
    );

    Ok(())
}

fn operation_strategy() -> impl Strategy<Value = Operation> {
    prop_oneof![
        (0..50u8, 0..100u8).prop_map(|(k, v)| Operation::Put(k, v)),
        (0..50u8).prop_map(Operation::Delete),
        Just(Operation::Flush),
        Just(Operation::Compact),
        Just(Operation::Restart),
        (0..50u8, 0..50u8).prop_map(|(k1, k2)| Operation::Scan(k1, k2)),
        (0..50u8, 0..100u8).prop_map(|(k, v)| Operation::PutIfAbsent(k, v)),
        (0..50u8).prop_map(Operation::GetOrDefault),
    ]
}

#[derive(Debug, Clone)]
enum Operation {
    Put(u8, u8),
    Delete(u8),
    Flush,
    Compact,
    Restart,
    Scan(u8, u8),
    PutIfAbsent(u8, u8),
    GetOrDefault(u8),
}

async fn verify_storage_state(
    storage: &LsmStorage,
    shadow: &BTreeMap<Vec<u8>, Vec<u8>>,
) -> Result<(), TestCaseError> {
    for (key, expected_val) in shadow {
        let actual_val = storage
            .get(key)
            .await
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(
            actual_val.as_deref(),
            Some(expected_val.as_slice()),
            "Proptest mismatch for key {:?}",
            String::from_utf8_lossy(key)
        );
    }

    for k_byte in 0..50u8 {
        let key = format!("prop_k_{}", k_byte).into_bytes();
        if !shadow.contains_key(&key) {
            let actual_val = storage
                .get(&key)
                .await
                .map_err(|e| TestCaseError::fail(e.to_string()))?;
            prop_assert_eq!(
                actual_val,
                None,
                "Key {:?} expected deleted but found in LSM",
                String::from_utf8_lossy(&key)
            );
        }
    }

    let lsm_all = storage
        .scan(Bound::Unbounded, Bound::Unbounded, None)
        .await
        .map_err(|e| TestCaseError::fail(e.to_string()))?;
    let model_all: Vec<_> = shadow.iter().collect();
    prop_assert_eq!(lsm_all.len(), model_all.len());
    for ((mk, mv), (lk, lv)) in model_all.iter().zip(lsm_all.iter()) {
        prop_assert_eq!(*mk, lk);
        prop_assert_eq!(*mv, lv);
    }

    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROPTEST_CASES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(200),
        ..ProptestConfig::default()
    })]
    #[test]
    fn prop_model_based_lsm_simulation(ops in proptest::collection::vec(operation_strategy(), 1..500)) {
        let rt = tokio::runtime::Runtime::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
        rt.block_on(async {
            let tmp = TempDir::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
            let path = tmp.path().to_path_buf();

            let config = LsmConfig {
                path: path.clone(),
                memtable_size_limit: 1024,
                ..Default::default()
            };

            let mut storage = LsmStorage::new(config.clone())
                .await
                .map_err(|e| TestCaseError::fail(e.to_string()))?;
            let mut shadow = BTreeMap::<Vec<u8>, Vec<u8>>::new();
            let mut tx_counter = 1u64;

            for op in ops {
                match op {
                    Operation::Put(k_byte, v_byte) => {
                        let key = format!("prop_k_{}", k_byte).into_bytes();
                        let val = format!("prop_v_{}", v_byte).into_bytes();
                        let tx = TxId::new(tx_counter);
                        tx_counter += 1;

                        storage
                            .put(tx, &key, &val)
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                        storage
                            .commit(tx)
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                        shadow.insert(key, val);
                    }
                    Operation::Delete(k_byte) => {
                        let key = format!("prop_k_{}", k_byte).into_bytes();
                        let tx = TxId::new(tx_counter);
                        tx_counter += 1;

                        storage
                            .delete(tx, &key)
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                        storage
                            .commit(tx)
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                        shadow.remove(&key);
                    }
                    Operation::Flush => {
                        storage
                            .force_flush()
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                    }
                    Operation::Compact => {
                        storage
                            .maybe_compact()
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                    }
                    Operation::Restart => {
                        drop(storage);
                        storage = LsmStorage::new(config.clone())
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                        verify_storage_state(&storage, &shadow).await?;
                    }
                    Operation::Scan(k_start, k_end) => {
                        let s1 = format!("prop_k_{}", k_start).into_bytes();
                        let s2 = format!("prop_k_{}", k_end).into_bytes();
                        let (start, end) = if s1 <= s2 { (s1, s2) } else { (s2, s1) };
                        let lsm_results = storage
                            .scan(
                                Bound::Included(start.as_slice()),
                                Bound::Excluded(end.as_slice()),
                                None,
                            )
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                        let model_results: Vec<_> = shadow.range(start..end).collect();
                        prop_assert_eq!(lsm_results.len(), model_results.len());
                        for ((mk, mv), (lk, lv)) in model_results.iter().zip(lsm_results.iter()) {
                            prop_assert_eq!(*mk, lk);
                            prop_assert_eq!(*mv, lv);
                        }
                    }
                    Operation::PutIfAbsent(k_byte, v_byte) => {
                        let key = format!("prop_k_{}", k_byte).into_bytes();
                        let val = format!("prop_v_{}", v_byte).into_bytes();
                        let tx = TxId::new(tx_counter);
                        tx_counter += 1;

                        let inserted = storage
                            .put_if_absent(tx, &key, &val)
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                        if inserted {
                            storage
                                .commit(tx)
                                .await
                                .map_err(|e| TestCaseError::fail(e.to_string()))?;
                            shadow.insert(key, val);
                        }
                    }
                    Operation::GetOrDefault(k_byte) => {
                        let key = format!("prop_k_{}", k_byte).into_bytes();
                        let actual_val = storage
                            .get(&key)
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?
                            .map(|b| b.to_vec())
                            .unwrap_or_default();
                        let expected_val = shadow.get(&key).cloned().unwrap_or_default();
                        prop_assert_eq!(actual_val, expected_val);
                    }
                }
            }

            verify_storage_state(&storage, &shadow).await?;
            Ok(())
        })?;
    }
}
