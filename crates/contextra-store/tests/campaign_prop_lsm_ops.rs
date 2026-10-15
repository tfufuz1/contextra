//! Campaign Differential Property Test Suite for `contextra-store` (LSM Storage Engine).
//! Validates stateful LSM operations (`put`, `delete`, `put_batch`, `delete_prefix`, `get`, `get_at_seq`, `scan_prefix`, `scan_range`, `commit`, `rollback`, close/reopen)
//! against an independent `BTreeMap`-based reference model tracking active transactions, MVCC versions, and committed history.

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound;
use tempfile::TempDir;

/// Independent MVCC Reference Model
#[derive(Debug, Default, Clone)]
struct ReferenceModel {
    /// Committed state: Key -> list of (seq, Option<Value>), ordered by seq asc
    committed: BTreeMap<Vec<u8>, Vec<(u64, Option<Vec<u8>>)>>,
    /// Uncommitted transaction staging: TxId -> Vec<(Key, Option<Value>)>
    active_txs: BTreeMap<u64, Vec<(Vec<u8>, Option<Vec<u8>>)>>,
}

impl ReferenceModel {
    fn new() -> Self {
        Self::default()
    }

    fn put_uncommitted(&mut self, tx_id: u64, key: Vec<u8>, val: Vec<u8>) {
        let entry = self.active_txs.entry(tx_id).or_default();
        if let Some(pos) = entry.iter().position(|(k, _)| k == &key) {
            entry[pos] = (key, Some(val));
        } else {
            entry.push((key, Some(val)));
        }
    }

    fn delete_uncommitted(&mut self, tx_id: u64, key: Vec<u8>) {
        let entry = self.active_txs.entry(tx_id).or_default();
        if let Some(pos) = entry.iter().position(|(k, _)| k == &key) {
            entry[pos] = (key, None);
        } else {
            entry.push((key, None));
        }
    }

    fn delete_prefix_uncommitted(
        &mut self,
        tx_id: u64,
        _prefix: &[u8],
        matching_committed_keys: &[Vec<u8>],
    ) {
        let entry = self.active_txs.entry(tx_id).or_default();
        for k in matching_committed_keys {
            if let Some(pos) = entry.iter().position(|(staged_k, _)| staged_k == k) {
                entry[pos] = (k.clone(), None);
            } else {
                entry.push((k.clone(), None));
            }
        }
    }

    fn commit(&mut self, tx_id: u64, starting_seq: u64) -> u64 {
        let mut curr_seq = starting_seq;
        if let Some(ops) = self.active_txs.remove(&tx_id) {
            if !ops.is_empty() {
                for (key, val) in ops {
                    curr_seq += 1;
                    let versions = self.committed.entry(key).or_default();
                    versions.push((curr_seq, val));
                }
                curr_seq += 1;
            }
        }
        curr_seq
    }

    fn rollback(&mut self, tx_id: u64) {
        self.active_txs.remove(&tx_id);
    }

    fn get_latest(&self, key: &[u8]) -> Option<Vec<u8>> {
        if let Some(versions) = self.committed.get(key) {
            if let Some((_, val_opt)) = versions.last() {
                return val_opt.clone();
            }
        }
        None
    }

    fn get_at_seq(&self, key: &[u8], target_seq: u64) -> Option<Vec<u8>> {
        if let Some(versions) = self.committed.get(key) {
            for (seq, val_opt) in versions.iter().rev() {
                if *seq <= target_seq {
                    return val_opt.clone();
                }
            }
        }
        None
    }

    fn scan_prefix(&self, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut results = Vec::new();
        for (k, versions) in &self.committed {
            if k.starts_with(prefix) {
                if let Some((_, Some(val))) = versions.last() {
                    results.push((k.clone(), val.clone()));
                }
            }
        }
        results
    }

    fn scan_range(&self, start: &[u8], end: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut results = Vec::new();
        for (k, versions) in self.committed.range(start.to_vec()..end.to_vec()) {
            if let Some((_, Some(val))) = versions.last() {
                results.push((k.clone(), val.clone()));
            }
        }
        results
    }
}

#[derive(Debug, Clone)]
enum CampaignOp {
    Put {
        tx: u64,
        key: Vec<u8>,
        val: Vec<u8>,
    },
    Delete {
        tx: u64,
        key: Vec<u8>,
    },
    PutBatch {
        tx: u64,
        items: Vec<(Vec<u8>, Vec<u8>)>,
    },
    DeletePrefix {
        tx: u64,
        prefix: Vec<u8>,
    },
    Commit {
        tx: u64,
    },
    Rollback {
        tx: u64,
    },
    Get {
        key: Vec<u8>,
    },
    GetAtSeq {
        key: Vec<u8>,
        target_seq: u64,
    },
    ScanPrefix {
        prefix: Vec<u8>,
    },
    ScanRange {
        start: Vec<u8>,
        end: Vec<u8>,
    },
    CloseAndReopen,
}

