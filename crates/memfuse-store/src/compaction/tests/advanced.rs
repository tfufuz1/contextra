use super::*;

#[tokio::test(flavor = "multi_thread")]
async fn test_compaction_stress_and_gc() {
    use crate::lsm::{LsmConfig, LsmStorage};
    use memfuse_core::TxId;
    use std::sync::atomic::{AtomicBool, Ordering};

    let tmp = TempDir::new().expect("temp dir"); // expect
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 64 * 1024, // 64KB - very small to force flushes
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig {
            min_sstables_per_tier: 3, // Small tier to trigger compaction often
            size_ratio: 2.0,
            check_interval: Duration::from_millis(100), // Fast check
            yield_threshold: 100,
            max_memory_bytes: Some(128 * 1024 * 1024),
            max_io_bytes_per_second: None,
        },
        encryption_passphrase: None,
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::new(config).await.expect("create storage")); // expect
    let running = Arc::new(AtomicBool::new(true));

    // 1. Parallel Reader Task [INV-C2]
    let storage_clone = Arc::clone(&storage);
    let running_clone = Arc::clone(&running);
    let reader_handle = tokio::spawn(async move {
        let mut rng = 0u64;
        while running_clone.load(Ordering::Relaxed) {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let key_idx = rng % 10000;
            let key = format!("doc-{:04}", key_idx);

            // Randomly perform get or scan to test stability during swaps
            if rng.is_multiple_of(2) {
                let _ = storage_clone.get(key.as_bytes()).await;
            } else {
                let _ = storage_clone.scan_prefix(b"doc-").await;
            }
            tokio::task::yield_now().await;
        }
    });

    // 2. Initial Inserts
    for i in 0..1000 {
        let tx = TxId::new(i as u64);
        let key = format!("doc-{:04}", i);
        let val = vec![(i % 255) as u8; 100];
        storage.put(tx, key.as_bytes(), &val).await.expect("put"); // expect
        storage.commit(tx).await.expect("commit"); // expect
    }

    // 3. Register a Snapshot [INV-C1]
    let snapshot_seq = storage.last_seq_no().await.expect("last_seq_no"); // expect
    let _guard = storage.snapshot_registry.register(snapshot_seq);

    // 4. Heavy Load: 10,000 Inserts to trigger churn and background compaction
    for i in 0..10000 {
        let tx = TxId::new(1000 + i as u64);
        let key = format!("doc-{:04}", i);
        let val = vec![(i % 255) as u8; 100];
        storage
            .put(tx, key.as_bytes(), &val)
            .await
            .expect("put heavy"); // expect
        storage.commit(tx).await.expect("commit heavy"); // expect
    }

    // 5. Deletes
    for i in 0..5000 {
        let tx = TxId::new(20000 + i as u64);
        let key = format!("doc-{:04}", i);
        storage.delete(tx, key.as_bytes()).await.expect("delete"); // expect
        storage.commit(tx).await.expect("commit delete"); // expect
    }

    // 6. Wait for background compactions to stabilize
    let mut stabilized = false;
    for _ in 0..100 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let stats = storage.stats().await.expect("stats"); // expect
                                                           // If we have few segments, compaction is doing its job
        if stats.num_segments <= 5 {
            stabilized = true;
            break;
        }
    }

    // Stop reader and check for errors
    running.store(false, Ordering::SeqCst);
    reader_handle.await.expect("reader task panicked"); // expect

    // 7. Final Verification
    let stats = storage.stats().await.expect("final stats"); // expect
    println!(
        "Stress test finished. Final SSTable count: {}",
        stats.num_segments
    );

    // Non-deleted data MUST be present
    for i in 5000..10000 {
        let key = format!("doc-{:04}", i);
        // SAFETY: This panic is in a test-only context. A missing key after
        // compaction is a test logic error, not a production code path.
        let val = storage
            .get(key.as_bytes())
            .await
            .expect("get final") // expect
            .unwrap_or_else(|| panic!("missing key {}", key));
        assert_eq!(val[0], (i % 255) as u8);
    }

    // Deleted data MUST NOT be present in current view
    for i in 0..5000 {
        let key = format!("doc-{:04}", i);
        let val = storage.get(key.as_bytes()).await.expect("get deleted"); // expect
        assert!(val.is_none(), "Key {} should be deleted but found", key);
    }

    // SSTable count should be significantly reduced from the peak
    assert!(
        stats.num_segments <= 12,
        "Compaction failed to reduce segments: {}",
        stats.num_segments
    );
    assert!(
        stabilized,
        "Compaction didn't reach target segment count in time"
    );
}

