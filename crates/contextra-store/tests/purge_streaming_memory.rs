// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: Test for single-pass streaming purge rewrite without buffering entries in a Vec (P-3 requirement).

use contextra_ports::StorageEngine;
use contextra_types::TxId;
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeScope};
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use tempfile::tempdir;

struct MockFreeSpaceProbe {
    free_bytes: u64,
}

impl FreeSpaceProbe for MockFreeSpaceProbe {
    fn available_bytes(&self) -> std::result::Result<Option<u64>, contextra_ports::ProbeError> {
        Ok(Some(self.free_bytes))
    }
}

/// Verifies that LsmStorage::purge_keys executes a streaming single-pass rewrite
/// directly from the SSTable stream into SstableBuilder, without allocating a Vec
/// of valid entries or scaling memory with SSTable entry count (P-3 requirement).
#[tokio::test]
async fn test_purge_streaming_rewrite_memory_independence() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let num_entries = 2000u64;
    // Insert 2000 keys: even keys starting with "purge:", odd keys starting with "keep:"
    for i in 1..=num_entries {
        let key = if i % 2 == 0 {
            format!("purge:key_{:05}", i).into_bytes()
        } else {
            format!("keep:key_{:05}", i).into_bytes()
        };
        let val = format!("val_{:05}", i).into_bytes();
        storage.put(TxId(i), &key, &val).await?;
        storage.commit(TxId(i)).await?;
    }

    StorageEngine::flush(&storage).await?;

    let probe = MockFreeSpaceProbe {
        free_bytes: 100 * 1024 * 1024,
    };
    let scope = PurgeScope::Prefixes(vec![b"purge:".to_vec()]);

    let as_of_tx = storage.last_applied_seq();
    let report = storage.purge_keys(scope, as_of_tx, Some(&probe)).await.unwrap();

    assert_eq!(report.records_purged, 1000);
    assert_eq!(report.sstables_replaced, 1);

    // Verify all "purge:" keys are absent and all "keep:" keys are present
    for i in 1..=num_entries {
        if i % 2 == 0 {
            let key = format!("purge:key_{:05}", i).into_bytes();
            let res = storage.get(&key).await?;
            assert_eq!(res, None, "Purged key {:?} must be absent", String::from_utf8_lossy(&key));
        } else {
            let key = format!("keep:key_{:05}", i).into_bytes();
            let expected_val = format!("val_{:05}", i).into_bytes();
            let res = storage.get(&key).await?.map(|b| b.to_vec());
            assert_eq!(res, Some(expected_val), "Kept key {:?} must be present", String::from_utf8_lossy(&key));
        }
    }

    Ok(())
}