fn valid_key_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        (0..10u8).prop_map(|b| vec![b]),
        Just(vec![0xFF, 0xFF, 0xFF, 0xFF]),
        Just(vec![0x00, 0x00, 0x00]),
        "[a-z0-9_]{1,16}".prop_map(|s| s.into_bytes()),
        prop::collection::vec(any::<u8>(), 1024),
    ]
}

fn key_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![Just(vec![]), valid_key_strategy(),]
}

fn value_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        Just(vec![]),
        (0..10u8).prop_map(|b| vec![b]),
        "[a-zA-Z0-9_\n\t]{1,32}".prop_map(|s| s.into_bytes()),
        prop::collection::vec(any::<u8>(), 256),
    ]
}

fn campaign_op_strategy() -> impl Strategy<Value = CampaignOp> {
    let tx_strat = 1u64..10u64;
    prop_oneof![
        (tx_strat.clone(), valid_key_strategy(), value_strategy())
            .prop_map(|(tx, key, val)| CampaignOp::Put { tx, key, val }),
        (tx_strat.clone(), valid_key_strategy())
            .prop_map(|(tx, key)| CampaignOp::Delete { tx, key }),
        (
            tx_strat.clone(),
            prop::collection::vec((valid_key_strategy(), value_strategy()), 1..5)
        )
            .prop_map(|(tx, items)| {
                let mut seen = BTreeSet::new();
                let mut rev_items = Vec::new();
                for (k, v) in items.into_iter().rev() {
                    if seen.insert(k.clone()) {
                        rev_items.push((k, v));
                    }
                }
                rev_items.reverse();
                CampaignOp::PutBatch {
                    tx,
                    items: rev_items,
                }
            }),
        (tx_strat.clone(), valid_key_strategy())
            .prop_map(|(tx, prefix)| CampaignOp::DeletePrefix { tx, prefix }),
        tx_strat.clone().prop_map(|tx| CampaignOp::Commit { tx }),
        tx_strat.clone().prop_map(|tx| CampaignOp::Rollback { tx }),
        key_strategy().prop_map(|key| CampaignOp::Get { key }),
        (key_strategy(), 0u64..100u64)
            .prop_map(|(key, target_seq)| CampaignOp::GetAtSeq { key, target_seq }),
        key_strategy().prop_map(|prefix| CampaignOp::ScanPrefix { prefix }),
        (key_strategy(), key_strategy()).prop_map(|(k1, k2)| {
            let (start, end) = if k1 <= k2 { (k1, k2) } else { (k2, k1) };
            CampaignOp::ScanRange { start, end }
        }),
        Just(CampaignOp::CloseAndReopen),
    ]
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
    fn prop_campaign_lsm_differential_simulation(ops in proptest::collection::vec(campaign_op_strategy(), 1..100)) {
        let rt = tokio::runtime::Runtime::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
        rt.block_on(async {
            let tmp = TempDir::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
            let path = tmp.path().to_path_buf();

            let config = LsmConfig {
                path: path.clone(),
                memtable_size_limit: 2048,
                ..Default::default()
            };

            let mut storage = LsmStorage::new(config.clone())
                .await
                .map_err(|e| TestCaseError::fail(e.to_string()))?;
            let mut model = ReferenceModel::new();
            let mut model_seq = 0u64;

            for op in ops {
                match op {
                    CampaignOp::Put { tx, key, val } => {
                        let res = storage.put(TxId::new(tx), &key, &val).await;
                        if key.is_empty() || key.len() > 2048 {
                            prop_assert!(res.is_err(), "Empty/oversized key should fail validation");
                        } else {
                            res.map_err(|e| TestCaseError::fail(e.to_string()))?;
                            model.put_uncommitted(tx, key, val);
                        }
                    }
                    CampaignOp::Delete { tx, key } => {
                        let res = storage.delete(TxId::new(tx), &key).await;
                        if key.is_empty() || key.len() > 2048 {
                            prop_assert!(res.is_err(), "Empty/oversized key should fail validation");
                        } else {
                            res.map_err(|e| TestCaseError::fail(e.to_string()))?;
                            model.delete_uncommitted(tx, key);
                        }
                    }
                    CampaignOp::PutBatch { tx, items } => {
                        let res = storage.put_batch(TxId::new(tx), &items).await;
                        let has_invalid = items.iter().any(|(k, _)| k.is_empty() || k.len() > 2048);
                        if has_invalid {
                            prop_assert!(res.is_err(), "PutBatch with invalid key should fail validation");
                        } else {
                            res.map_err(|e| TestCaseError::fail(e.to_string()))?;
                            for (k, v) in items {
                                model.put_uncommitted(tx, k, v);
                            }
                        }
                    }
                    CampaignOp::DeletePrefix { tx, prefix } => {
                        let res = storage.delete_prefix(TxId::new(tx), &prefix).await;
                        if prefix.len() > 2048 {
                            prop_assert!(res.is_err(), "DeletePrefix with oversized prefix should fail validation");
                        } else {
                            res.map_err(|e| TestCaseError::fail(e.to_string()))?;
                            let matching: Vec<Vec<u8>> = model.scan_prefix(&prefix).into_iter().map(|(k, _)| k).collect();
                            model.delete_prefix_uncommitted(tx, &prefix, &matching);
                        }
                    }
                    CampaignOp::Commit { tx } => {
                        let res = storage.commit(TxId::new(tx)).await;
                        if res.is_ok() {
                            model_seq = model.commit(tx, model_seq);
                        } else {
                            model.rollback(tx);
                        }
                    }
                    CampaignOp::Rollback { tx } => {
                        storage.rollback(TxId::new(tx)).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                        model.rollback(tx);
                    }
                    CampaignOp::Get { key } => {
                        if !key.is_empty() && key.len() <= 2048 {
                            let actual = storage.get(&key).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                            let expected = model.get_latest(&key);
                            prop_assert_eq!(actual.map(|b| b.to_vec()), expected, "Mismatch on Get for key {:?}", key);
                        }
                    }
                    CampaignOp::GetAtSeq { key, target_seq } => {
                        if !key.is_empty() && key.len() <= 2048 {
                            let actual = storage.get_at_seq(&key, target_seq).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                            let expected = model.get_at_seq(&key, target_seq);
                            prop_assert_eq!(actual.map(|b| b.to_vec()), expected, "Mismatch on GetAtSeq for key {:?} at seq {}", key, target_seq);
                        }
                    }
                    CampaignOp::ScanPrefix { prefix } => {
                        if !prefix.is_empty() && prefix.len() <= 2048 {
                            let lsm_scan = storage.scan_prefix(&prefix).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                            let model_scan = model.scan_prefix(&prefix);
                            prop_assert_eq!(lsm_scan.len(), model_scan.len(), "ScanPrefix len mismatch for prefix {:?}", prefix);
                            for ((lk, lv), (mk, mv)) in lsm_scan.iter().zip(model_scan.iter()) {
                                prop_assert_eq!(lk.as_slice(), mk.as_slice());
                                prop_assert_eq!(lv.as_slice(), mv.as_slice());
                            }
                        }
                    }
                    CampaignOp::ScanRange { start, end } => {
                        if !start.is_empty() && start.len() <= 2048 && !end.is_empty() && end.len() <= 2048 && start <= end {
                            let lsm_scan = storage.scan(Bound::Included(start.as_slice()), Bound::Excluded(end.as_slice()), None)
                                .await
                                .map_err(|e| TestCaseError::fail(e.to_string()))?;
                            let model_scan = model.scan_range(&start, &end);
                            prop_assert_eq!(lsm_scan.len(), model_scan.len(), "ScanRange len mismatch for start {:?} end {:?}", start, end);
                            for ((lk, lv), (mk, mv)) in lsm_scan.iter().zip(model_scan.iter()) {
                                prop_assert_eq!(lk.as_slice(), mk.as_slice());
                                prop_assert_eq!(lv.as_slice(), mv.as_slice());
                            }
                        }
                    }
                    CampaignOp::CloseAndReopen => {
                        model.active_txs.clear();
                        drop(storage);
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        storage = LsmStorage::new(config.clone())
                            .await
                            .map_err(|e| TestCaseError::fail(e.to_string()))?;
                    }
                }
            }

            Ok(())
        })?;
    }
}

