// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Raw-Sweep-Test zur Bestätigung vollständiger Befreiung von Purge-Marker-Bytes.

use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::purge::dispose::dispose_obsolete_files;
use contextra_store::lsm::LsmStorage;
use contextra_testkit::erasure_harness::raw_sweep;
use contextra_types::TxId;
use tempfile::tempdir;

#[tokio::test]
async fn test_purge_obsolete_raw_sweep_clean() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let marker_key = b"t:1:doc:purge_target_123".to_vec();
    let marker_val = b"MARKER_BYTES_SECRET_PURGE_998877".to_vec();
    let keep_key = b"t:1:doc:keep_target_456".to_vec();
    let keep_val = b"KEEP_BYTES_SECRET_STAYS_112233".to_vec();

    // Fill store
    storage.put(TxId(1), &marker_key, &marker_val).await?;
    storage.put(TxId(2), &keep_key, &keep_val).await?;
    storage.commit(TxId(1)).await?;
    storage.commit(TxId(2)).await?;

    // Force flush to SSTable
    StorageEngine::flush(&storage).await?;

    // Positive control: Marker must be present in SSTable on disk before purge
    let pre_sweep = raw_sweep(dir.path(), &[marker_val.clone()])?;
    assert!(
        !pre_sweep.is_clean(),
        "Positive control failed: Marker bytes were not found on disk before purge"
    );

    // Perform purge
    let scope = PurgeScope::Keys(vec![marker_key.clone()]);
    let report = storage.purge_keys(scope, 100, None).await?;

    assert_eq!(report.records_purged, 1);
    assert!(!report.obsolete_files.is_empty());

    // Dispose obsolete files (Phase 5)
    dispose_obsolete_files(dir.path(), report.obsolete_files, false).await?;

    // Post-purge raw sweep over ALL store files (SSTables, WAL, Manifest, Journal)
    let post_sweep = raw_sweep(dir.path(), &[marker_val.clone()])?;

    assert!(
        post_sweep.is_clean(),
        "Raw sweep failed: Found residual marker bytes on disk post-purge: {:?}",
        post_sweep.hits
    );

    Ok(())
}