#[tokio::test]
async fn test_compaction_swap_maintains_shadowing_order_without_restart() {
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

    // Setup 3 SSTables (A, C, B) in chronological sequence:
    // SSTable A (oldest, seq=10): [key = "val_A", key_common = "val_A_old"]
    // SSTable C (middle, seq=15): [key_common = "val_C_middle"]
    // SSTable B (newest, seq=20): [key = "val_B", key_common = "val_B_newest"]
    let sst_a = create_test_sstable(
        tmp.path(),
        "sst_a.sst",
        &[(b"key_a", b"val_A", 10), (b"key_common", b"val_A_old", 10)],
        Arc::clone(&bc),
    )
    .await;

    let sst_c = create_test_sstable(
        tmp.path(),
        "sst_c.sst",
        &[(b"key_common", b"val_C_middle", 15)],
        Arc::clone(&bc),
    )
    .await;

    let sst_b = create_test_sstable(
        tmp.path(),
        "sst_b.sst",
        &[
            (b"key_b", b"val_B", 20),
            (b"key_common", b"val_B_newest", 20),
        ],
        Arc::clone(&bc),
    )
    .await;

    // SSTable list in memory in max_seq ascending order: [A (10), C (15), B (20)]
    let sstables = Arc::new(RwLock::new(vec![
        Arc::clone(&sst_a),
        Arc::clone(&sst_c),
        Arc::clone(&sst_b),
    ]));

    // Select candidates to compact A and B (e.g. tier/fallback selection or explicit input list)
    // Here we compact input_ssts = [sst_a, sst_b] into M (max_seq = 20)
    let min_snapshot_seq = u64::MAX;
    let output_path = tmp.path().join("sst_merged_m.sst");
    engine
        .merge_sstables(
            &[sst_a.clone(), sst_b.clone()],
            &output_path,
            min_snapshot_seq,
            false,
        )
        .await
        .expect("merge A and B into M");

    let new_reader = Arc::new(
        SstableReader::open(&output_path, Arc::clone(&bc))
            .await
            .expect("open merged M"),
    );

    // Perform the swap manually inside a write guard (replicating swap logic in maybe_compact)
    let input_ssts = [sst_a, sst_b];
    {
        let mut ssts = sstables.write().await;
        let insertion_point = ssts
            .iter()
            .position(|sst| input_ssts.iter().any(|inp| Arc::ptr_eq(inp, sst)))
            .unwrap_or(ssts.len());

        ssts.retain(|sst| !input_ssts.iter().any(|inp| Arc::ptr_eq(inp, sst)));

        let insert_idx = insertion_point.min(ssts.len());
        ssts.insert(insert_idx, new_reader);

        // Re-sort SSTable list by max_seq to guarantee shadowing/visibility order.
        ssts.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);

        debug_assert!(
            ssts.windows(2)
                .all(|w| (w[0].metadata().max_seq & !TOMBSTONE_BIT)
                    <= (w[1].metadata().max_seq & !TOMBSTONE_BIT)),
            "SSTable list must be sorted by max_seq in ascending order after compaction swap"
        );
    }

    // Simulating LSM point lookup / scan (.iter().rev()):
    // Reading key_common from current sstables list in .iter().rev() order MUST find M first (seq=20),
    // returning "val_B_newest", NOT "val_C_middle" from C (seq=15).
    let ssts = sstables.read().await;
    let mut found_val = None;
    for sst in ssts.iter().rev() {
        if let Ok(Some((val, _seq, _tx))) = sst.get(b"key_common").await {
            found_val = Some(val);
            break;
        }
    }

    assert_eq!(
        found_val.expect("key_common found").as_ref(),
        b"val_B_newest",
        "Read path (.iter().rev()) must return newest value from merged SSTable M, not stale C"
    );
}

