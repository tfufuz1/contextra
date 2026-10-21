//! Integration test for `select_purge_candidates` wiring in `CompactionEngine`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeScope};
use contextra_store::compaction::CompactionConfig;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::TxId;
use std::sync::Arc;
use tempfile::TempDir;

struct LargeFreeSpaceProbe;

impl FreeSpaceProbe for LargeFreeSpaceProbe {
    fn available_bytes(&self) -> std::result::Result<Option<u64>, contextra_ports::ProbeError> {
        Ok(Some(1_000_000_000))
    }

    fn free_space_bytes(&self) -> Option<u64> {
        Some(1_000_000_000)
    }
}

#[tokio::test]
async fn test_compaction_prioritizes_and_purges_marked_sstables() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        compaction: CompactionConfig {
            min_sstables_per_tier: 10, // Require 10 for normal STCS so standard STCS won't trigger automatically
            enable_adaptive_compaction: false,
            ..Default::default()
        },
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::open(config).await.expect("open storage"));

    // 1. Write SSTable 1: b_target_1
    let tx1 = TxId::new(10);
    storage
        .put(tx1, b"b_target_1", b"val_1")
        .await
        .expect("put tx1");
    storage.commit(tx1).await.expect("commit tx1");
    storage.force_flush().await.expect("flush 1");

    // 2. Write SSTable 2: b_target_2
    let tx2 = TxId::new(20);
    storage
        .put(tx2, b"b_target_2", b"val_2")
        .await
        .expect("put tx2");
    storage.commit(tx2).await.expect("commit tx2");
    storage.force_flush().await.expect("flush 2");

    // 3. Write SSTable 3: z_other
    let tx3 = TxId::new(30);
    storage
        .put(tx3, b"z_other", b"val_z")
        .await
        .expect("put tx3");
    storage.commit(tx3).await.expect("commit tx3");
    storage.force_flush().await.expect("flush 3");

    // Verify 3 SSTables exist prior to purge compaction
    let initial_count = storage.sstables_for_test().read().await.len();
    assert_eq!(
        initial_count, 3,
        "Expected 3 SSTables before purge compaction"
    );

    // 4. Set purge scope on compaction engine targeting prefix b_target_
    let scope = PurgeScope::Prefixes(vec![b"b_target_".to_vec()]);
    let probe: Arc<dyn FreeSpaceProbe> = Arc::new(LargeFreeSpaceProbe);
    storage
        .compaction_engine_for_test()
        .set_purge_scope(scope, Some(probe));

    // 5. Trigger compaction
    let compacted = storage.maybe_compact().await.expect("maybe_compact");
    assert!(
        compacted,
        "Compaction should occur because purge scope prioritized candidate SSTables"
    );

    // 6. Verify candidate SSTables (SSTable 1 and SSTable 2) were selected and merged into 1
    let final_count = storage.sstables_for_test().read().await.len();
    assert_eq!(
        final_count, 2,
        "Expected 2 SSTables after purge compaction merged the 2 target SSTables into 1"
    );

    // Clear purge scope
    storage.compaction_engine_for_test().clear_purge_scope();
}
