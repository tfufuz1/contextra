//! Integration tests for MANIFEST crash safety and SSTable non-resurrection.
//!
//! Verifies Opus Stufe 0 (Point 0.6) invariants:
//! 1. Scenario A: Crash after MANIFEST Replace append but before old SSTable file deletion.
//!    Restoring old SSTables before reopen must NOT resurrect deleted/overwritten keys.
//!    Dead SSTables proven by MANIFEST must be removed during startup recovery.
//! 2. Scenario B: Crash BEFORE Replace append to MANIFEST (output SSTable exists on disk unmanifested).
//!    Old SSTables remain valid and readable, unmanifested output file is skipped and kept.
//! 3. Scenario C: Tail-truncated Replace record at MANIFEST end.
//!    Incomplete Replace record is safely discarded (no Err/panic), input SSTables remain active.

use contextra_core::{Result, StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;

fn create_test_config(path: &Path) -> LsmConfig {
    LsmConfig {
        path: path.to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        encryption_passphrase: None,
        compaction: contextra_store::compaction::CompactionConfig {
            min_sstables_per_tier: 2,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Scenario A: Crash between MANIFEST Replace append and old SSTable deletion.
/// Old SSTables are restored to disk before reopen.
/// Invariant: `get("k1")` must be `None`, dead SSTables must be deleted from disk by startup recovery.
#[tokio::test]
async fn test_manifest_crash_scenario_a_no_resurrection_and_dead_sst_cleanup() -> Result<()> {
    let tmp = TempDir::new().expect("temp dir");
    let config = create_test_config(tmp.path());

    let (sst1_path, sst2_path, sst1_bytes, sst2_bytes) = {
        let storage = LsmStorage::new(config.clone()).await?;

        // 1. Insert k1 and flush to SST-1
        let tx1 = TxId::new(1);
        storage.put(tx1, b"k1", b"v1").await?;
        storage.commit(tx1).await?;
        storage.force_flush().await?;

        // 2. Delete k1 (tombstone) and flush to SST-2
        let tx2 = TxId::new(2);
        storage.delete(tx2, b"k1").await?;
        storage.commit(tx2).await?;
        storage.force_flush().await?;

        let mut ssts: Vec<PathBuf> = std::fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "sst"))
            .collect();
        ssts.sort();
        assert_eq!(
            ssts.len(),
            2,
            "Expected exactly 2 SSTable files before compaction"
        );

        let p1 = ssts[0].clone();
        let p2 = ssts[1].clone();
        let b1 = std::fs::read(&p1).expect("read sst1");
        let b2 = std::fs::read(&p2).expect("read sst2");

        // 3. Force compaction -> creates SST-3, appends Replace to MANIFEST, deletes SST-1 and SST-2
        let compacted = storage.maybe_compact().await?;
        assert!(compacted, "Compaction should have run and merged SSTables");

        assert!(!p1.exists(), "SST-1 should be deleted after compaction");
        assert!(!p2.exists(), "SST-2 should be deleted after compaction");

        // 4. Simulate crash recovery scenario: restore SST-1 and SST-2 bytes back to disk
        std::fs::write(&p1, &b1).expect("restore sst1");
        std::fs::write(&p2, &b2).expect("restore sst2");

        assert!(p1.exists(), "SST-1 restored");
        assert!(p2.exists(), "SST-2 restored");

        storage.wait_shutdown().await;
        (p1, p2, b1, b2)
    };

    let _ = (sst1_bytes, sst2_bytes);

    // 5. Reopen storage
    {
        let storage = LsmStorage::new(config).await?;

        // 6. Verify k1 is NOT resurrected (get(k1) == None)
        let val = storage.get(b"k1").await?;
        assert_eq!(
            val, None,
            "Key 'k1' must NOT be resurrected after reopening with dead SSTables restored"
        );

        // 7. Verify SST-1 and SST-2 were removed from disk by startup recovery
        assert!(
            !sst1_path.exists(),
            "Dead SSTable 1 must be cleaned up by startup recovery"
        );
        assert!(
            !sst2_path.exists(),
            "Dead SSTable 2 must be cleaned up by startup recovery"
        );

        storage.wait_shutdown().await;
    }

    Ok(())
}

/// Scenario B: Crash BEFORE Replace append (unmanifested SSTable on disk).
/// Invariant: Input SSTables remain active and readable, unmanifested SSTable is skipped and NOT deleted.
#[tokio::test]
async fn test_manifest_crash_scenario_b_unmanifested_sstable_skipped_and_preserved() -> Result<()> {
    let tmp = TempDir::new().expect("temp dir");
    let config = create_test_config(tmp.path());

    let fake_sst_path = tmp.path().join("sst-compact-99999999999999999999-0000.sst");

    // Phase 1: Write two SSTables
    {
        let storage = LsmStorage::new(config.clone()).await?;

        let tx1 = TxId::new(1);
        storage.put(tx1, b"key-a", b"val-a").await?;
        storage.commit(tx1).await?;
        storage.force_flush().await?;

        let tx2 = TxId::new(2);
        storage.put(tx2, b"key-b", b"val-b").await?;
        storage.commit(tx2).await?;
        storage.force_flush().await?;

        // Write an unmanifested dummy SST file (simulating output file written before MANIFEST Replace)
        std::fs::write(&fake_sst_path, b"dummy unmanifested sstable content")
            .expect("write unmanifested file");

        storage.wait_shutdown().await;
    }

    // Phase 2: Reopen storage
    {
        let storage = LsmStorage::new(config).await?;

        // Verify keys from valid manifest entries are readable
        assert_eq!(
            storage.get(b"key-a").await?,
            Some(bytes::Bytes::from_static(b"val-a"))
        );
        assert_eq!(
            storage.get(b"key-b").await?,
            Some(bytes::Bytes::from_static(b"val-b"))
        );

        // Verify unmanifested SSTable was NOT deleted
        assert!(
            fake_sst_path.exists(),
            "Unmanifested SSTable file must be skipped and preserved on disk"
        );

        storage.wait_shutdown().await;
    }

    Ok(())
}

/// Scenario C: Tail-truncated Replace record at MANIFEST end.
/// Invariant: Incomplete Replace record is safely ignored (no Err/panic), input SSTables remain active.
#[tokio::test]
async fn test_manifest_crash_scenario_c_truncated_replace_record_ignored() -> Result<()> {
    use contextra_store::manifest::{Manifest, ManifestEntry};

    let tmp = TempDir::new().expect("temp dir");
    let config = create_test_config(tmp.path());

    // Phase 1: Write data and create a truncated Replace entry in MANIFEST
    {
        let storage = LsmStorage::new(config.clone()).await?;

        let tx1 = TxId::new(1);
        storage.put(tx1, b"k-x", b"v-x").await?;
        storage.commit(tx1).await?;
        storage.force_flush().await?;

        let tx2 = TxId::new(2);
        storage.put(tx2, b"k-y", b"v-y").await?;
        storage.commit(tx2).await?;
        storage.force_flush().await?;

        let ssts: Vec<PathBuf> = std::fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "sst"))
            .collect();
        assert_eq!(ssts.len(), 2);

        // Append a valid Replace entry to MANIFEST and then truncate it mid-payload
        let manifest_path = tmp.path().join("MANIFEST");
        let replace_entry = ManifestEntry::Replace {
            removed: ssts.clone(),
            added: tmp.path().join("sst-compact-001.sst"),
            added_max_tx: 2,
            rank: 0,
        };

        {
            let manifest = Manifest::open(&manifest_path).await?;
            manifest.append(&replace_entry).await?;
        }

        // Truncate MANIFEST file to chop off trailing bytes of Replace entry
        let mut manifest_bytes = std::fs::read(&manifest_path).expect("read manifest");
        assert!(manifest_bytes.len() > 15, "MANIFEST should contain data");
        manifest_bytes.truncate(manifest_bytes.len() - 10);
        std::fs::write(&manifest_path, manifest_bytes).expect("write truncated manifest");

        storage.wait_shutdown().await;
    }

    // Phase 2: Reopen storage
    {
        let storage = LsmStorage::new(config).await?;

        // Verify startup succeeds and original data is readable
        assert_eq!(
            storage.get(b"k-x").await?,
            Some(bytes::Bytes::from_static(b"v-x"))
        );
        assert_eq!(
            storage.get(b"k-y").await?,
            Some(bytes::Bytes::from_static(b"v-y"))
        );

        storage.wait_shutdown().await;
    }

    Ok(())
}