#[tokio::test]
async fn prop_counter_probe_scan_ordering_inversion() {
    let tmp = TempDir::new().unwrap();
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.unwrap();

    let tx = TxId::new(1);
    storage.put(tx, b"key_a", b"val_a").await.unwrap();
    storage.put(tx, b"key_b", b"val_b").await.unwrap();
    storage.commit(tx).await.unwrap();

    let mut scan = storage.scan_prefix(b"key_").await.unwrap();
    scan.reverse();

    let is_ordered = scan.windows(2).all(|w| w[0].0 <= w[1].0);
    assert!(
        !is_ordered,
        "R10 Counter-probe successfully detected ordering mutation!"
    );
}

#[tokio::test]
async fn prop_counter_probe_uncommitted_rollback_leakage() {
    let tmp = TempDir::new().unwrap();
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.unwrap();

    let tx = TxId::new(1);
    storage.put(tx, b"leak_key", b"leak_val").await.unwrap();
    storage.rollback(tx).await.unwrap();

    let val = storage.get(b"leak_key").await.unwrap();
    assert_eq!(
        val, None,
        "R10 Counter-probe verified rolled back transaction is not visible"
    );
}

#[tokio::test]
async fn prop_counter_probe_prefix_bound_overflow() {
    let tmp = TempDir::new().unwrap();
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.unwrap();

    let tx = TxId::new(1);
    storage.put(tx, b"pfx_1", b"v1").await.unwrap();
    storage.put(tx, b"pfy_1", b"v2").await.unwrap();
    storage.commit(tx).await.unwrap();

    let scan_pfx = storage.scan_prefix(b"pfx_").await.unwrap();
    let contains_pfy = scan_pfx.iter().any(|(k, _)| k.starts_with(b"pfy_"));
    assert!(
        !contains_pfy,
        "R10 Counter-probe verified prefix scan bounds constraint!"
    );
}
