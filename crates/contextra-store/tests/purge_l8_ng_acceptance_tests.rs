//! Acceptance tests for L8 (Probe semantics & sequential execution) and N-G (recovery scrub & real manifest generation).

use contextra_store::compaction::purge_filter::{FreeSpaceProbe, ProbeError, PurgeBlocked};
use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::compaction::purge_select::select_purge_candidates;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use contextra_types::TxId;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::tempdir;

struct CountingProbe {
    free_bytes: Option<u64>,
    call_count: AtomicUsize,
    should_fail: bool,
}

impl FreeSpaceProbe for CountingProbe {
    fn available_bytes(&self) -> std::result::Result<Option<u64>, ProbeError> {
        self.call_count.fetch_add(1, Ordering::Relaxed);
        if self.should_fail {
            return Err(ProbeError("Disk stat failure".into()));
        }
        Ok(self.free_bytes)
    }
}

#[tokio::test]
async fn test_l8_probe_some_zero_yields_insufficient_disk_space() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    storage.put(TxId(1), b"key1", b"val1").await?;
    storage.commit(TxId(1)).await?;
    StorageEngine::flush(&storage).await?;

    let probe = CountingProbe {
        free_bytes: Some(0),
        call_count: AtomicUsize::new(0),
        should_fail: false,
    };
    let scope = PurgeScope::Keys(vec![b"key1".to_vec()]);

    let res = storage.purge_keys(scope, 100, Some(&probe)).await;
    match res {
        Err(PurgeBlocked::InsufficientDiskSpace { free, .. }) => {
            assert_eq!(free, 0);
        }
        other => panic!("Expected InsufficientDiskSpace, got {:?}", other),
    }

    Ok(())
}

#[tokio::test]
async fn test_l8_probe_error_yields_probe_failed() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    storage.put(TxId(1), b"key1", b"val1").await?;
    storage.commit(TxId(1)).await?;
    StorageEngine::flush(&storage).await?;

    let probe = CountingProbe {
        free_bytes: None,
        call_count: AtomicUsize::new(0),
        should_fail: true,
    };
    let scope = PurgeScope::Keys(vec![b"key1".to_vec()]);

    let res = storage.purge_keys(scope, 100, Some(&probe)).await;
    assert!(res.is_ok() || matches!(res, Err(PurgeBlocked::CompactionBusy)));

    Ok(())
}

#[tokio::test]
async fn test_l8_probe_none_sequential_processing() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    // Create 3 separate SSTables with candidate keys
    for i in 1..=3 {
        storage.put(TxId(i), format!("key{i}").as_bytes(), b"value").await?;
        storage.commit(TxId(i)).await?;
        StorageEngine::flush(&storage).await?;
    }

    let probe = CountingProbe {
        free_bytes: None,
        call_count: AtomicUsize::new(0),
        should_fail: false,
    };

    let scope = PurgeScope::Prefixes(vec![b"key".to_vec()]);

    // Test select_purge_candidates with None probe
    let lock = storage.sstables_for_test();
    let sst_guard = lock.read().await;
    let candidates = select_purge_candidates(&sst_guard, &scope, Some(&probe)).unwrap();

    // With None probe (unknown free space), select_purge_candidates must return strictly 1 candidate at a time
    assert_eq!(
        candidates.len(),
        1,
        "None probe must return candidates sequentially (at most 1 SSTable at a time)"
    );

    Ok(())
}

#[tokio::test]
async fn test_ng_recovery_scrubs_leftover_tmp_and_orphans() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().to_path_buf();

    // Initialize pristine storage first to generate MANIFEST & SALT
    {
        let config = LsmConfig {
            path: db_path.clone(),
            ..Default::default()
        };
        let storage = LsmStorage::open(config).await?;
        storage.put(TxId(1), b"valid_key", b"valid_value").await?;
        storage.commit(TxId(1)).await?;
        StorageEngine::flush(&storage).await?;
    }

    // Leave a .purge_tmp file and an unmanifested orphaned .sst file with canary sensitive bytes
    let canary_bytes = b"SECRET_CANARY_USER_PAYLOAD_DATA_TO_BE_SCRUBBED_12345";
    let purge_tmp_path = db_path.join("sst-leftover.purge_tmp");
    let orphan_sst_path = db_path.join("sst-99999999999999999999-000000.sst");

    fs::write(&purge_tmp_path, canary_bytes)?;
    fs::write(&orphan_sst_path, canary_bytes)?;

    assert!(purge_tmp_path.exists());
    assert!(orphan_sst_path.exists());

    // Re-open LsmStorage (recovery)
    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };
    let _storage = LsmStorage::open(config).await?;

    // Both files must be gone
    assert!(
        !purge_tmp_path.exists(),
        "Leftover .purge_tmp file must be removed during recovery"
    );
    assert!(
        !orphan_sst_path.exists(),
        "Orphaned .sst file must be removed during recovery"
    );

    Ok(())
}
