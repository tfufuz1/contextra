//! Integration tests for ErasureLedger wiring during WAL replay.

use contextra_core::{ContextraError, StorageEngine, TxId};
use contextra_store::erasure_ledger::{
    compute_keyed_hash, LedgerEntry, LedgerEntryKind, LedgerKey,
};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use tempfile::tempdir;

#[tokio::test]
async fn test_ledger_replay_drops_erased_record_even_if_tombstone_missing(
) -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    tokio::fs::write(dir.path().join("SALT"), &[0u8; 32])
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    let key = b"erased_key_1";
    let val = b"val1";

    let salt_bytes = [0u8; 32];
    let ledger_key = LedgerKey::derive_from_master_secret(&salt_bytes);
    let keyed_hash = compute_keyed_hash(&ledger_key, b"lsm", key);

    // 1. Initialize store, insert key at tx 1 (written to WAL)
    {
        let store = LsmStorage::new(config.clone()).await?;
        store.put(TxId::new(1), key, val).await?;
        store.commit(TxId::new(1)).await?;

        // 2. Append erasure entry for key at erased_at_tx = 10
        let entry = LedgerEntry {
            ledger_seq: 1,
            kind: LedgerEntryKind::Key,
            keyed_hash,
            erased_at_tx: 10,
            proof_id: [1u8; 16],
        };
        store.ledger_append(&[entry]).await?;

        // 3. Drop store without flushing memtable to SSTable so Put remains in WAL
        drop(store);
    }

    // 4. Reopen store and verify key at tx 1 is filtered out by ledger during WAL replay
    let store = LsmStorage::new(config).await?;
    let read_val = store.get(key).await?;
    assert_eq!(
        read_val, None,
        "Record at tx <= erased_at_tx must be filtered out by ErasureLedger during WAL replay"
    );

    Ok(())
}

#[tokio::test]
async fn test_ledger_replay_allows_rewrite_newer_than_erased_tx() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    tokio::fs::write(dir.path().join("SALT"), &[0u8; 32])
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    let key = b"rewrite_key_1";
    let val_old = b"old_val";
    let val_new = b"new_val";

    let salt_bytes = [0u8; 32];
    let ledger_key = LedgerKey::derive_from_master_secret(&salt_bytes);
    let keyed_hash = compute_keyed_hash(&ledger_key, b"lsm", key);

    {
        let store = LsmStorage::new(config.clone()).await?;
        // Put old version at tx 5
        store.put(TxId::new(5), key, val_old).await?;
        store.commit(TxId::new(5)).await?;

        // Erase up to tx 10
        let entry = LedgerEntry {
            ledger_seq: 1,
            kind: LedgerEntryKind::Key,
            keyed_hash,
            erased_at_tx: 10,
            proof_id: [2u8; 16],
        };
        store.ledger_append(&[entry]).await?;

        // Put new version at tx 15 (> erased_at_tx)
        store.put(TxId::new(15), key, val_new).await?;
        store.commit(TxId::new(15)).await?;

        drop(store);
    }

    // Reopen store
    let store = LsmStorage::new(config).await?;
    let read_val = store.get(key).await?;
    assert_eq!(
        read_val,
        Some(bytes::Bytes::from("new_val")),
        "Rewritten record at tx > erased_at_tx must survive and be readable"
    );

    Ok(())
}

#[tokio::test]
async fn test_ledger_prefix_erasure_filters_all_keys_under_prefix() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    tokio::fs::write(dir.path().join("SALT"), &[0u8; 32])
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    let prefix = b"tenant_col_A:";
    let key1 = b"tenant_col_A:item1";
    let key2 = b"tenant_col_A:item2";
    let key_other = b"tenant_col_B:item1";

    let salt_bytes = [0u8; 32];
    let ledger_key = LedgerKey::derive_from_master_secret(&salt_bytes);
    let prefix_hash = compute_keyed_hash(&ledger_key, b"lsm", prefix);

    {
        let store = LsmStorage::new(config.clone()).await?;
        store.put(TxId::new(1), key1, b"val1").await?;
        store.commit(TxId::new(1)).await?;

        store.put(TxId::new(2), key2, b"val2").await?;
        store.commit(TxId::new(2)).await?;

        store.put(TxId::new(3), key_other, b"val_other").await?;
        store.commit(TxId::new(3)).await?;

        // Erasure entry for prefix "tenant_col_A:" at tx 10
        let entry = LedgerEntry {
            ledger_seq: 1,
            kind: LedgerEntryKind::Prefix {
                prefix_len: prefix.len() as u32,
            },
            keyed_hash: prefix_hash,
            erased_at_tx: 10,
            proof_id: [3u8; 16],
        };
        store.ledger_append(&[entry]).await?;
        drop(store);
    }

    let store = LsmStorage::new(config).await?;
    assert_eq!(store.get(key1).await?, None);
    assert_eq!(store.get(key2).await?, None);
    assert_eq!(
        store.get(key_other).await?,
        Some(bytes::Bytes::from("val_other")),
        "Keys outside erasure prefix must be preserved"
    );

    Ok(())
}
