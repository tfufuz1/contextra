//! Integration test for `LsmStorage::open_with_merge_operator`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{Result, StorageEngine, TxId};
use contextra_store::compaction::CompactionConfig;
use contextra_store::{LsmConfig, LsmStorage, MergeOperator};
use std::sync::Arc;
use tempfile::TempDir;

/// U64 sum merge operator adding two 8-byte little-endian integers.
struct U64SumMergeOperator;

impl MergeOperator for U64SumMergeOperator {
    fn merge(&self, existing_val: &[u8], new_val: &[u8]) -> Result<Vec<u8>> {
        let a = u64::from_le_bytes(existing_val.try_into().map_err(|_| {
            contextra_core::ContextraError::InvalidInput("invalid u64 bytes".into())
        })?);
        let b = u64::from_le_bytes(new_val.try_into().map_err(|_| {
            contextra_core::ContextraError::InvalidInput("invalid u64 bytes".into())
        })?);
        Ok((a.wrapping_add(b)).to_le_bytes().to_vec())
    }
}

#[tokio::test]
async fn test_lsm_storage_with_merge_operator_sums_values_on_compaction() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        compaction: CompactionConfig {
            min_sstables_per_tier: 2,
            ..Default::default()
        },
        ..Default::default()
    };

    let operator = Arc::new(U64SumMergeOperator);
    let storage = Arc::new(
        LsmStorage::open_with_merge_operator(config, operator)
            .await
            .expect("open_with_merge_operator"),
    );

    // 1. Put key_sum = 10 (u64 LE) and force flush to 1st SSTable
    let tx1 = TxId::new(10);
    storage
        .put(tx1, b"key_sum", &10u64.to_le_bytes())
        .await
        .expect("put tx1");
    storage.commit(tx1).await.expect("commit tx1");
    storage.force_flush().await.expect("flush 1");

    // 2. Put key_sum = 20 (u64 LE) and force flush to 2nd SSTable
    let tx2 = TxId::new(20);
    storage
        .put(tx2, b"key_sum", &20u64.to_le_bytes())
        .await
        .expect("put tx2");
    storage.commit(tx2).await.expect("commit tx2");
    storage.force_flush().await.expect("flush 2");

    // 3. Trigger compaction via public API
    let compacted = storage.maybe_compact().await.expect("maybe_compact");
    assert!(compacted, "Compaction should have occurred");

    // 4. Read key_sum -> must return 30 (10 + 20)
    let res = storage.get(b"key_sum").await.expect("get key_sum");
    assert!(res.is_some(), "key_sum should exist");

    let val_bytes = res.unwrap();
    let sum = u64::from_le_bytes(val_bytes[..8].try_into().unwrap());
    assert_eq!(sum, 30, "Merged value must equal 30 (10 + 20), got {}", sum);
}

#[tokio::test]
async fn test_lsm_storage_without_merge_operator_replaces_old_value() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        compaction: CompactionConfig {
            min_sstables_per_tier: 2,
            ..Default::default()
        },
        ..Default::default()
    };

    let storage = Arc::new(
        LsmStorage::open(config)
            .await
            .expect("open without operator"),
    );

    // 1. Put key_sum = 10 (u64 LE) and force flush to 1st SSTable
    let tx1 = TxId::new(10);
    storage
        .put(tx1, b"key_sum", &10u64.to_le_bytes())
        .await
        .expect("put tx1");
    storage.commit(tx1).await.expect("commit tx1");
    storage.force_flush().await.expect("flush 1");

    // 2. Put key_sum = 20 (u64 LE) and force flush to 2nd SSTable
    let tx2 = TxId::new(20);
    storage
        .put(tx2, b"key_sum", &20u64.to_le_bytes())
        .await
        .expect("put tx2");
    storage.commit(tx2).await.expect("commit tx2");
    storage.force_flush().await.expect("flush 2");

    // 3. Trigger compaction via public API
    let compacted = storage.maybe_compact().await.expect("maybe_compact");
    assert!(compacted, "Compaction should have occurred");

    // 4. Read key_sum -> without merge operator, newest value (20) replaces old value
    let res = storage.get(b"key_sum").await.expect("get key_sum");
    assert!(res.is_some(), "key_sum should exist");

    let val_bytes = res.unwrap();
    let val = u64::from_le_bytes(val_bytes[..8].try_into().unwrap());
    assert_eq!(
        val, 20,
        "Without merge operator, newest value (20) replaces old value, got {}",
        val
    );
}
