// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: Testet CompactionGate-Lease Verhalten (Purge meldet Pending/CompactionBusy bei belegtem Lock, und laeuft nach Freigabe).

use contextra_ports::erasure::PurgeError;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use tempfile::tempdir;

#[tokio::test]
async fn test_purge_returns_compaction_busy_when_lease_is_held() -> std::io::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await.expect("open storage");

    let scope = PurgeScope::Keys(vec![b"k_lease".to_vec()]);

    {
        // Acquire CompactionGate lease manually to simulate active compaction
        let lock = storage.compaction_lease_lock_for_test();
        let guard = lock.try_lock();
        assert!(guard.is_ok(), "lease lock acquire should succeed");

        let res = storage.purge_keys(scope.clone(), 100, None).await;
        assert!(
            matches!(res, Err(PurgeError::CompactionBusy)),
            "Purge must return CompactionBusy when lease is held, got {:?}",
            res
        );
    } // lease released here

    // After lease release, purge_keys should proceed without CompactionBusy
    let res_after = storage.purge_keys(scope, 100, None).await;
    assert!(
        res_after.is_ok(),
        "Purge should succeed after compaction lease release, got {:?}",
        res_after
    );

    Ok(())
}
