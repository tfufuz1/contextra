use contextra_core::{StorageEngine, TxId};
use contextra_store::compaction::CompactionConfig;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use std::collections::BTreeMap;
use std::ops::Bound;
use tempfile::TempDir;

fn operation_strategy() -> impl Strategy<Value = Operation> {
    prop_oneof![
        (0..30u8, 0..100u8).prop_map(|(k, v)| Operation::Put(k, v)),
        (0..30u8).prop_map(Operation::Delete),
        Just(Operation::Flush),
        Just(Operation::Compact),
        Just(Operation::Restart),
        (0..30u8, 0..30u8).prop_map(|(k1, k2)| Operation::Scan(k1, k2)),
        (0..30u8).prop_map(Operation::Get),
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
    Get(u8),
}

async fn verify_storage_state(
    storage: &LsmStorage,
    shadow: &BTreeMap<Vec<u8>, Vec<u8>>,
) -> Result<(), TestCaseError> {
    // 1. Verify get against shadow model
    for (key, expected_val) in shadow {
        let actual_val = storage
            .get(key)
            .await
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(
            actual_val.as_deref(),
            Some(expected_val.as_slice()),
            "Mismatch for key {:?}",
            String::from_utf8_lossy(key)
        );
    }

    // 2. Verify absent keys return None
    for k_byte in 0..30u8 {
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

    // 3. Verify get_at_seq at current max_seq matches shadow model
    let max_seq = storage.last_applied_seq();
    for k_byte in 0..30u8 {
        let key = format!("prop_k_{}", k_byte).into_bytes();
        let expected_val = shadow.get(&key).cloned();
        let actual_val = storage
            .get_at_seq(&key, max_seq)
            .await
            .map_err(|e| TestCaseError::fail(e.to_string()))?
            .map(|b| b.to_vec());
        prop_assert_eq!(
            actual_val,
            expected_val,
            "get_at_seq at max_seq {} mismatch for key {:?}",
            max_seq,
            String::from_utf8_lossy(&key)
        );
    }

    // 4. Verify scan across entire range
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
            .unwrap_or(30),
        ..ProptestConfig::default()
    })]
    #[test]
    fn prop_compaction_overlap_scan_and_tombstone(ops in proptest::collection::vec(operation_strategy(), 1..100)) {
        let rt = tokio::runtime::Runtime::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
        rt.block_on(async {
            let tmp = TempDir::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
            let path = tmp.path().to_path_buf();

            let config = LsmConfig {
                path: path.clone(),
                memtable_size_limit: 1024,
                compaction: CompactionConfig {
                    min_sstables_per_tier: 2,
                    size_ratio: 4.0,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut storage = Some(LsmStorage::new(config.clone())
                .await
                .map_err(|e| TestCaseError::fail(e.to_string()))?);
            let mut shadow = BTreeMap::<Vec<u8>, Vec<u8>>::new();
            let mut tx_counter = 1u64;

            for op in ops {
                let st = storage.as_ref().unwrap();
                match op {
                    Operation::Put(k_byte, v_byte) => {
                        let key = format!("prop_k_{}", k_byte).into_bytes();
                        let val = format!("prop_v_{}", v_byte).into_bytes();
                        let tx = TxId::new(tx_counter);
                        tx_counter += 1;

                        st.put(tx, &key, &val)
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                        st.commit(tx)
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;

                        shadow.insert(key, val);
                    }
                    Operation::Delete(k_byte) => {
                        let key = format!("prop_k_{}", k_byte).into_bytes();
                        let tx = TxId::new(tx_counter);
                        tx_counter += 1;

                        st.delete(tx, &key)
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                        st.commit(tx)
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;

                        shadow.remove(&key);
                    }
                    Operation::Flush => {
                        st.force_flush()
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                    }
                    Operation::Compact => {
                        st.maybe_compact()
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                        verify_storage_state(st, &shadow).await?;
                    }
                    Operation::Restart => {
                        let st_instance = storage.take().unwrap();
                        let _ = st_instance.close().await;
                        drop(st_instance);
                        tokio::task::yield_now().await;
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                        let new_st = LsmStorage::new(config.clone())
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                        verify_storage_state(&new_st, &shadow).await?;
                        storage = Some(new_st);
                    }
                    Operation::Scan(k_start, k_end) => {
                        let s1 = format!("prop_k_{}", k_start).into_bytes();
                        let s2 = format!("prop_k_{}", k_end).into_bytes();
                        let (start, end) = if s1 <= s2 { (s1, s2) } else { (s2, s1) };
                        let lsm_results = st
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
                    Operation::Get(k_byte) => {
                        let key = format!("prop_k_{}", k_byte).into_bytes();
                        let actual_val = st
                            .get(&key)
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?
                            .map(|b| b.to_vec());
                        let expected_val = shadow.get(&key).cloned();
                        prop_assert_eq!(actual_val, expected_val);
                    }
                }
            }

            if let Some(st) = storage.as_ref() {
                verify_storage_state(st, &shadow).await?;
            }
            if let Some(st) = storage.take() {
                let _ = st.close().await;
            }
            Ok::<(), TestCaseError>(())
        })?;
    }
}