#[tokio::test]
async fn test_phantom_data_after_partial_compaction() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let engine = CompactionEngine::new(
        CompactionConfig {
            min_sstables_per_tier: 2,
            ..CompactionConfig::default()
        },
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

    // Scenario:
    // SST1 (Older): [key-1: value-1, seq 10]
    // SST2 (Newer): [key-1: tombstone, seq 20]
    // SST3 (Irrelevant): [key-x: val, seq 30]
    // Partial compaction of {SST1, SST2} must NOT GC the tombstone,
    // because we don't know if an even older version exists in some other SSTable not in the set.
    // Wait, even if we compact ALL SSTables that contain key-1, if it's not a FULL compaction
    // of the entire system, we must be conservative.

    let sst1 = create_test_sstable(
        tmp.path(),
        "sst1.sst",
        &[(b"key-1", b"value-1", 10)],
        Arc::clone(&bc),
    )
    .await;
    // seq 20 | TOMBSTONE_BIT
    let sst2 = create_test_sstable(
        tmp.path(),
        "sst2.sst",
        &[(b"key-1", &[], 20 | TOMBSTONE_BIT)],
        Arc::clone(&bc),
    )
    .await;
    let sst3 = create_test_sstable(
        tmp.path(),
        "sst3.sst",
        &[(b"key-x", b"val-x", 30)],
        Arc::clone(&bc),
    )
    .await;

    let _sstables = Arc::new(tokio::sync::RwLock::new(vec![
        Arc::clone(&sst1),
        Arc::clone(&sst2),
        Arc::clone(&sst3),
    ]));

    // min_snapshot_seq is 100 (high enough that 20 would normally be GC'd)
    let engine = Arc::new(engine);
    engine.snapshot_registry.pin(100);

    // Manually trigger merger for {sst1, sst2} -> partial compaction
    let output_path = tmp.path().join("merged.sst");
    engine
        .merge_sstables(&[sst1, sst2], &output_path, 100, false) // is_full = false
        .await
        .expect("merge"); // expect

    let reader = SstableReader::open(&output_path, Arc::clone(&bc))
        .await
        .expect("open"); // expect

    // Verify tombstone is RETAINED
    let res = reader.get(b"key-1").await.expect("get"); // expect
    assert!(
        res.is_some(),
        "Tombstone should be RETAINED in partial compaction"
    );
    let (_, seq, _) = res.unwrap(); // unwrap
    assert_eq!(seq & TOMBSTONE_BIT, TOMBSTONE_BIT);

    // Now test FULL compaction
    let output_path_full = tmp.path().join("full.sst");
    let sst1 = create_test_sstable(
        tmp.path(),
        "sst1_b.sst",
        &[(b"key-1", b"value-1", 10)],
        Arc::clone(&bc),
    )
    .await;
    let sst2 = create_test_sstable(
        tmp.path(),
        "sst2_b.sst",
        &[(b"key-1", &[], 20 | TOMBSTONE_BIT)],
        Arc::clone(&bc),
    )
    .await;

    engine
        .merge_sstables(&[sst1, sst2], &output_path_full, 100, true) // is_full = true
        .await
        .expect("merge"); // expect

    let reader_full = SstableReader::open(&output_path_full, Arc::clone(&bc))
        .await
        .expect("open"); // expect
    let res_full = reader_full.get(b"key-1").await.expect("get"); // expect
    assert!(
        res_full.is_none(),
        "Tombstone should be REMOVED in full compaction"
    );
}

