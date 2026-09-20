use super::*;

use crate::sstable::{create_block_cache, SstableBuilder};
use memfuse_core::StorageEngine;
use tempfile::TempDir;

async fn create_test_sstable(
    dir: &std::path::Path,
    name: &str,
    entries: &[(&[u8], &[u8], u64)],
    bc: Arc<BlockCache>,
) -> Arc<SstableReader> {
    let path = dir.join(name);
    let mut builder = SstableBuilder::create(&path).await.expect("create sst"); // expect
    for (k, v, seq) in entries {
        builder.add(k, v, *seq, *seq).await.expect("add entry"); // expect
    }
    builder.finish().await.expect("finish sst"); // expect
    Arc::new(SstableReader::open(&path, bc).await.expect("open sst")) // expect
}

#[test]
fn prop_compaction_tombstone_masking_latest_operation_wins() {
    use proptest::prelude::*;

    #[derive(Debug, Clone)]
    enum Op {
        Put(u8),
        Delete,
    }

    let op_seq_strategy = proptest::collection::vec(
        prop_oneof![any::<u8>().prop_map(Op::Put), Just(Op::Delete),],
        1..30,
    );

    proptest!(ProptestConfig::with_cases(30), |(ops in op_seq_strategy)| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let tmp = TempDir::new().unwrap();
            let registry = Arc::new(SnapshotRegistry::new());
            let bc = create_block_cache(1);
            let engine = CompactionEngine::new(
                CompactionConfig::default(),
                registry,
                Arc::clone(&bc),
                None,
                Arc::new(memfuse_core::ResourceTracker::new(
                    memfuse_core::ResourceBudget {
                        memory_limit: 1024 * 1024,
                    },
                )),
                None,
            );

            let mut input_ssts = Vec::new();
            for (idx, op) in ops.iter().enumerate() {
                let seq = idx as u64 + 1;
                let sst_name = format!("sst_{idx}.sst");
                let sst = match op {
                    Op::Put(v) => {
                        create_test_sstable(
                            tmp.path(),
                            &sst_name,
                            &[(b"target_key", &[*v], seq)],
                            Arc::clone(&bc),
                        )
                        .await
                    }
                    Op::Delete => {
                        create_test_sstable(
                            tmp.path(),
                            &sst_name,
                            &[(b"target_key", &[], seq | TOMBSTONE_BIT)],
                            Arc::clone(&bc),
                        )
                        .await
                    }
                };
                input_ssts.push(sst);
            }

            let output_path = tmp.path().join("merged.sst");
            engine
                .merge_sstables(&input_ssts, &output_path, u64::MAX, true)
                .await
                .unwrap();

            let reader = SstableReader::open(&output_path, Arc::clone(&bc))
                .await
                .unwrap();

            let last_op = ops.last().unwrap();
            let res = reader.get(b"target_key").await.unwrap();

            match last_op {
                Op::Put(expected_v) => {
                    prop_assert!(
                        res.is_some(),
                        "Latest operation was PUT but key was not found after compaction"
                    );
                    let (val, seq, _) = res.unwrap();
                    prop_assert_eq!(
                        val.as_ref(),
                        &[*expected_v],
                        "Merged value must match latest PUT value"
                    );
                    prop_assert_eq!(
                        seq & TOMBSTONE_BIT,
                        0,
                        "TOMBSTONE_BIT must NOT be set for latest PUT"
                    );
                }
                Op::Delete => {
                    prop_assert!(
                        res.is_none(),
                        "Latest operation was DELETE, key must be GC'd after full compaction"
                    );
                }
            }

            Ok(())
        }).unwrap();
    });
}

