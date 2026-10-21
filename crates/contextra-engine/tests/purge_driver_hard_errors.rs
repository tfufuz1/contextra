// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Test driver_propagates_hard_errors for DpiPurgeDriver (§WP-1).

use contextra_engine::deletion::forget_options::FonLogicalHandle;
use contextra_engine::deletion::proof_issue::DpiDeletionRuntime;
use contextra_engine::deletion::purge_driver::{DpiPurgeDriver, DpiPurgeDriverConfig};
use contextra_ports::erasure::{FreeSpaceProbe, ProbeError};
use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use contextra_types::{ContextraError, TxId};
use std::fs;
use tempfile::tempdir;

struct ZeroFreeSpaceProbe;
impl FreeSpaceProbe for ZeroFreeSpaceProbe {
    fn available_bytes(&self) -> Result<Option<u64>, ProbeError> {
        Ok(Some(0))
    }

    fn free_space_bytes(&self) -> Option<u64> {
        Some(0)
    }
}

#[tokio::test]
async fn driver_propagates_hard_errors() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let runtime = DpiDeletionRuntime::open(dir.path(), None, [1u8; 16])?;

    storage.put(TxId(1), b"corrupt_key", b"corrupt_val").await?;
    storage.commit(TxId(1)).await?;
    StorageEngine::flush(&storage).await?;

    // Corrupt SSTable file on disk
    for entry in fs::read_dir(dir.path())? {
        let entry = entry?;
        if entry.path().extension().map_or(false, |ext| ext == "sst") {
            fs::write(entry.path(), b"CORRUPTED_BYTES")?;
        }
    }

    let driver = DpiPurgeDriver::new(&storage, &runtime, DpiPurgeDriverConfig::default());
    let scope = PurgeScope::Keys(vec![b"corrupt_key".to_vec()]);
    let logical_handle = FonLogicalHandle {
        proof_id: [1u8; 16],
        proof_path: dir.path().join("proof.json"),
        full_proof_id: [1u8; 32],
    };

    let res = driver
        .run_physical(
            scope,
            &["101/1".to_string()],
            TxId(10),
            &logical_handle,
            &Default::default(),
            None,
            &[],
            None,
        )
        .await;

    // Hard errors must return Err(ContextraError::Storage), NEVER Ok(PurgeHandle::Blocked)
    assert!(res.is_err(), "Hard I/O error must produce Err");
    match res.unwrap_err() {
        ContextraError::Storage(msg) => {
            assert!(msg.contains("Physical purge failed") || msg.contains("Read error"));
        }
        other => panic!("Expected ContextraError::Storage, got {:?}", other),
    }

    Ok(())
}

#[tokio::test]
async fn driver_maps_blocked_properly() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let runtime = DpiDeletionRuntime::open(dir.path(), None, [1u8; 16])?;

    storage.put(TxId(1), b"blocked_key", b"val").await?;
    storage.commit(TxId(1)).await?;
    StorageEngine::flush(&storage).await?;

    let driver = DpiPurgeDriver::new(&storage, &runtime, DpiPurgeDriverConfig::default());
    let scope = PurgeScope::Keys(vec![b"blocked_key".to_vec()]);
    let logical_handle = FonLogicalHandle {
        proof_id: [1u8; 16],
        proof_path: dir.path().join("proof.json"),
        full_proof_id: [1u8; 32],
    };

    let probe = ZeroFreeSpaceProbe;
    let res = driver
        .run_physical(
            scope,
            &["101/1".to_string()],
            TxId(10),
            &logical_handle,
            &Default::default(),
            Some(&probe),
            &[],
            None,
        )
        .await?;

    match res {
        contextra_engine::deletion::forget_options::PurgeHandle::Blocked { reason, retry_hint } => {
            assert!(reason.contains("Insufficient disk space"));
            assert!(!retry_hint.is_empty());
        }
        other => panic!("Expected PurgeHandle::Blocked, got {:?}", other),
    }

    Ok(())
}