#[tokio::test]
async fn test_compaction_pressure_awareness() {
    use crate::system_pressure::{PressureLevel, SystemPressure};

    let tmp = TempDir::new().expect("temp dir");
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let config = CompactionConfig {
        min_sstables_per_tier: 2,
        yield_threshold: 5, // Yield/check pressure every 5 entries
        ..Default::default()
    };

    let (pressure_tx, pressure_rx) = tokio::sync::watch::channel(SystemPressure {
        wal_queue_depth: 0,
        blocking_thread_utilization: 0.0,
        embedding_queue_depth: 0,
        pressure_level: PressureLevel::Normal,
    });

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
    )
    .with_pressure_rx(pressure_rx);

    // Create 2 SSTables with 20 entries each
    let mut entries1 = Vec::new();
    let mut entries2 = Vec::new();
    for i in 0..20u8 {
        entries1.push((
            format!("key1-{:02}", i).into_bytes(),
            b"val1".to_vec(),
            i as u64 + 1,
        ));
        entries2.push((
            format!("key2-{:02}", i).into_bytes(),
            b"val2".to_vec(),
            i as u64 + 21,
        ));
    }

    let sst1 = create_test_sstable(
        tmp.path(),
        "sst1.sst",
        &entries1
            .iter()
            .map(|(k, v, s)| (k.as_slice(), v.as_slice(), *s))
            .collect::<Vec<_>>(),
        Arc::clone(&bc),
    )
    .await;

    let sst2 = create_test_sstable(
        tmp.path(),
        "sst2.sst",
        &entries2
            .iter()
            .map(|(k, v, s)| (k.as_slice(), v.as_slice(), *s))
            .collect::<Vec<_>>(),
        Arc::clone(&bc),
    )
    .await;

    // Measure merge duration with Normal pressure
    let normal_out = tmp.path().join("normal_merged.sst");
    let start_normal = std::time::Instant::now();
    engine
        .merge_sstables(
            &[Arc::clone(&sst1), Arc::clone(&sst2)],
            &normal_out,
            u64::MAX,
            true,
        )
        .await
        .expect("merge under normal pressure");
    let duration_normal = start_normal.elapsed();

    // Switch pressure level to Critical
    pressure_tx
        .send(SystemPressure {
            wal_queue_depth: 600,
            blocking_thread_utilization: 0.9,
            embedding_queue_depth: 0,
            pressure_level: PressureLevel::Critical,
        })
        .expect("send pressure");

    // Measure merge duration with Critical pressure (yielding 4 times across 40 items -> 4 * 50ms = ~200ms delay)
    let critical_out = tmp.path().join("critical_merged.sst");
    let start_critical = std::time::Instant::now();
    engine
        .merge_sstables(&[sst1, sst2], &critical_out, u64::MAX, true)
        .await
        .expect("merge under critical pressure");
    let duration_critical = start_critical.elapsed();

    assert!(
            duration_critical >= Duration::from_millis(150),
            "Compaction under Critical pressure should take at least ~150ms due to backpressure delays, took {:?}",
            duration_critical
        );
    assert!(
            duration_critical > duration_normal,
            "Compaction under Critical pressure ({:?}) should be measurably slower than under Normal pressure ({:?})",
            duration_critical,
            duration_normal
        );

    // Verify output file content correctness under critical pressure
    let reader = SstableReader::open(&critical_out, Arc::clone(&bc))
        .await
        .expect("open critical sst");
    let entries = reader.iter().await.expect("iter entries");
    assert_eq!(
        entries.len(),
        40,
        "All entries must be preserved after backpressure merge"
    );
}

