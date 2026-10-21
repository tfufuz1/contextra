//! G15 Test: WAL Tail Delete Truncation & ErasureLedger Defense Line
//!
//! Evaluates whether truncating a Delete entry from WAL tail causes record resurrection when Put exists in SSTable,
//! and proves that ErasureLedger successfully prevents resurrection as a second line of defense.

use contextra_ports::StorageEngine;
use contextra_store::erasure_ledger::{
    compute_keyed_hash, ErasureLedger, LedgerEntry, LedgerEntryKind, LedgerKey,
};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::{ContextraError, TxId};
use tempfile::tempdir;

#[tokio::test]
async fn test_g15_wal_truncation_resurrection_prevented_by_ledger() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    tokio::fs::write(dir.path().join("SALT"), &[0u8; 32])
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    let key = b"g15_resurrection_key";
    let val = b"g15_resurrection_val";

    let salt_bytes = [0u8; 32];
    let ledger_key = LedgerKey::derive_from_master_secret(&salt_bytes);
    let keyed_hash = compute_keyed_hash(&ledger_key, b"lsm", key);

    // 1. Put key at Tx 1 and flush to SSTable (so SSTable contains Put)
    {
        let store = LsmStorage::new(config.clone()).await?;
        store.put(TxId::new(1), key, val).await?;
        store.commit(TxId::new(1)).await?;
        store.force_flush().await?;

        // 2. Append ErasureLedger entry at erased_at_tx = 2 directly
        let (ledger_opt, _) =
            ErasureLedger::open_or_fail_closed(dir.path(), ledger_key.clone(), true, None).await?;
        let mut ledger = ledger_opt.expect("ledger open");
        let entry = LedgerEntry {
            ledger_seq: 1,
            kind: LedgerEntryKind::Key,
            keyed_hash,
            erased_at_tx: 2,
            proof_id: [200u8; 16],
        };
        ledger.append(&[entry]).await?;

        // 3. Perform Delete at Tx 2 (goes to WAL, NOT flushed to SSTable)
        store.delete(TxId::new(2), key).await?;
        store.commit(TxId::new(2)).await?;

        store.close().await?;
    }

    // 4. Truncate active WAL file to simulate crash/truncation that removes Delete frame from WAL tail
    let wal_path = dir.path().join("wal.log");
    if wal_path.exists() {
        let wal_bytes = tokio::fs::read(&wal_path)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        let truncate_len = 4.min(wal_bytes.len());
        tokio::fs::write(&wal_path, &wal_bytes[..truncate_len])
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
    }

    // 5. Reopen LsmStorage
    let reopened_store = LsmStorage::new(config).await?;

    // 6. Read key
    let read_res = reopened_store.get(key).await?;

    // 7. Assert: ErasureLedger MUST prevent resurrection even when Delete frame was lost from WAL tail
    assert_eq!(
        read_res, None,
        "ErasureLedger second line of defense MUST prevent resurrection when WAL Delete frame is truncated"
    );

    Ok(())
}
