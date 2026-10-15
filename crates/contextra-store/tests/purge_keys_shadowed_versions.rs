mod purge_oracle_helper;

use contextra_core::{StorageEngine, TxId};
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeScope};
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use contextra_store::sstable::witness_types::SstScopeSpec;
use purge_oracle_helper::verify_purge_oracle;
use tempfile::tempdir;

struct TestFreeSpaceProbe {
    free_bytes: Option<u64>,
}

impl FreeSpaceProbe for TestFreeSpaceProbe {
    fn available_bytes(&self) -> std::result::Result<Option<u64>, contextra_ports::ProbeError> {
        Ok(self.free_bytes)
    }

    fn free_space_bytes(&self) -> Option<u64> {
        self.free_bytes
    }
}

/// Injection Matrix Row 5:
/// Older version in deeper SSTable level, shadowed by Tombstone in L0.
/// After Purge, nothing remains in storage (oracle check: versions_found == 0 && tombstones_remaining == 0).
#[tokio::test]
async fn test_purge_keys_shadowed_versions_injection_row5() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    // Step 1: Put key in deeper level (flush to SSTable 1)
    storage
        .put(TxId(10), b"shadowed_key", b"old_secret")
        .await?;
    storage.commit(TxId(10)).await?;
    StorageEngine::flush(&storage).await?;

    // Step 2: Delete key in L0 (flush tombstone to SSTable 2)
    storage.delete(TxId(20), b"shadowed_key").await?;
    storage.commit(TxId(20)).await?;
    StorageEngine::flush(&storage).await?;

    let probe = TestFreeSpaceProbe {
        free_bytes: Some(100 * 1024 * 1024),
    };
    let scope = PurgeScope::Keys(vec![b"shadowed_key".to_vec()]);

    // Purge with as_of_tx = 100
    let report = storage.purge_keys(scope, 100, Some(&probe)).await.unwrap();

    // Simulate Orchestrator disposing obsolete files
    for file in &report.obsolete_files {
        let _ = contextra_durable_fs::durable_remove(file);
    }

    // Verify via Oracle that BOTH the older version and the shadowing tombstone are gone
    let scope_spec = SstScopeSpec {
        explicit: vec![b"shadowed_key".to_vec()],
        prefixes: vec![],
    };
    verify_purge_oracle(dir.path(), &scope_spec, 100);

    // Also verify normal API read returns None
    let res = storage.get_at_seq(b"shadowed_key", 100).await?;
    assert_eq!(res, None);

    Ok(())
}
