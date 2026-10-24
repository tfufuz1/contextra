// ZWECK: Property-/Beispieltest für zusammenhängende Kandidatenauswahl (S-01) und Tombstone-Schutz bei aktivem Snapshot.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use bytes::Bytes;
use contextra_core::{StorageEngine, TxId};
use contextra_store::compaction::adaptive::{CostBasedAdaptivePlanner, WorkloadMetricsSnapshot};
use contextra_store::compaction::{CompactionConfig, CompactionEngine};
use contextra_store::{LsmConfig, LsmStorage};
use tempfile::tempdir;

#[tokio::test]
async fn test_compaction_candidate_selection_is_contiguous() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        compaction: CompactionConfig {
            min_sstables_per_tier: 4,
            size_ratio: 4.0,
            enable_adaptive_compaction: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let storage = LsmStorage::open(config.clone())
        .await
        .expect("open storage");

    // Write multiple SSTables by putting data, committing, and forcing flushes
    let mut tx_counter = 1u64;
    for batch in 0..8 {
        let tx = TxId::new(tx_counter);
        tx_counter += 1;
        for i in 0..10 {
            let k = format!("key_{:02}_{:02}", batch, i);
            let v = format!("val_{:02}_{:02}", batch, i);
            storage
                .put(tx, k.as_bytes(), v.as_bytes())
                .await
                .expect("put");
        }
        storage.commit(tx).await.expect("commit batch");
        storage.flush().await.expect("flush");
    }

    let sstables_lock = storage.sstables_for_test();
    let sstables_guard = sstables_lock.read().await;
    assert!(
        sstables_guard.len() >= 8,
        "Expected at least 8 SSTables after 8 flushes, got {}",
        sstables_guard.len()
    );

    let comp_engine = CompactionEngine::new(
        config.compaction.clone(),
        storage.snapshot_registry.clone(),
        storage.block_cache_for_test(),
        None,
        storage.budget_for_test(),
        Some(storage.manifest_for_test()),
    );

    // Test select_compaction_candidates directly
    let candidates = comp_engine.select_compaction_candidates(&sstables_guard);
    if let Some(cand_vec) = candidates {
        // Find position of cand_vec[0] in sstables_guard
        let start_pos = sstables_guard
            .iter()
            .position(|s| std::sync::Arc::ptr_eq(s, &cand_vec[0]))
            .expect("First candidate must exist in SSTables list");

        // Verify that every subsequent candidate in cand_vec is strictly adjacent in sstables_guard
        for (idx, cand) in cand_vec.iter().enumerate() {
            assert!(
                std::sync::Arc::ptr_eq(cand, &sstables_guard[start_pos + idx]),
                "Compaction candidates must be contiguous without skipping intermediate SSTables (S-01)"
            );
        }
    }

    // Test CostBasedAdaptivePlanner plan_compaction
    let planner = CostBasedAdaptivePlanner::new(0.70, 4, 4.0);
    use contextra_store::AdaptiveCompactionPlanner;
    let stats = contextra_core::StorageStats {
        num_segments: sstables_guard.len(),
        total_size_bytes: sstables_guard.iter().map(|s| s.metadata().file_size).sum(),
        memtable_size_bytes: 0,
    };
    let metrics = WorkloadMetricsSnapshot {
        read_count: 100,
        write_count: 100,
        last_seq_no: 100,
    };

    let plan_res = planner
        .plan_compaction(&stats, &metrics, &sstables_guard, 0)
        .expect("plan_compaction");

    if let Some(plan) = plan_res {
        let start_pos = sstables_guard
            .iter()
            .position(|s| std::sync::Arc::ptr_eq(s, &plan.candidates[0]))
            .expect("First candidate in adaptive plan must exist in SSTables list");

        for (idx, cand) in plan.candidates.iter().enumerate() {
            assert!(
                std::sync::Arc::ptr_eq(cand, &sstables_guard[start_pos + idx]),
                "Adaptive planner compaction candidates must be contiguous without skipping intermediate SSTables (S-01)"
            );
        }
    }
}

#[tokio::test]
async fn test_tombstone_retained_when_active_snapshot_exists() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        compaction: CompactionConfig {
            min_sstables_per_tier: 2,
            size_ratio: 4.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let storage = LsmStorage::open(config.clone())
        .await
        .expect("open storage");

    // 1. Put key_a = val_1 and commit
    let tx1 = TxId::new(1);
    storage
        .put(tx1, b"key_a", b"val_1")
        .await
        .expect("put val_1");
    storage.commit(tx1).await.expect("commit tx1");
    storage.flush().await.expect("flush 1");

    let snap_seq = storage.next_seq_no_for_test().saturating_sub(1);
    storage.snapshot_registry.pin(snap_seq);

    // 2. Delete key_a (creates tombstone) and commit
    let tx2 = TxId::new(2);
    storage.delete(tx2, b"key_a").await.expect("delete key_a");
    storage.commit(tx2).await.expect("commit tx2");
    storage.flush().await.expect("flush 2");

    // 3. Verify snapshot at snap_seq can still see val_1
    let snap_val = storage
        .get_at_seq(b"key_a", snap_seq)
        .await
        .expect("get_at_seq");
    assert_eq!(
        snap_val,
        Some(Bytes::from("val_1")),
        "Active snapshot must see pre-deletion value"
    );

    // 4. Trigger compaction
    let _ = storage.maybe_compact_for_test().await;

    // 5. Verify active snapshot can STILL see val_1 after compaction because tombstone was NOT illegally purged
    let snap_val_after = storage
        .get_at_seq(b"key_a", snap_seq)
        .await
        .expect("get_at_seq after compaction");
    assert_eq!(
        snap_val_after,
        Some(Bytes::from("val_1")),
        "Active snapshot must still see pre-deletion value after compaction"
    );
}
