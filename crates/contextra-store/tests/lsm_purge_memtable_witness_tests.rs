use contextra_ports::StorageEngine;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::witness::MemScopeSpec;
use contextra_store::lsm::LsmStorage;
use contextra_types::{Result, TxId};
use tempfile::tempdir;

#[tokio::test]
async fn test_memtable_residue_scanning() -> Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    // Write keys via storage engine / transaction
    storage.put(TxId(1), b"key1", b"val1").await?;
    storage.commit(TxId(1)).await?;

    storage.delete(TxId(2), b"key1").await?;
    storage.commit(TxId(2)).await?;

    let key_vec = vec![b"key1".to_vec()];
    let evidence = storage
        .scan_memtable_residue(MemScopeSpec::Keys(&key_vec), 100)
        .await?;

    assert_eq!(evidence.entries_seen, 2);
    assert_eq!(evidence.versions_found, 1); // 1 non-tombstone (val1 at seq 1)

    // Issue witness should fail due to non-tombstone version at seq 1
    let witness_res = storage
        .issue_memtable_witness(&key_vec, &[], TxId(100))
        .await;
    assert!(witness_res.is_err());

    Ok(())
}
