#![allow(clippy::unwrap_used, clippy::expect_used)]

// FILE-CONTEXT: Integration test verifying HNSW snapshot search consistency before, during, and after index rebuilds. (TS: 2026-09-11)

use contextra_core::{DocId, TxId, VectorIndex};
use contextra_ports::{Rng, SeededRng};
use contextra_vector::{HnswConfig, HnswIndex, RebuildStatus};
use std::sync::Arc;

const TEST_SEEDS: [u64; 8] = [
    0x1234_5678_9ABC_DEF0,
    0xDEAD_BEEF_CAFE_BABE,
    0x0000_0000_0000_1001,
    0x0000_0000_0000_2002,
    0x0000_0000_0000_3003,
    0x0000_0000_0000_4004,
    0x0000_0000_0000_5005,
    0x0000_0000_0000_6006,
];

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_hnsw_rebuild_snapshot_consistency_during_concurrent_search() {
    for &seed in &TEST_SEEDS {
        run_hnsw_rebuild_snapshot_consistency_for_seed(seed).await;
    }
}

async fn run_hnsw_rebuild_snapshot_consistency_for_seed(seed: u64) {
    let rng: Arc<dyn Rng> = Arc::new(SeededRng::new(seed));

    // 1. Initialize HNSW index with rebuild_threshold = 0.50 (rebuild required when active ratio < 50%)
    let config = HnswConfig {
        dimension: 4,
        m: 16,
        ef_construction: 64,
        rebuild_threshold: 0.50,
        distance_metric: contextra_core::DistanceMetric::Euclidean,
        ..Default::default()
    };

    let index = Arc::new(HnswIndex::try_new_with_rng(config, rng).expect("valid index"));

    // 2. Populate index with 40 baseline vectors at tx1 (seq=1)
    let tx1 = TxId::new(1);
    for i in 1..=40 {
        let val = i as f32;
        let vec = [val, val * 0.1, 0.0, 0.0];
        index
            .insert(tx1, DocId::new(i), &vec)
            .await
            .expect("insert baseline");
    }
    index.commit(tx1).await.expect("commit tx1");

    // 3. Obtain reference search results at pinned snapshot seq=1 BEFORE any deletes or rebuilds
    index.pin_snapshot(1);
    let query = [10.0, 1.0, 0.0, 0.0];
    let reference_results = index
        .search_at(&query, 10, 1)
        .await
        .expect("reference search at seq 1");

    assert_eq!(
        reference_results.len(),
        10,
        "Reference search must return top 10 documents for seed {}",
        seed
    );

    // Extract reference doc_ids and scores
    let reference_docs: Vec<_> = reference_results
        .iter()
        .map(|d| (d.doc_id.inner(), (d.score * 10000.0).round() as i64))
        .collect();

    // 4. Soft-delete 25 vectors at tx2 (seq=2) to drop active ratio below 50% (15/40 = 37.5% active)
    let tx2 = TxId::new(2);
    for i in 1..=25 {
        index.delete(tx2, DocId::new(i)).await.expect("delete doc");
    }
    index.commit(tx2).await.expect("commit tx2");

    // Verify index reports rebuild required
    assert!(
        index.is_rebuild_required(),
        "HNSW index must indicate rebuild is required after deleting >50% of nodes (seed {})",
        seed
    );

    // 5. Targeted timing harness:
    // Spawn a search task that explicitly waits until `rebuilding` flag transitions to true
    // (rebuild_status() == RebuildStatus::Running), ensuring search executes inside the critical
    // swap window (Phase 1 build & Phase 2 merge/swap).
    let index_clone = Arc::clone(&index);
    let query_clone = query;

    let targeted_search_task = tokio::spawn(async move {
        // Poll status indicator until rebuild starts or finishes
        while index_clone.rebuild_status() != RebuildStatus::Running
            && index_clone.rebuild_count() == 0
        {
            tokio::time::sleep(std::time::Duration::from_micros(200)).await;
        }

        let mut search_runs = Vec::new();
        // Execute 3 searches sequentially right after rebuild status becomes Running / during rebuild
        for _ in 0..3 {
            let res = index_clone
                .search_at(&query_clone, 10, 1)
                .await
                .expect("concurrent search at seq 1 during rebuild");
            search_runs.push(res);
            tokio::time::sleep(std::time::Duration::from_micros(100)).await;
        }

        search_runs
    });

    // Explicitly trigger 2-phase rebuild
    index.rebuild().await.expect("rebuild must succeed");

    // Await targeted search task
    let concurrent_search_runs = targeted_search_task
        .await
        .expect("targeted search task panicked");

    // 6. Verify ALL search runs during/around critical rebuild window match baseline snapshot reference IDENTICALLY
    for (run_idx, concurrent_results) in concurrent_search_runs.iter().enumerate() {
        let concurrent_docs: Vec<_> = concurrent_results
            .iter()
            .map(|d| (d.doc_id.inner(), (d.score * 10000.0).round() as i64))
            .collect();

        assert_eq!(
            concurrent_docs, reference_docs,
            "Search results run {} at pinned snapshot seq=1 during rebuild MUST be 100% identical to baseline reference search (seed {})",
            run_idx, seed
        );
    }

    // 7. Verify post-rebuild search_at(query, 10, 1) also remains IDENTICAL
    let post_rebuild_results = index
        .search_at(&query, 10, 1)
        .await
        .expect("post-rebuild search at seq 1");

    let post_rebuild_docs: Vec<_> = post_rebuild_results
        .iter()
        .map(|d| (d.doc_id.inner(), (d.score * 10000.0).round() as i64))
        .collect();

    assert_eq!(
        post_rebuild_docs, reference_docs,
        "Search results at pinned snapshot seq=1 AFTER rebuild MUST remain 100% identical to baseline reference search (seed {})",
        seed
    );
}