#[tokio::test]
async fn test_compaction_cancellation() {
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = crate::sstable::create_block_cache(1024);
    let config = CompactionConfig {
        min_sstables_per_tier: 2,
        size_ratio: 2.0,
        check_interval: std::time::Duration::from_millis(10),
        yield_threshold: 100,
        max_memory_bytes: None,
        max_io_bytes_per_second: None,
    };
    let engine = Arc::new(CompactionEngine::new(
        config,
        registry,
        bc,
        None,
        Arc::new(memfuse_core::ResourceTracker::new(
            memfuse_core::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    ));
    let sstables = Arc::new(tokio::sync::RwLock::new(Vec::new()));
    let tmp = tempfile::TempDir::new().unwrap(); // unwrap

    let cancel_token = tokio_util::sync::CancellationToken::new();

    // Spawn the loop
    let engine_clone = Arc::clone(&engine);
    let sstables_clone = Arc::clone(&sstables);
    let path = tmp.path().to_path_buf();
    let ct_clone = cancel_token.clone();
    let handle = tokio::spawn(async move {
        engine_clone.run_loop(sstables_clone, path, ct_clone).await;
    });

    // Let it run for a bit
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Cancel it
    cancel_token.cancel();

    // Wait for it to finish
    let result = tokio::time::timeout(std::time::Duration::from_secs(1), handle).await;
    assert!(
        result.is_ok(),
        "Compaction loop did not shut down gracefully"
    );
}

#[tokio::test]
async fn concurrent_flush_and_compact_is_safe() {
    use crate::lsm::{LsmConfig, LsmStorage};
    use memfuse_core::traits::StorageEngine;

    for _iteration in 0..10 {
        let tmp = tempfile::TempDir::new().unwrap(); // unwrap
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024,
            max_ram_mb: 64,
            tx_timeout: std::time::Duration::from_secs(60),
            compaction: CompactionConfig {
                min_sstables_per_tier: 2,
                size_ratio: 2.0,
                check_interval: std::time::Duration::from_millis(10),
                yield_threshold: 100,
                max_memory_bytes: Some(1024 * 1024),
                max_io_bytes_per_second: None,
            },
            encryption_passphrase: None,
            ..Default::default()
        };

        let storage = Arc::new(LsmStorage::new(config).await.expect("create storage")); // expect

        // Insert initial data and flush to create SSTables
        for i in 0..10u64 {
            let tx = memfuse_core::TxId::new(i + 1);
            let key = format!("key-{:04}", i);
            let val = format!("val-{:04}", i);
            storage
                .put(tx, key.as_bytes(), val.as_bytes())
                .await
                .expect("put"); // expect
            storage.commit(tx).await.expect("commit"); // expect
        }
        storage.force_flush().await.expect("flush 1"); // expect

        for i in 10..20u64 {
            let tx = memfuse_core::TxId::new(i + 1);
            let key = format!("key-{:04}", i);
            let val = format!("val-{:04}", i);
            storage
                .put(tx, key.as_bytes(), val.as_bytes())
                .await
                .expect("put"); // expect
            storage.commit(tx).await.expect("commit"); // expect
        }
        storage.force_flush().await.expect("flush 2"); // expect

        // Write un-flushed memtable data for concurrent flush task
        for i in 20..30u64 {
            let tx = memfuse_core::TxId::new(i + 1);
            let key = format!("key-{:04}", i);
            let val = format!("val-{:04}", i);
            storage
                .put(tx, key.as_bytes(), val.as_bytes())
                .await
                .expect("put"); // expect
            storage.commit(tx).await.expect("commit"); // expect
        }

        let s1 = Arc::clone(&storage);
        let s2 = Arc::clone(&storage);

        let flush_handle = tokio::spawn(async move { s1.force_flush().await });

        let compact_handle = tokio::spawn(async move { s2.maybe_compact().await });

        let (flush_res, compact_res) = tokio::join!(flush_handle, compact_handle);
        flush_res
            .expect("flush task joined") // expect
            .expect("flush succeeded"); // expect
        compact_res
            .expect("compact task joined") // expect
            .expect("compact succeeded"); // expect

        // Verify data readability
        for i in 0..30u64 {
            let key = format!("key-{:04}", i);
            let expected_val = format!("val-{:04}", i);
            let val = storage
                .get(key.as_bytes())
                .await
                .expect("get") // expect
                .expect("key must exist"); // expect
            assert_eq!(val, expected_val.as_bytes());
        }
    }
}

#[tokio::test]
async fn test_compaction_single_lock_candidate_selection_concurrency() {
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
    for i in 0..3u8 {
        let sst = create_test_sstable(
            tmp.path(),
            &format!("sst-{}.sst", i),
            &[(format!("key-{}", i).as_bytes(), b"val", i as u64 + 1)],
            Arc::clone(&bc),
        )
        .await;
        sstables.write().await.push(sst);
    }

    // 1. Obtain selected candidate Arcs in a single read lock call
    let candidates = engine
        .select_compaction_candidates(&*sstables.read().await)
        .expect("candidates selected");
    assert_eq!(candidates.len(), 3);

    // 2. Simulate concurrent modification (flush/rollback/pop/clear) on sstables
    let extra_sst = create_test_sstable(
        tmp.path(),
        "sst-concurrent-flush.sst",
        &[(b"concurrent-key", b"val", 99)],
        Arc::clone(&bc),
    )
    .await;
    {
        let mut guard = sstables.write().await;
        guard.remove(0); // remove item 0 (rollback/compaction modification)
        guard.push(extra_sst); // append new sstable (concurrent flush)
    }

    // 3. Verify candidates acquired in step 1 are completely decoupled from index changes
    // in sstables list and can be safely merged without out-of-bounds panics.
    let output = tmp.path().join("merged_decoupled.sst");
    let merge_res = engine
        .merge_sstables(&candidates, &output, u64::MAX, true)
        .await;
    assert!(
        merge_res.is_ok(),
        "Merge of Arc candidates must succeed regardless of list modifications"
    );

    let reader = SstableReader::open(&output, Arc::clone(&bc))
        .await
        .expect("open merged sst");
    let entries = reader.iter().await.expect("iter entries");
    assert_eq!(
        entries.len(),
        3,
        "All 3 original Arc candidates must be merged safely"
    );
}

#[tokio::test]
async fn test_mvcc_floor_version_retained_for_active_snapshot() {
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

    // Key "k1" has 3 versions: seq 30 ("v30"), seq 20 ("v20"), seq 10 ("v10")
    let sst3 = create_test_sstable(
        tmp.path(),
        "sst3.sst",
        &[(b"k1", b"v30", 30)],
        Arc::clone(&bc),
    )
    .await;

    let sst2 = create_test_sstable(
        tmp.path(),
        "sst2.sst",
        &[(b"k1", b"v20", 20)],
        Arc::clone(&bc),
    )
    .await;

    let sst1 = create_test_sstable(
        tmp.path(),
        "sst1.sst",
        &[(b"k1", b"v10", 10)],
        Arc::clone(&bc),
    )
    .await;

    let output = tmp.path().join("merged_mvcc_floor.sst");

    // Active snapshot pinned at min_snapshot_seq = 15.
    // Versions > 15: seq 30, seq 20 (MUST both be retained).
    // Floor version (newest <= 15): seq 10 ("v10") (MUST be retained).
    engine
        .merge_sstables(&[sst1, sst2, sst3], &output, 15, true)
        .await
        .expect("merge");

    let reader = SstableReader::open(&output, Arc::clone(&bc))
        .await
        .expect("open merged");
    let entries = reader.iter().await.expect("iter");

    assert_eq!(
        entries.len(),
        3,
        "All 3 versions (seq 30, seq 20, seq 10) must be retained when min_snapshot_seq = 15"
    );
    assert_eq!(entries[0].2 & !TOMBSTONE_BIT, 30);
    assert_eq!(entries[1].2 & !TOMBSTONE_BIT, 20);
    assert_eq!(entries[2].2 & !TOMBSTONE_BIT, 10);
}

#[tokio::test]
async fn test_chain_linkage_tier_grouping() {
    let tmp = TempDir::new().expect("temp dir");
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let config = CompactionConfig {
        min_sstables_per_tier: 3,
        size_ratio: 2.0,
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

    // SSTables with sizes 100, 180, 320
    // Under single-linkage (against 100):
    // 180 / 100 = 1.8 <= 2.0 (fits)
    // 320 / 100 = 3.2 > 2.0 (would NOT fit)
    // Under chain-linkage (against neighbor 180):
    // 180 / 100 = 1.8 <= 2.0
    // 320 / 180 = 1.77 <= 2.0 (FITS in tier under chain-linkage!)

    async fn create_padded_sst(
        dir: &std::path::Path,
        name: &str,
        size_target_kb: usize,
        seq: u64,
        bc: Arc<BlockCache>,
    ) -> Arc<SstableReader> {
        let path = dir.join(name);
        let mut builder = SstableBuilder::create(&path).await.expect("create sst");
        let pad = vec![0u8; 1024];
        for i in 0..size_target_kb {
            let k = format!("k-{:06}", i);
            builder
                .add(k.as_bytes(), &pad, seq, seq)
                .await
                .expect("add entry");
        }
        builder.finish().await.expect("finish sst");
        Arc::new(SstableReader::open(&path, bc).await.expect("open sst"))
    }

    let sst1 = create_padded_sst(tmp.path(), "sst1.sst", 10, 1, Arc::clone(&bc)).await;
    let sst2 = create_padded_sst(tmp.path(), "sst2.sst", 18, 2, Arc::clone(&bc)).await;
    let sst3 = create_padded_sst(tmp.path(), "sst3.sst", 32, 3, Arc::clone(&bc)).await;

    let candidates = engine
        .select_compaction_candidates(&[sst1, sst2, sst3])
        .expect("chain linkage should select all 3 SSTables into a single tier");

    assert_eq!(
        candidates.len(),
        3,
        "Chain linkage must group [10, 18, 32] into a tier with size_ratio = 2.0"
    );
}

#[tokio::test]
async fn test_compaction_concurrent_rollback_flush_no_panic() {
    let tmp = TempDir::new().expect("temp dir");
    let registry = Arc::new(SnapshotRegistry::new());
    let bc = create_block_cache(1);
    let config = CompactionConfig {
        min_sstables_per_tier: 2,
        yield_threshold: 1, // force frequent yields during merge
        ..CompactionConfig::default()
    };
    let engine = Arc::new(CompactionEngine::new(
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
    ));

    let sstables = Arc::new(RwLock::new(Vec::new()));
    // Populate initial SSTables
    for i in 0..5u8 {
        let sst = create_test_sstable(
            tmp.path(),
            &format!("sst-init-{}.sst", i),
            &[(format!("key-{}", i).as_bytes(), b"val", i as u64 + 1)],
            Arc::clone(&bc),
        )
        .await;
        sstables.write().await.push(sst);
    }

    let engine_clone = Arc::clone(&engine);
    let sstables_clone = Arc::clone(&sstables);
    let tmp_path = tmp.path().to_path_buf();

    // Spawn concurrent task simulating rollback (shortening list) and flush (appending list)
    let mutator_cancel = tokio_util::sync::CancellationToken::new();
    let ct = mutator_cancel.clone();
    let sstables_mut = Arc::clone(&sstables);
    let bc_mut = Arc::clone(&bc);
    let tmp_path_mut = tmp.path().to_path_buf();

    let mutator_handle = tokio::spawn(async move {
        let mut counter = 100u8;
        while !ct.is_cancelled() {
            tokio::time::sleep(Duration::from_millis(1)).await;
            let mut guard = sstables_mut.write().await;
            if !guard.is_empty() && counter.is_multiple_of(2) {
                // Simulate rollback / compaction cleanup: remove an entry
                guard.pop();
            } else {
                // Simulate concurrent flush: append a new SSTable
                counter += 1;
                let new_sst = create_test_sstable(
                    &tmp_path_mut,
                    &format!("sst-mut-{}.sst", counter),
                    &[(
                        format!("key-mut-{}", counter).as_bytes(),
                        b"val",
                        counter as u64,
                    )],
                    Arc::clone(&bc_mut),
                )
                .await;
                guard.push(new_sst);
            }
        }
    });

    // Run maybe_compact multiple times during concurrent list mutations
    for _ in 0..10 {
        let res = engine_clone.maybe_compact(&sstables_clone, &tmp_path).await;
        // Compaction must return Ok(true) or Ok(false), never panic or error out bounds
        assert!(
                res.is_ok(),
                "maybe_compact must complete cleanly without panic or unexpected error under concurrent list mutation"
            );
    }

    mutator_cancel.cancel();
    let _ = mutator_handle.await;
}

#[tokio::test]
async fn test_tombstone_retention_floor_with_active_snapshot() {
    use memfuse_core::TOMBSTONE_BIT;

    let tmp = TempDir::new().expect("temp dir");
    let bc = create_block_cache(1);
    let manifest = Arc::new(
        crate::manifest::Manifest::open(tmp.path().join("MANIFEST"))
            .await
            .expect("manifest"),
    );
    let registry = Arc::new(SnapshotRegistry::new());

    let mut config = CompactionConfig::default();
    config.min_sstables_per_tier = 2;

    let budget = Arc::new(memfuse_core::ResourceTracker::new(
        memfuse_core::ResourceBudget {
            memory_limit: 100 * 1024 * 1024,
        },
    ));
    let engine = CompactionEngine::new(
        config,
        registry.clone(),
        bc.clone(),
        None,
        budget,
        Some(manifest),
    );

    // Active snapshot pinned at seq 15
    registry.pin(15);

    // Input 1: Put at seq 20, Tombstone at seq 18, Put at seq 5.
    // Active snapshot is pinned at seq 15.
    let sst1 = create_test_sstable(
        tmp.path(),
        "sst-ts-1.sst",
        &[
            (b"key-1", b"new_val", 20),
            (b"key-1", b"", 18 | TOMBSTONE_BIT),
            (b"key-1", b"old_val", 5),
        ],
        Arc::clone(&bc),
    )
    .await;

    let output_path = tmp.path().join("sst-compacted.sst");
    engine
        .merge_sstables(&[sst1], &output_path, 15, true)
        .await
        .expect("compaction succeeds");

    let reader = SstableReader::open(&output_path, bc)
        .await
        .expect("open reader");
    let entries = reader.iter().await.expect("iter entries");

    // Under min_snapshot_seq = 15:
    // 1) seq 20 (>= 15): kept.
    // 2) seq 18 (>= 15, tombstone): kept because raw_seq >= min_snapshot_seq.
    // 3) seq 5 (< 15): floor version below min_snapshot_seq, kept.
    assert_eq!(
        entries.len(),
        3,
        "Expected 3 entries (seq 20, seq 18 tombstone, seq 5 floor)"
    );
    assert_eq!(entries[0].2 & !TOMBSTONE_BIT, 20);
    assert_eq!(entries[1].2 & !TOMBSTONE_BIT, 18);
    assert_ne!(
        entries[1].2 & TOMBSTONE_BIT,
        0,
        "seq 18 must be a tombstone"
    );
    assert_eq!(entries[2].2 & !TOMBSTONE_BIT, 5);
}
