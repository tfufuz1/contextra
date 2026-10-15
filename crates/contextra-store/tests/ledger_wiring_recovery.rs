//! Integration tests for ErasureLedger status checks during recovery.

use contextra_types::{ContextraError, TxId};
use contextra_ports::StorageEngine;
use contextra_store::erasure_ledger::{
    LedgerEntry, LedgerEntryKind, LedgerOpenStatus, HEAD_FILENAME, LEDGER_FILENAME,
};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use tempfile::tempdir;

#[tokio::test]
async fn test_missing_ledger_on_non_pristine_db_opens_read_only() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    tokio::fs::write(dir.path().join("SALT"), &[0u8; 32])
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    // 1. Create DB with data
    {
        let store = LsmStorage::new(config.clone()).await?;
        store.put(TxId::new(1), b"k1", b"v1").await?;
        store.close().await?;
    }

    // 2. Delete erasure.ledger file
    let ledger_path = dir.path().join(LEDGER_FILENAME);
    if ledger_path.exists() {
        tokio::fs::remove_file(&ledger_path)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
    }

    // 3. Reopen DB - must open in ReadOnly mode with RollbackSuspected status
    let store = LsmStorage::new(config).await?;
    assert!(store.is_read_only());
    assert!(matches!(
        store.ledger_state(),
        LedgerOpenStatus::RollbackSuspected { .. }
    ));

    // 4. Writes must be rejected
    let append_res = store.ledger_append(&[]).await;
    assert!(
        append_res.is_err(),
        "ledger_append must fail in ReadOnly mode"
    );

    let flush_res = store.flush().await;
    assert!(flush_res.is_err(), "flush must fail in ReadOnly mode");

    let rollback_res = store.rollback_to_tx(TxId::new(1)).await;
    assert!(
        rollback_res.is_err(),
        "rollback_to_tx must fail in ReadOnly mode"
    );

    Ok(())
}

#[tokio::test]
async fn test_head_marker_mac_mismatch_opens_read_only() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    tokio::fs::write(dir.path().join("SALT"), &[0u8; 32])
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    // 1. Create DB and ledger
    {
        let store = LsmStorage::new(config.clone()).await?;
        let entry = LedgerEntry {
            ledger_seq: 1,
            kind: LedgerEntryKind::Key,
            keyed_hash: [1u8; 32],
            erased_at_tx: 5,
            proof_id: [1u8; 16],
        };
        store.ledger_append(&[entry]).await?;
        store.close().await?;
    }

    // 2. Tamper with ledger.head file (flip last MAC byte)
    let head_path = dir.path().join(HEAD_FILENAME);
    let mut head_bytes = tokio::fs::read(&head_path)
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;
    let last_idx = head_bytes.len() - 1;
    head_bytes[last_idx] ^= 0xFF;
    tokio::fs::write(&head_path, &head_bytes)
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    // 3. Reopen DB - must detect head marker mismatch
    let store = LsmStorage::new(config).await?;
    assert!(store.is_read_only());
    assert!(matches!(
        store.ledger_state(),
        LedgerOpenStatus::RollbackSuspected { .. }
    ));

    Ok(())
}

#[tokio::test]
async fn test_corrupt_ledger_frame_opens_read_only() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    tokio::fs::write(dir.path().join("SALT"), &[0u8; 32])
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    {
        let store = LsmStorage::new(config.clone()).await?;
        let entry = LedgerEntry {
            ledger_seq: 1,
            kind: LedgerEntryKind::Key,
            keyed_hash: [2u8; 32],
            erased_at_tx: 5,
            proof_id: [2u8; 16],
        };
        store.ledger_append(&[entry]).await?;
        store.close().await?;
    }

    // Corrupt erasure.ledger payload bytes
    let ledger_path = dir.path().join(LEDGER_FILENAME);
    let mut bytes = tokio::fs::read(&ledger_path)
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;
    if bytes.len() > 10 {
        bytes[10] ^= 0xFF; // Flip byte in payload
    }
    tokio::fs::write(&ledger_path, &bytes)
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    // Reopen DB - must detect corruption
    let store = LsmStorage::new(config).await?;
    assert!(store.is_read_only());
    assert!(matches!(
        store.ledger_state(),
        LedgerOpenStatus::Corrupt { .. }
    ));

    Ok(())
}
