//! Integration test for crash scenario between ledger_append and Tombstone commit.

use contextra_core::{ContextraError, StorageEngine, TxId};
use contextra_store::erasure_ledger::{
    compute_keyed_hash, LedgerEntry, LedgerEntryKind, LedgerKey,
};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use tempfile::tempdir;

#[tokio::test]
async fn test_crash_between_ledger_append_and_tombstone_commit_keeps_data_invisible(
) -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    tokio::fs::write(dir.path().join("SALT"), &[0u8; 32])
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    let key = b"crash_test_key";
    let val = b"crash_test_val";

    let salt_bytes = [0u8; 32];
    let ledger_key = LedgerKey::derive_from_master_secret(&salt_bytes);
    let keyed_hash = compute_keyed_hash(&ledger_key, b"lsm", key);

    // 1. Insert key and commit
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
            proof_id: [100u8; 16],
        };
        store.ledger_append(&[entry]).await?;

        // 3. Simulate process crash BEFORE tombstone delete/commit is executed
        // (Do NOT call store.delete() or store.close(), just drop storage instance)
        drop(store);
    }

    // 4. Reopen LsmStorage
    let reopened_store = LsmStorage::new(config).await?;

    // 5. Query key - must return None because ErasureLedger filters out key at tx 1 <= erased_at_tx 10
    let read_val = reopened_store.get(key).await?;
    assert_eq!(
        read_val, None,
        "Data must be invisible after restart even if crash occurred before Tombstone commit"
    );

    Ok(())
}
