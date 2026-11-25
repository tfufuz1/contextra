use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::config::DurabilityMode;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use tempfile::tempdir;

#[tokio::test]
async fn test_memory_only_crash_semantics() {
    let dir = tempdir().expect("Failed to create temporary directory");
    let mut config = LsmConfig::default();
    config.path = dir.path().to_path_buf();
    config.durability_mode = DurabilityMode::MemoryOnly;

    // 1. Initialize MemoryOnly storage engine
    let storage = LsmStorage::new(config.clone())
        .await
        .expect("LsmStorage initialization failed");

    let tx = TxId::new(100);
    let key = b"ephemeral_key_1";
    let val = b"ephemeral_value_data";

    // 2. Put key and commit transaction
    storage
        .put(tx, key, val)
        .await
        .expect("Put failed in MemoryOnly mode");
    storage
        .commit(tx)
        .await
        .expect("Commit failed in MemoryOnly mode");

    // 3. Verify key is accessible before restart
    let read_val = storage
        .get(key)
        .await
        .expect("Get failed")
        .expect("Key must be present in memory before restart");
    assert_eq!(read_val.as_ref(), val);

    // 4. Drop storage instance (simulating abrupt process restart/crash)
    drop(storage);

    // 5. Reopen storage engine on the same physical directory
    let reopened = LsmStorage::new(config)
        .await
        .expect("Reopening MemoryOnly storage failed");

    // 6. Verify storage is demonstrably empty after restart
    let recovered_val = reopened
        .get(key)
        .await
        .expect("Get failed on reopened storage");
    assert!(
        recovered_val.is_none(),
        "MemoryOnly storage must be empty after restart/crash, but key was found"
    );
}
