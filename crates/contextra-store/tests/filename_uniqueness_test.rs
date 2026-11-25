use contextra_core::{ResourceBudget, ResourceTracker, SnapshotRegistry, TxId};
use contextra_store::compaction::{CompactionConfig, CompactionEngine};
use contextra_store::sstable::BlockCache;
use contextra_store::wal::{Wal, WalOp};
use std::collections::HashSet;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_wal_seal_filename_uniqueness_rapid_repetition() {
    let dir = tempdir().unwrap();
    let mut sealed_paths = HashSet::new();
    let iterations = 50;

    for i in 0..iterations {
        let wal_path = dir.path().join(format!("wal-{i}.log"));
        let wal = Wal::open(&wal_path).await.expect("Wal::open");
        let op = WalOp::Put {
            key: vec![1, 2, 3],
            value: vec![4, 5, 6],
            tx_id: TxId::new(1),
        };
        let (batch, _) = wal
            .prepare_batch(vec![(op, 1)])
            .await
            .expect("prepare_batch");
        wal.append_batch(batch).await.expect("append_batch");

        let sealed_path = wal.rotate_and_seal().await.expect("rotate_and_seal");

        assert!(
            sealed_path.exists(),
            "Sealed WAL file must exist on disk: {:?}",
            sealed_path
        );

        let inserted = sealed_paths.insert(sealed_path.clone());
        assert!(
            inserted,
            "Filename collision detected for sealed WAL! Duplicate path: {:?}",
            sealed_path
        );
    }

    assert_eq!(sealed_paths.len(), iterations);
}

#[tokio::test]
async fn test_compaction_sst_filename_uniqueness_rapid_repetition() {
    let dir = tempdir().unwrap();
    let snapshot_reg = Arc::new(SnapshotRegistry::new());
    let cache = Arc::new(BlockCache::new(8 * 1024 * 1024));
    let budget = Arc::new(ResourceTracker::new(ResourceBudget::default()));
    let config = CompactionConfig::default();
    let engine = Arc::new(CompactionEngine::new(
        config,
        snapshot_reg,
        cache,
        None,
        budget,
        None,
    ));

    let iterations = 1000;
    let mut handles = Vec::new();

    // Spawn 10 concurrent tasks calling generate_sst_path repeatedly
    for _ in 0..10 {
        let engine_clone = Arc::clone(&engine);
        let path_clone = dir.path().to_path_buf();
        handles.push(tokio::spawn(async move {
            let mut paths = Vec::new();
            for _ in 0..(iterations / 10) {
                let sst_path = engine_clone
                    .generate_sst_path(&path_clone)
                    .expect("generate_sst_path");
                paths.push(sst_path);
            }
            paths
        }));
    }

    let mut all_paths = HashSet::new();
    for handle in handles {
        let paths = handle.await.expect("task join");
        for path in paths {
            let inserted = all_paths.insert(path.clone());
            assert!(
                inserted,
                "Filename collision detected for SSTable! Duplicate path: {:?}",
                path
            );
        }
    }

    assert_eq!(all_paths.len(), iterations);
}
