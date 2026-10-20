//! Storage conformance tests for `RedbStorageEngine`.

use bytes::Bytes;
use tempfile::tempdir;

use contextra_ports::StorageEngine;
use contextra_store_redb::RedbStorageEngine;
use contextra_types::TxId;

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_redb_store_put_get_commit_contract() -> TestResult {
    let dir = tempdir()?;
    let db_path = dir.path().join("test.redb");

    let store = RedbStorageEngine::open(&db_path)?;
    let tx1 = TxId(1);

    // Initial get returns None
    assert!(store.get(b"key1").await?.is_none());

    // Uncommitted put
    store.put(tx1, b"key1", b"value1").await?;

    // Transaction isolation: uncommitted writes are not visible
    assert!(store.get(b"key1").await?.is_none());

    // Commit transaction
    store.commit(tx1).await?;

    // Value is now visible
    let val = store.get(b"key1").await?;
    assert_eq!(val, Some(Bytes::from_static(b"value1")));

    Ok(())
}

#[tokio::test]
async fn test_redb_store_rollback() -> TestResult {
    let dir = tempdir()?;
    let db_path = dir.path().join("test.redb");

    let store = RedbStorageEngine::open(&db_path)?;
    let tx = TxId(42);

    store.put(tx, b"uncommitted_key", b"val").await?;
    assert!(store.get(b"uncommitted_key").await?.is_none());

    // Rollback transaction
    store.rollback(tx).await?;
    assert!(store.get(b"uncommitted_key").await?.is_none());

    Ok(())
}

#[tokio::test]
async fn test_redb_store_mvcc_get_at_seq() -> TestResult {
    let dir = tempdir()?;
    let db_path = dir.path().join("test.redb");

    let store = RedbStorageEngine::open(&db_path)?;
    let tx1 = TxId(1);
    let tx2 = TxId(2);

    // Version 1 at seq 1
    store.put(tx1, b"key_mvcc", b"v1").await?;
    store.commit(tx1).await?;

    // Version 2 at seq 2
    store.put(tx2, b"key_mvcc", b"v2").await?;
    store.commit(tx2).await?;

    // Check reads at sequence numbers
    let val_at_1 = store.get_at_seq(b"key_mvcc", 1).await?;
    assert_eq!(val_at_1, Some(Bytes::from_static(b"v1")));

    let val_at_2 = store.get_at_seq(b"key_mvcc", 2).await?;
    assert_eq!(val_at_2, Some(Bytes::from_static(b"v2")));

    Ok(())
}

#[tokio::test]
async fn test_redb_store_delete_and_tombstone() -> TestResult {
    let dir = tempdir()?;
    let db_path = dir.path().join("test.redb");

    let store = RedbStorageEngine::open(&db_path)?;
    let tx1 = TxId(1);
    let tx2 = TxId(2);

    store.put(tx1, b"key_del", b"hello").await?;
    store.commit(tx1).await?;
    assert_eq!(
        store.get(b"key_del").await?,
        Some(Bytes::from_static(b"hello"))
    );

    store.delete(tx2, b"key_del").await?;
    store.commit(tx2).await?;

    // After delete commit, get returns None
    assert!(store.get(b"key_del").await?.is_none());

    // Historical read at seq 1 still sees "hello"
    assert_eq!(
        store.get_at_seq(b"key_del", 1).await?,
        Some(Bytes::from_static(b"hello"))
    );

    Ok(())
}

#[tokio::test]
async fn test_redb_store_scan_prefix_and_bounded() -> TestResult {
    let dir = tempdir()?;
    let db_path = dir.path().join("test.redb");

    let store = RedbStorageEngine::open(&db_path)?;
    let tx = TxId(10);

    store.put(tx, b"user:1", b"alice").await?;
    store.put(tx, b"user:2", b"bob").await?;
    store.put(tx, b"item:1", b"sword").await?;
    store.commit(tx).await?;

    let user_scans = store.scan_prefix(b"user:").await?;
    assert_eq!(user_scans.len(), 2);
    assert_eq!(user_scans[0], (b"user:1".to_vec(), b"alice".to_vec()));
    assert_eq!(user_scans[1], (b"user:2".to_vec(), b"bob".to_vec()));

    Ok(())
}

#[tokio::test]
async fn test_redb_store_rollback_to_tx() -> TestResult {
    let dir = tempdir()?;
    let db_path = dir.path().join("test.redb");

    let store = RedbStorageEngine::open(&db_path)?;
    let tx1 = TxId(10);
    let tx2 = TxId(20);

    store.put(tx1, b"key_t1", b"v1").await?;
    store.commit(tx1).await?;

    store.put(tx2, b"key_t2", b"v2").await?;
    store.commit(tx2).await?;

    // Rollback to tx1
    store.rollback_to_tx(tx1).await?;

    assert_eq!(store.get(b"key_t1").await?, Some(Bytes::from_static(b"v1")));
    assert!(store.get(b"key_t2").await?.is_none());
    assert_eq!(store.last_tx_id().await?, tx1);

    Ok(())
}

#[tokio::test]
async fn test_redb_store_persistence_reopen() -> TestResult {
    let dir = tempdir()?;
    let db_path = dir.path().join("test.redb");

    {
        let store = RedbStorageEngine::open(&db_path)?;
        let tx = TxId(100);
        store.put(tx, b"persistent_key", b"data").await?;
        store.commit(tx).await?;
    }

    // Reopen database
    let store2 = RedbStorageEngine::open(&db_path)?;
    assert_eq!(
        store2.get(b"persistent_key").await?,
        Some(Bytes::from_static(b"data"))
    );
    assert_eq!(store2.last_tx_id().await?, TxId(100));

    Ok(())
}
