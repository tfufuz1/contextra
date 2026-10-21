// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Fehlerinjektionstest für Lesefehler mitten in einer SSTable während des Purge.

use contextra_ports::StorageEngine;
use contextra_ports::erasure::PurgeError;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use contextra_types::TxId;
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_purge_obsolete_fault_injection_read_error() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let key1 = b"t:1:k1".to_vec();
    let val1 = b"v1_data_fault_test".to_vec();

    storage.put(TxId(1), &key1, &val1).await?;
    storage.commit(TxId(1)).await?;
    StorageEngine::flush(&storage).await?;

    // Find the created sst file
    let mut sst_path = None;
    for entry in fs::read_dir(dir.path())?.flatten() {
        if entry.path().extension().map_or(false, |ext| ext == "sst") {
            sst_path = Some(entry.path());
            break;
        }
    }
    let sst_file = sst_path.expect("SSTable file must exist");
    let original_bytes = fs::read(&sst_file)?;

    // Corrupt the SSTable header to simulate a read error mid-scan
    fs::write(&sst_file, b"CORRUPTED_HEADER_INVALID_SSTABLE_BYTES_TRIGGERING_READ_ERROR")?;

    let scope = PurgeScope::Keys(vec![key1]);
    let result = storage.purge_keys(scope, 10, None).await;

    assert!(result.is_err(), "purge_keys must fail on read error");

    match result.unwrap_err() {
        PurgeError::ReadError { file, source_details, .. } => {
            assert!(
                file.contains(sst_file.file_name().unwrap().to_str().unwrap()),
                "ReadError must contain filename"
            );
            assert!(!source_details.is_empty(), "ReadError details must not be empty");
        }
        other => panic!("Expected PurgeError::ReadError, got {:?}", other),
    }

    // Restore original bytes to inspect filesystem state
    fs::write(&sst_file, &original_bytes)?;

    Ok(())
}
