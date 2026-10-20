// FILE-CONTEXT
// ZWECK: Red reproduction test proving that WAL entries remain in active/sealed WAL segments post-purge under default wiring on HEAD.

use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::purge::wiring::PurgeWiring;
use contextra_store::lsm::LsmStorage;
use contextra_testkit::deletion_oracle::rawsweep::raw_sweep;
use contextra_types::TxId;
use tempfile::tempdir;

#[tokio::test]
async fn test_red_proof_wal_marker_remains_post_purge_on_default_wiring(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::open(config).await?;
    let tx_id = TxId::try_new(100)?;

    let marker_key = b"key_wal_repro_marker".to_vec();
    let marker_val = b"secret_wal_payload_bytes_12345".to_vec();

    // 1. Write marker to storage (writes to WAL and Memtable)
    storage.put(tx_id, &marker_key, &marker_val).await?;
    storage.commit(tx_id).await?;

    // 2. Flush memtable to SSTable so SSTables also contain marker
    storage.flush().await?;

    // 3. Confirm marker is found by raw_sweep in directory prior to purge
    let pre_sweep = raw_sweep(dir.path(), &[marker_val.clone()])?;
    let pre_hits = pre_sweep.hits.len();
    assert!(
        pre_hits > 0,
        "Marker must be found in store directory before purge"
    );

    // 4. Execute purge on active storage
    let scope = PurgeScope::Keys(vec![marker_key.clone()]);

    let report = storage.purge_keys(scope.clone(), 200, None).await?;

    // SSTable compaction purges records from SSTables
    assert_eq!(report.records_purged, 1, "SSTable records purged");

    // Close storage to release directory lock before orchestrator standalone runs
    storage.close().await?;

    // 5. Execute purge via default wiring path (standalone orchestrator)
    let wiring = PurgeWiring::build_default(dir.path())?;
    let orchestrator = wiring.orchestrator_standalone();

    let _res = orchestrator
        .execute_purge_internal(scope, 200, None, wiring.hooks(), false)
        .await?;

    // 6. Run raw_sweep across entire store directory
    let post_sweep = raw_sweep(dir.path(), &[marker_val.clone()])?;
    let post_hits = post_sweep.hits.len();

    println!("Matches after purge on default wiring: {}", post_hits);
    assert_eq!(
        post_hits, 0,
        "Marker bytes must be completely purged from WAL"
    );

    Ok(())
}
