// Campaign test for J-11-F03: memory-only-storage behavior under store engine creation.
// Oracle: StorageEngine transaction lifecycle (put + commit) vs uncommitted isolation.

use contextra_ports::storage::StorageEngine;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use contextra_types::TxId;
use tempfile::tempdir;

#[tokio::test]
async fn test_campaign_j11_memory_only_storage_contract() {
    let dir = tempdir().expect("Failed to create tempdir");
    let mut config = LsmConfig::default();
    config.path = dir.path().join("memory_test_db");

    let storage = LsmStorage::open(config).await;
    assert!(storage.is_ok(), "LsmStorage::open failed on valid temp dir");

    let engine = storage.expect("Failed to unwrap engine");

    // Verify basic key-value operations with commit
    let tx_id = TxId::new(100);
    let key = b"campaign_key";
    let val = b"campaign_value";

    assert!(engine.put(tx_id, key, val).await.is_ok());
    assert!(engine.commit(tx_id).await.is_ok());

    let fetched = engine.get(key).await;
    assert!(fetched.is_ok());
    assert_eq!(
        fetched.expect("Failed to get key"),
        Some(val.to_vec().into())
    );
}

#[tokio::test]
async fn test_campaign_j11_counter_probing_failure() {
    // Counter-probing (R10): If key is put but transaction is NOT committed, get should return None
    let dir = tempdir().expect("Failed to create tempdir");
    let mut config = LsmConfig::default();
    config.path = dir.path().join("dummy_db");

    let engine = LsmStorage::open(config)
        .await
        .expect("Failed to open storage");
    let tx_id = TxId::new(200);
    let key = b"uncommitted_key";
    let val = b"uncommitted_val";

    engine.put(tx_id, key, val).await.expect("Put failed");
    let res = engine.get(key).await.expect("Get failed");

    // Counter probe assertion check: uncommitted key MUST NOT be visible
    assert!(
        res.is_none(),
        "Uncommitted key should not be visible before commit"
    );
}
