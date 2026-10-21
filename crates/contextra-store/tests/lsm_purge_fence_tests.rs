use contextra_ports::StorageEngine;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use contextra_types::{Result, TxId};
use tempfile::tempdir;

#[tokio::test]
async fn test_erasure_fence_blocking() -> Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let tx1 = TxId::try_new(1).unwrap();

    // 1. Initial write permitted
    storage.put(TxId(1), b"key1", b"val1").await?;
    assert!(storage.erasure_fence().check_write_allowed(b"key1"));

    // 2. Acquire scope fence
    storage.erasure_fence().acquire_scope_fence();
    assert!(!storage.erasure_fence().check_write_allowed(b"key1"));

    // 3. Release scope fence
    storage.erasure_fence().release_scope_fence();
    assert!(storage.erasure_fence().check_write_allowed(b"key1"));

    // 4. Lock document
    storage.erasure_fence().lock_documents(&[b"key1".to_vec()]);
    assert!(!storage.erasure_fence().check_write_allowed(b"key1"));
    assert!(storage.erasure_fence().check_write_allowed(b"key2"));

    storage
        .erasure_fence()
        .unlock_documents(&[b"key1".to_vec()]);
    assert!(storage.erasure_fence().check_write_allowed(b"key1"));

    Ok(())
}