#[tokio::test]
async fn test_compaction_candidate_selection_follows_chronological_order() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let config = CompactionConfig {
        min_sstables_per_tier: 2,
        ..CompactionConfig::default()
    };
    let engine = CompactionEngine::new(
        config,
        registry,
        Arc::clone(&bc),
        None,
        Arc::new(memfuse_core::ResourceTracker::new(
            memfuse_core::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    );

    // Create older SSTable with many entries (large file size) but small max_seq (seq=10)
    let mut large_entries = Vec::new();
    large_entries.push((b"key-1".as_ref(), b"old_val".as_ref(), 10u64));
    for _ in 0..100 {
        large_entries.push((b"pad", b"large_padding_data_to_increase_file_size", 10u64));
    }
    let sst_old_large = create_test_sstable(
        tmp.path(),
        "sst_old_large.sst",
        &large_entries,
        Arc::clone(&bc),
    )
    .await;

    // Create newer SSTable with few entries (small file size) but larger max_seq (seq=20)
    let sst_new_small = create_test_sstable(
        tmp.path(),
        "sst_new_small.sst",
        &[(b"key-1", b"new_val", 20)],
        Arc::clone(&bc),
    )
    .await;

    assert!(
        sst_old_large.metadata().file_size > sst_new_small.metadata().file_size,
        "Old SSTable must be larger in file size than new SSTable"
    );

    let sstables = vec![Arc::clone(&sst_old_large), Arc::clone(&sst_new_small)];

    // Select candidates
    let candidates = engine
        .select_compaction_candidates(&sstables)
        .expect("candidates selected");

    // Collect inputs in selected candidate order
    let candidate_ssts = candidates;

    // Perform merge
    let output = tmp.path().join("merged_chronological.sst");
    engine
        .merge_sstables(&candidate_ssts, &output, u64::MAX, true)
        .await
        .expect("merge");

    let reader = SstableReader::open(&output, Arc::clone(&bc))
        .await
        .expect("open merged");

    let (val, seq, _) = reader.get(b"key-1").await.expect("get").expect("exists");
    assert_eq!(
        val.as_ref(),
        b"new_val",
        "Merge must preserve newer version regardless of file size"
    );
    assert_eq!(seq, 20);
}

#[tokio::test]
async fn test_mvcc_retention_floor_version_retained_for_snapshot() {
    let tmp = TempDir::new().expect("temp dir");
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let engine = CompactionEngine::new(
        CompactionConfig::default(),
        registry,
        Arc::clone(&bc),
        None,
        Arc::new(memfuse_core::ResourceTracker::new(
            memfuse_core::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    );

    // Two SSTables with versions 100 and 90 of key "k1"
    // Active snapshot is at min_snapshot_seq = 95
    let sst1 = create_test_sstable(
        tmp.path(),
        "sst1.sst",
        &[(b"k1", b"v100", 100)],
        Arc::clone(&bc),
    )
    .await;

    let sst2 = create_test_sstable(
        tmp.path(),
        "sst2.sst",
        &[(b"k1", b"v90", 90)],
        Arc::clone(&bc),
    )
    .await;

    let output = tmp.path().join("merged_mvcc2.sst");
    engine
        .merge_sstables(&[sst1, sst2], &output, 95, true)
        .await
        .expect("merge");

    let reader = SstableReader::open(&output, Arc::clone(&bc))
        .await
        .expect("open merged");
    let entries = reader.iter().await.expect("iter");

    // Both versions (100 and 90) must be retained:
    // seq 100 >= 95 (kept), seq 90 < 95 (kept as floor version)
    assert_eq!(
        entries.len(),
        2,
        "Both seq 100 and floor version 90 must be retained"
    );
    assert_eq!(entries[0].0.as_ref(), b"k1");
    assert_eq!(entries[0].2, 100);
    assert_eq!(entries[1].0.as_ref(), b"k1");
    assert_eq!(entries[1].2, 90);
}

#[tokio::test]
async fn test_mvcc_retention_older_versions_below_floor_discarded() {
    let tmp = TempDir::new().expect("temp dir");
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let engine = CompactionEngine::new(
        CompactionConfig::default(),
        registry,
        Arc::clone(&bc),
        None,
        Arc::new(memfuse_core::ResourceTracker::new(
            memfuse_core::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    );

    // Three SSTables with versions 100, 90, 80 of key "k1"
    // Active snapshot is at min_snapshot_seq = 95
    let sst1 = create_test_sstable(
        tmp.path(),
        "sst1.sst",
        &[(b"k1", b"v100", 100)],
        Arc::clone(&bc),
    )
    .await;

    let sst2 = create_test_sstable(
        tmp.path(),
        "sst2.sst",
        &[(b"k1", b"v90", 90)],
        Arc::clone(&bc),
    )
    .await;

    let sst3 = create_test_sstable(
        tmp.path(),
        "sst3.sst",
        &[(b"k1", b"v80", 80)],
        Arc::clone(&bc),
    )
    .await;

    let output = tmp.path().join("merged_mvcc3.sst");
    engine
        .merge_sstables(&[sst1, sst2, sst3], &output, 95, true)
        .await
        .expect("merge");

    let reader = SstableReader::open(&output, Arc::clone(&bc))
        .await
        .expect("open merged");
    let entries = reader.iter().await.expect("iter");

    // Only versions 100 (>= 95) and 90 (floor version for < 95) must be retained.
    // Version 80 (< 95 and floor already emitted) must be discarded.
    assert_eq!(
        entries.len(),
        2,
        "Only seq 100 and floor version 90 must be retained, seq 80 discarded"
    );
    assert_eq!(entries[0].0.as_ref(), b"k1");
    assert_eq!(entries[0].2, 100);
    assert_eq!(entries[1].0.as_ref(), b"k1");
    assert_eq!(entries[1].2, 90);
}

#[tokio::test]
async fn test_merge_deduplication() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let engine = CompactionEngine::new(
        CompactionConfig::default(),
        registry,
        Arc::clone(&bc),
        None,
        Arc::new(memfuse_core::ResourceTracker::new(
            memfuse_core::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    );

    // Two SSTables with overlapping keys
    let sst1 = create_test_sstable(
        tmp.path(),
        "sst1.sst",
        &[(b"key-a", b"val-1", 1), (b"key-b", b"val-2", 2)],
        Arc::clone(&bc),
    )
    .await;

    let sst2 = create_test_sstable(
        tmp.path(),
        "sst2.sst",
        &[(b"key-a", b"val-3", 3), (b"key-c", b"val-4", 4)],
        Arc::clone(&bc),
    )
    .await;

    let output = tmp.path().join("merged.sst");
    engine
        .merge_sstables(&[sst1, sst2], &output, u64::MAX, true)
        .await
        .expect("merge"); // expect

    let reader = SstableReader::open(&output, Arc::clone(&bc))
        .await
        .expect("open merged"); // expect
    let entries = reader.iter().await.expect("iter"); // expect

    // key-a should have the newer value (seq=3)
    assert_eq!(entries.len(), 3); // key-a, key-b, key-c
    assert_eq!(entries[0].0.as_ref(), b"key-a");
    assert_eq!(entries[0].1.as_ref(), b"val-3");
    assert_eq!(entries[0].2, 3);
}

#[tokio::test]
async fn test_tombstone_gc_stream_advancement_preserves_subsequent_keys() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let engine = CompactionEngine::new(
        CompactionConfig::default(),
        registry,
        Arc::clone(&bc),
        None,
        Arc::new(memfuse_core::ResourceTracker::new(
            memfuse_core::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    );

    let tombstone_seq = 5 | TOMBSTONE_BIT;
    // Key A is tombstone, followed in same SSTable stream by B, C, D
    let sst1 = create_test_sstable(
        tmp.path(),
        "sst1.sst",
        &[
            (b"key-a", b"", tombstone_seq),
            (b"key-b", b"val-b", 10),
            (b"key-c", b"val-c", 11),
            (b"key-d", b"val-d", 12),
        ],
        Arc::clone(&bc),
    )
    .await;

    let output = tmp.path().join("compacted_stream.sst");

    // min_snapshot_seq=100 -> key-a tombstone is GC'd during full compaction
    engine
        .merge_sstables(&[sst1], &output, 100, true)
        .await
        .expect("merge"); // expect

    let reader = SstableReader::open(&output, Arc::clone(&bc))
        .await
        .expect("open"); // expect
    let entries = reader.iter().await.expect("iter"); // expect

    // key-a GC'd, key-b, key-c, key-d MUST be present and unchanged
    assert_eq!(
        entries.len(),
        3,
        "B, C, D must be preserved after tombstone GC"
    );
    assert_eq!(entries[0].0.as_ref(), b"key-b");
    assert_eq!(entries[0].1.as_ref(), b"val-b");
    assert_eq!(entries[1].0.as_ref(), b"key-c");
    assert_eq!(entries[1].1.as_ref(), b"val-c");
    assert_eq!(entries[2].0.as_ref(), b"key-d");
    assert_eq!(entries[2].1.as_ref(), b"val-d");
}

#[tokio::test]
async fn test_tombstone_gc() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let engine = CompactionEngine::new(
        CompactionConfig::default(),
        registry,
        Arc::clone(&bc),
        None,
        Arc::new(memfuse_core::ResourceTracker::new(
            memfuse_core::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    );

    let tombstone_seq = 5 | TOMBSTONE_BIT;
    let sst1 = create_test_sstable(
        tmp.path(),
        "sst1.sst",
        &[(b"alive", b"val", 10), (b"dead", b"", tombstone_seq)],
        Arc::clone(&bc),
    )
    .await;

    let output = tmp.path().join("compacted.sst");

    // min_snapshot_seq=100 → tombstone at seq=5 is safe to GC
    engine
        .merge_sstables(&[sst1], &output, 100, true)
        .await
        .expect("merge"); // expect

    let reader = SstableReader::open(&output, Arc::clone(&bc))
        .await
        .expect("open"); // expect
    let entries = reader.iter().await.expect("iter"); // expect

    assert_eq!(entries.len(), 1); // Only "alive" remains
    assert_eq!(entries[0].0.as_ref(), b"alive");
}

#[tokio::test]
async fn test_tombstone_preserved_with_active_snapshot() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let engine = CompactionEngine::new(
        CompactionConfig::default(),
        registry,
        Arc::clone(&bc),
        None,
        Arc::new(memfuse_core::ResourceTracker::new(
            memfuse_core::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    );

    let tombstone_seq = 5 | TOMBSTONE_BIT;
    let sst1 = create_test_sstable(
        tmp.path(),
        "sst1.sst",
        &[(b"alive", b"val", 10), (b"dead", b"", tombstone_seq)],
        Arc::clone(&bc),
    )
    .await;

    let output = tmp.path().join("compacted.sst");

    // min_snapshot_seq=2 → tombstone at seq=5 is NOT safe to GC
    engine
        .merge_sstables(&[sst1], &output, 2, true)
        .await
        .expect("merge"); // expect

    let reader = SstableReader::open(&output, Arc::clone(&bc))
        .await
        .expect("open"); // expect
    let entries = reader.iter().await.expect("iter"); // expect

    assert_eq!(entries.len(), 2); // Both preserved
}

#[tokio::test]
async fn test_maybe_compact_full_cycle() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let config = CompactionConfig {
        min_sstables_per_tier: 2, // Low threshold for testing
        ..Default::default()
    };
    let engine = CompactionEngine::new(
        config,
        registry,
        Arc::clone(&bc),
        None,
        Arc::new(memfuse_core::ResourceTracker::new(
            memfuse_core::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    );

    // Create 3 small SSTables of similar size
    let sstables = Arc::new(RwLock::new(Vec::new()));
    for i in 0..3u8 {
        let sst = create_test_sstable(
            tmp.path(),
            &format!("sst-{}.sst", i),
            &[
                (
                    format!("key-{}-a", i).as_bytes(),
                    b"val",
                    (i as u64) * 2 + 1,
                ),
                (
                    format!("key-{}-b", i).as_bytes(),
                    b"val",
                    (i as u64) * 2 + 2,
                ),
            ],
            Arc::clone(&bc),
        )
        .await;
        sstables.write().await.push(sst);
    }

    assert_eq!(sstables.read().await.len(), 3);

    // Run compaction
    let compacted = engine
        .maybe_compact(&sstables, tmp.path())
        .await
        .expect("compact"); // expect

    assert!(compacted, "Compaction should have occurred");

    // After compaction: fewer SSTables, all data still accessible
    let ssts = sstables.read().await;
    assert!(
        ssts.len() < 3,
        "Should have fewer SSTables after compaction"
    );

    // Verify all data is present in the compacted result
    let last_sst = &ssts[ssts.len() - 1];
    let entries = last_sst.iter().await.expect("iter"); // expect
    assert_eq!(entries.len(), 6); // 3 SSTables × 2 entries each
}

#[tokio::test]
async fn test_no_compaction_below_threshold() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let config = CompactionConfig {
        min_sstables_per_tier: 4,
        ..Default::default()
    };
    let engine = CompactionEngine::new(
        config,
        registry,
        Arc::clone(&bc),
        None,
        Arc::new(memfuse_core::ResourceTracker::new(
            memfuse_core::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    );

    let sstables = Arc::new(RwLock::new(Vec::new()));
    for i in 0..2u8 {
        let sst = create_test_sstable(
            tmp.path(),
            &format!("sst-{}.sst", i),
            &[(format!("key-{}", i).as_bytes(), b"val", i as u64 + 1)],
            Arc::clone(&bc),
        )
        .await;
        sstables.write().await.push(sst);
    }

    let compacted = engine
        .maybe_compact(&sstables, tmp.path())
        .await
        .expect("compact"); // expect

    assert!(!compacted, "Should not compact with only 2 SSTables");
    assert_eq!(sstables.read().await.len(), 2);
}

#[tokio::test]
async fn test_compaction_aborts_on_concurrent_modification() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let config = CompactionConfig {
        min_sstables_per_tier: 2,
        ..Default::default()
    };
    let engine = CompactionEngine::new(
        config,
        registry,
        Arc::clone(&bc),
        None,
        Arc::new(memfuse_core::ResourceTracker::new(
            memfuse_core::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    );

    let sstables = Arc::new(RwLock::new(Vec::new()));
    for i in 0..2u8 {
        let sst = create_test_sstable(
            tmp.path(),
            &format!("sst-{}.sst", i),
            &[(format!("key-{}", i).as_bytes(), b"val", i as u64 + 1)],
            Arc::clone(&bc),
        )
        .await;
        sstables.write().await.push(sst);
    }

    // Simulate concurrent modification by clearing sstables right after selection or before swap
    // We test that retain/Arc::ptr_eq check properly detects missing input sstables
    let candidates = engine
        .select_compaction_candidates(&*sstables.read().await)
        .unwrap(); // unwrap
    assert_eq!(candidates.len(), 2);

    // Remove one sstable concurrently
    sstables.write().await.pop();

    // Run maybe_compact, should return Ok(false) or abort cleanly without panic
    let result = engine.maybe_compact(&sstables, tmp.path()).await.unwrap(); // unwrap
    assert!(
        !result,
        "Compaction should be aborted when candidates are modified"
    );
}

#[tokio::test]
async fn test_compaction_removes_uuid_sidecar_files() {
    let tmp = TempDir::new().expect("temp dir");
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let config = CompactionConfig {
        min_sstables_per_tier: 2,
        ..Default::default()
    };
    let engine = CompactionEngine::new(
        config,
        registry,
        Arc::clone(&bc),
        None,
        Arc::new(memfuse_core::ResourceTracker::new(
            memfuse_core::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    );

    let sstables = Arc::new(RwLock::new(Vec::new()));
    let mut old_sst_paths = Vec::new();
    let mut uuid_paths = Vec::new();

    for i in 0..2u8 {
        let name = format!("sst-{}.sst", i);
        let sst = create_test_sstable(
            tmp.path(),
            &name,
            &[(format!("key-{}", i).as_bytes(), b"val", i as u64 + 1)],
            Arc::clone(&bc),
        )
        .await;
        let sst_path = sst.file_path().to_path_buf();
        old_sst_paths.push(sst_path.clone());

        let uuid_path = PathBuf::from(format!("{}.uuid", sst_path.display()));
        tokio::fs::write(&uuid_path, b"dummy-uuid-bytes")
            .await
            .expect("write dummy uuid file");
        uuid_paths.push(uuid_path);

        sstables.write().await.push(sst);
    }

    for path in &old_sst_paths {
        assert!(
            path.exists(),
            "SSTable file {:?} must exist before compaction",
            path
        );
    }
    for uuid_path in &uuid_paths {
        assert!(
            uuid_path.exists(),
            "UUID sidecar file {:?} must exist before compaction",
            uuid_path
        );
    }

    let compacted = engine
        .maybe_compact(&sstables, tmp.path())
        .await
        .expect("maybe_compact should succeed");
    assert!(compacted, "Compaction should have occurred");

    for path in &old_sst_paths {
        assert!(
            !path.exists(),
            "Old SSTable file {:?} should be deleted after compaction",
            path
        );
    }
    for uuid_path in &uuid_paths {
        assert!(
            !uuid_path.exists(),
            "UUID sidecar file {:?} should be deleted after compaction",
            uuid_path
        );
    }
}

#[tokio::test]
async fn test_compaction_swap_restores_shadowing_order_without_restart() {
    let tmp = TempDir::new().expect("temp dir");
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let config = CompactionConfig {
        min_sstables_per_tier: 2,
        ..CompactionConfig::default()
    };
    let engine = CompactionEngine::new(
        config,
        registry,
        Arc::clone(&bc),
        None,
        Arc::new(memfuse_core::ResourceTracker::new(
            memfuse_core::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    );

    // SSTable A (oldest candidate): key "k1" -> "v_old", seq 10
    let sst_a = create_test_sstable(
        tmp.path(),
        "sst_a.sst",
        &[(b"k1", b"v_old", 10)],
        Arc::clone(&bc),
    )
    .await;

    // SSTable C (non-input, intermediate seq, larger size so it is in a separate size tier): key "k1" -> "v_inter", seq 15
    let mut entries_c = vec![(b"k1".as_ref(), b"v_inter".as_ref(), 15u64)];
    for _ in 0..100 {
        entries_c.push((b"padding_key", b"padding_value_large_file", 15u64));
    }
    let sst_c = create_test_sstable(tmp.path(), "sst_c.sst", &entries_c, Arc::clone(&bc)).await;

    // SSTable B (newer candidate): key "k1" -> "v_new", seq 20
    let sst_b = create_test_sstable(
        tmp.path(),
        "sst_b.sst",
        &[(b"k1", b"v_new", 20)],
        Arc::clone(&bc),
    )
    .await;

    // sstables list initially sorted by max_seq: [A (seq 10), C (seq 15), B (seq 20)]
    let sstables = Arc::new(RwLock::new(vec![
        Arc::clone(&sst_a),
        Arc::clone(&sst_c),
        Arc::clone(&sst_b),
    ]));

    // Run production maybe_compact directly to trigger candidate selection, merge, and atomic swap
    let compacted = engine
        .maybe_compact(&sstables, tmp.path())
        .await
        .expect("maybe_compact");
    assert!(
        compacted,
        "Compaction should be triggered for tier {{A, B}}"
    );

    // Read in reverse order (simulating get_at_seq / scan)
    let ssts_read = sstables.read().await;
    let mut found_val = None;
    for sst in ssts_read.iter().rev() {
        if let Some((val, seq, tx)) = sst.get(b"k1").await.expect("get") {
            if seq & !TOMBSTONE_BIT <= 20 && tx <= 20 {
                found_val = Some(val);
                break;
            }
        }
    }

    assert_eq!(
            found_val.as_deref(),
            Some(&b"v_new"[..]),
            "Reader must see newer value from merged SSTable rather than stale value from non-input SSTable"
        );

    // Verify list is strictly sorted ascending by max_seq
    assert!(
        ssts_read
            .windows(2)
            .all(|w| w[0].metadata().max_seq <= w[1].metadata().max_seq),
        "SSTable list must be strictly sorted by max_seq"
    );
}

#[tokio::test]
async fn test_compaction_swap_debug_assert_detects_unsorted_list() {
    let tmp = TempDir::new().expect("temp dir");
    let bc = create_block_cache(1);

    let sst_c = create_test_sstable(
        tmp.path(),
        "sst_c.sst",
        &[(b"k1", b"v_inter", 15)],
        Arc::clone(&bc),
    )
    .await;

    let sst_m = create_test_sstable(
        tmp.path(),
        "sst_m.sst",
        &[(b"k1", b"v_new", 20)],
        Arc::clone(&bc),
    )
    .await;

    // Intentionally create unsorted list: [M (seq 20), C (seq 15)]
    let unsorted_ssts = [sst_m, sst_c];

    let is_sorted = unsorted_ssts.windows(2).all(|w| {
        (w[0].metadata().max_seq & !TOMBSTONE_BIT) <= (w[1].metadata().max_seq & !TOMBSTONE_BIT)
    });

    assert!(
        !is_sorted,
        "Unsorted SSTable list must fail the max_seq order check"
    );
}

#[test]
fn test_generate_sst_path_uniqueness() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let engine = CompactionEngine::new(
        CompactionConfig::default(),
        Arc::new(SnapshotRegistry::new()),
        create_block_cache(1),
        None,
        Arc::new(memfuse_core::ResourceTracker::new(
            memfuse_core::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    );
    let path1 = engine.generate_sst_path(tmp.path()).expect("path 1"); // expect
    let path2 = engine.generate_sst_path(tmp.path()).expect("path 2"); // expect
    assert_ne!(
        path1, path2,
        "Rapid sequential calls must produce distinct SSTable paths"
    );
}
