//! G15 Decision Test: WAL Tail with Delete-Frame Truncation
//!
//! Evaluates whether deleting a record, flushing to SSTable (so a Tombstone exists in SSTables),
//! then truncating the Delete entry from WAL tail, and restarting LsmStorage causes record resurrection or not.
//!
//! Result: If the deleted record stays deleted (`get` returns `None`), G15 is refuted for the WAL path
//! because SSTable tombstone takes precedence over missing WAL log entries.

use contextra_core::{ContextraError, StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use tempfile::tempdir;

#[tokio::test]
async fn test_g15_wal_tail_delete_truncation_with_sstable_tombstone() -> Result<(), ContextraError>
{
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    tokio::fs::write(dir.path().join("SALT"), &[0u8; 32])
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    let key = b"g15_test_key";
    let val = b"g15_test_value";

    // 1. Initialize store and insert key
    {
        let store = LsmStorage::new(config.clone()).await?;
        store.put(TxId::new(1), key, val).await?;
        // Flush Put to SSTable
        store.force_flush().await?;

        // 2. Delete key
        store.delete(TxId::new(2), key).await?;
        // Flush Delete (Tombstone) to SSTable
        store.force_flush().await?;

        store.close().await?;
    }

    // 3. Truncate active WAL file in the storage directory to remove the Delete frame from WAL
    let wal_path = dir.path().join("wal.log");
    if wal_path.exists() {
        let wal_bytes = tokio::fs::read(&wal_path)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        // Truncate active WAL to header size (4 bytes)
        let truncate_len = 4.min(wal_bytes.len());
        tokio::fs::write(&wal_path, &wal_bytes[..truncate_len])
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
    }

    // 4. Reopen LsmStorage
    let reopened_store = LsmStorage::new(config).await?;

    // 5. Read key
    let read_res = reopened_store.get(key).await?;

    // 6. Assert: Key MUST remain deleted (None) due to SSTable Tombstone persistence
    if read_res.is_some() {
        return Err(ContextraError::Storage(
            "G15 failed: record resurrected".into(),
        ));
    }

    Ok(())
}
