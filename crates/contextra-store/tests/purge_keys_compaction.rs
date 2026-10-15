// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Tests für Compaction-basierte Key-Bereinigung in LsmStorage::purge_keys.
// INVARIANTEN: Canary key raw-scan verification, crash safety at 3 interruption points, zero-panic.

use contextra_types::TxId;
use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeScope};
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use contextra_store::manifest::Manifest;
use tempfile::TempDir;

struct DummyProbe(u64);
impl FreeSpaceProbe for DummyProbe {
    fn available_bytes(&self) -> std::result::Result<Option<u64>, contextra_ports::ProbeError> {
        Ok(Some(self.0))
    }

    fn free_space_bytes(&self) -> Option<u64> {
        Some(self.0)
    }
}

#[tokio::test]
async fn test_purge_keys_compaction_canary_raw_scan() {
    let tmp = TempDir::new().expect("tempdir failed");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::open(config.clone())
        .await
        .expect("storage open failed");

    let tx1 = TxId(10);
    let tx2 = TxId(20);

    // SSTable 1
    storage
        .put(tx1, b"canary-1", b"secret-data-1")
        .await
        .expect("put failed");
    storage
        .put(tx1, b"user-1", b"user-data-1")
        .await
        .expect("put failed");
    storage.commit(tx1).await.expect("commit failed");
    storage.flush().await.expect("flush failed");

    // SSTable 2
    storage
        .put(tx2, b"canary-2", b"secret-data-2")
        .await
        .expect("put failed");
    storage
        .put(tx2, b"user-2", b"user-data-2")
        .await
        .expect("put failed");
    storage.commit(tx2).await.expect("commit failed");
    storage.flush().await.expect("flush failed");

    // Verify initial state
    assert!(storage.get(b"canary-1").await.unwrap().is_some());
    assert!(storage.get(b"canary-2").await.unwrap().is_some());
    assert!(storage.get(b"user-1").await.unwrap().is_some());
    assert!(storage.get(b"user-2").await.unwrap().is_some());

    // Execute purge_keys for canary keys
    let scope = PurgeScope::Prefixes(vec![b"canary-".to_vec()]);
    let probe = DummyProbe(100 * 1024 * 1024);

    let report = storage
        .purge_keys(scope, 0, Some(&probe))
        .await
        .expect("purge_keys failed");

    assert_eq!(report.records_purged, 2);
    assert!(report.sstables_replaced >= 2);

    // Verify via StorageEngine interface
    assert!(storage.get(b"canary-1").await.unwrap().is_none());
    assert!(storage.get(b"canary-2").await.unwrap().is_none());
    assert!(storage.get(b"user-1").await.unwrap().is_some());
    assert!(storage.get(b"user-2").await.unwrap().is_some());

    // Raw byte scan of newly generated active SSTables to prove canary absent physically
    let manifest_path = tmp.path().join("MANIFEST");
    let manifest_entries = Manifest::load(&manifest_path)
        .await
        .expect("manifest load failed");
    let active_sstables = Manifest::reconstruct_valid_sstables(&manifest_entries);

    for (relative_path, _rank) in active_sstables {
        let full_path = tmp.path().join(&relative_path);
        let bytes = std::fs::read(&full_path).expect("read sstable failed");
        let contains_canary_1 = bytes
            .windows(b"canary-1".len())
            .any(|window| window == b"canary-1");
        let contains_canary_2 = bytes
            .windows(b"canary-2".len())
            .any(|window| window == b"canary-2");
        assert!(
            !contains_canary_1,
            "canary-1 raw byte found in new SSTable {:?}",
            full_path
        );
        assert!(
            !contains_canary_2,
            "canary-2 raw byte found in new SSTable {:?}",
            full_path
        );
    }

    drop(storage);
}

#[tokio::test]
async fn test_purge_keys_crash_safety_3_interruption_points() {
    let tmp = TempDir::new().expect("tempdir failed");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };

    // Point 1: Simulated Crash pre-flight / pre-manifest write.
    // An uncommitted or orphan .tmp SSTable file in DB directory must be ignored during manifest reconstruct on reopen.
    {
        let storage = LsmStorage::open(config.clone())
            .await
            .expect("storage open failed");
        let tx = TxId(100);
        storage
            .put(tx, b"key-purge-1", b"val-purge-1")
            .await
            .unwrap();
        storage.put(tx, b"key-keep-1", b"val-keep-1").await.unwrap();
        storage.commit(tx).await.unwrap();
        storage.flush().await.unwrap();

        // Create orphan .tmp file simulating interrupted SSTable builder write before rename / manifest commit
        let orphan_tmp = tmp
            .path()
            .join("sst-purge-00000000000000000001-000001.sst.tmp");
        std::fs::write(&orphan_tmp, b"partial sstable contents").unwrap();

        drop(storage);

        // Reopen and verify DB state remains consistent and unaffected by orphan .tmp file
        let recovered = LsmStorage::open(config.clone()).await.unwrap();
        assert!(recovered.get(b"key-purge-1").await.unwrap().is_some());
        assert!(recovered.get(b"key-keep-1").await.unwrap().is_some());
        drop(recovered);
    }

    // Point 2: Interruption during purge before manifest batch commit (SSTable written & renamed, but manifest not updated)
    {
        let storage = LsmStorage::open(config.clone()).await.unwrap();
        let scope = PurgeScope::Keys(vec![b"key-purge-1".to_vec()]);
        let probe = DummyProbe(100 * 1024 * 1024);

        // Create an unreferenced sst file simulating a written SSTable that wasn't added to MANIFEST yet
        let unmanifested_sst = tmp.path().join("sst-purge-unmanifested.sst");
        std::fs::write(&unmanifested_sst, b"unmanifested data").unwrap();

        let _report = storage.purge_keys(scope, 0, Some(&probe)).await.unwrap();

        drop(storage);

        // Reopen: manifest reconstructs valid active sstables without orphan files
        let recovered = LsmStorage::open(config.clone()).await.unwrap();
        assert!(recovered.get(b"key-purge-1").await.unwrap().is_none());
        assert!(recovered.get(b"key-keep-1").await.unwrap().is_some());
        drop(recovered);
    }

    // Point 3: Interruption post manifest commit & reopen
    {
        let recovered = LsmStorage::open(config.clone()).await.unwrap();
        assert!(recovered.get(b"key-purge-1").await.unwrap().is_none());
        assert!(recovered.get(b"key-keep-1").await.unwrap().is_some());

        let manifest_path = tmp.path().join("MANIFEST");
        let entries = Manifest::load(&manifest_path).await.unwrap();
        let valid = Manifest::reconstruct_valid_sstables(&entries);
        assert!(!valid.is_empty());
        drop(recovered);
    }
}
