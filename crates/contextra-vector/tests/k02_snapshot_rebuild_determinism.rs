#![allow(clippy::unwrap_used, clippy::expect_used)]

// FILE-CONTEXT: Integration test verifying K-02 SSI search_at(seq) determinism under concurrent HNSW rebuild.
// Spec v17 (Part 4.3, K-02; Part 6.5 "Pin-first").

use contextra_core::{DocId, TxId, VectorIndex};
use contextra_ports::{Rng, SeededRng};
use contextra_vector::{HnswConfig, HnswIndex, RebuildStatus};
use std::sync::Arc;

const TEST_SEEDS: [u64; 4] = [
    0x1234_5678_9ABC_DEF0,
    0xCAFE_BABE_1234_5678,
    0xFEED_FACE_0000_1111,
    0x9876_5432_10FE_DCBA,
];

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_k02_snapshot_rebuild_determinism() {
    for &seed in &TEST_SEEDS {
        run_snapshot_rebuild_determinism_for_seed(seed).await;
    }
}

async fn run_snapshot_rebuild_determinism_for_seed(seed: u64) {
    let rng_index: Arc<dyn Rng> = Arc::new(SeededRng::new(seed));
    let rng_data = SeededRng::new(seed ^ 0x5A5A_5A5A_5A5A_5A5A);

    const DIM: usize = 8;
    const NUM_VECTORS: usize = 300;
    const K: usize = 20;

    let config = HnswConfig {
        dimension: DIM,
        m: 16,
        ef_construction: 64,
        ef_search: 64,
        rebuild_threshold: 0.50,
        distance_metric: contextra_core::DistanceMetric::Euclidean,
        ..Default::default()
    };

    let index = Arc::new(HnswIndex::try_new_with_rng(config, rng_index).expect("valid index"));

    // Generate 300 deterministic vectors
    let mut vectors = Vec::with_capacity(NUM_VECTORS);
    for _ in 0..NUM_VECTORS {
        let mut vec = vec![0.0f32; DIM];
        for elem in vec.iter_mut() {
            *elem = rng_data.next_unit_f64() as f32;
        }
        vectors.push(vec);
    }

    // 1. Insert 300 vectors at tx1 (seq = 1)
    let tx1 = TxId::new(1);
    for (i, vec) in vectors.iter().enumerate() {
        index
            .insert(tx1, DocId::new((i + 1) as u64), vec)
            .await
            .expect("insert baseline vector");
    }
    index.commit(tx1).await.expect("commit tx1");

    // 2. Soft-delete 160 vectors at tx2 (seq = 2)
    let tx2 = TxId::new(2);
    for i in 1..=160 {
        index
            .delete(tx2, DocId::new(i as u64))
            .await
            .expect("delete doc");
    }
    index.commit(tx2).await.expect("commit tx2");

    assert!(
        index.is_rebuild_required(),
        "Index must report rebuild required after deleting 160/300 nodes (seed {})",
        seed
    );

    // Create 3 query vectors deterministically
    let mut queries = Vec::new();
    for _ in 0..3 {
        let mut q = vec![0.0f32; DIM];
        for elem in q.iter_mut() {
            *elem = rng_data.next_unit_f64() as f32;
        }
        queries.push(q);
    }

    // Reference search_at(query, K, 2) BEFORE rebuild
    let mut reference_results = Vec::new();
    for q in &queries {
        let ref_run1 = index
            .search_at(q, K, 2)
            .await
            .expect("reference search_at run 1");
        let ref_run2 = index
            .search_at(q, K, 2)
            .await
            .expect("reference search_at run 2");

        let ids1: Vec<DocId> = ref_run1.iter().map(|d| d.doc_id).collect();
        let ids2: Vec<DocId> = ref_run2.iter().map(|d| d.doc_id).collect();

        assert_eq!(
            ids1, ids2,
            "Baseline search_at without rebuild must be strictly deterministic (seed {})",
            seed
        );
        reference_results.push((ref_run1, ids1));
    }

    // 3. Concurrently search while triggering rebuild
    let index_clone = Arc::clone(&index);
    let queries_clone = queries.clone();
    let reference_results_clone = reference_results.clone();

    let concurrent_search_task = tokio::spawn(async move {
        let mut concurrent_runs = Vec::new();

        // Poll status until rebuild starts or finishes
        while index_clone.rebuild_status() != RebuildStatus::Running
            && index_clone.rebuild_count() == 0
        {
            tokio::time::sleep(std::time::Duration::from_micros(100)).await;
        }

        // Execute repeated search_at queries during rebuild
        for _ in 0..5 {
            for (q_idx, q) in queries_clone.iter().enumerate() {
                let res = index_clone
                    .search_at(q, K, 2)
                    .await
                    .expect("concurrent search_at at seq 2");
                let res_ids: Vec<DocId> = res.iter().map(|d| d.doc_id).collect();
                assert_eq!(
                    res_ids, reference_results_clone[q_idx].1,
                    "Concurrent search_at result (q_idx {}) during rebuild MUST match reference search_at results (seed {})",
                    q_idx, seed
                );
                concurrent_runs.push((q_idx, res_ids));
            }
            tokio::time::sleep(std::time::Duration::from_micros(100)).await;
        }

        concurrent_runs
    });

    // Trigger rebuild
    index.rebuild().await.expect("rebuild must succeed");

    // Wait for concurrent task to finish
    let concurrent_runs = concurrent_search_task
        .await
        .expect("concurrent search task panicked");

    assert!(
        !concurrent_runs.is_empty(),
        "At least one concurrent search run must have executed (seed {})",
        seed
    );

    // 4. Post-rebuild search_at(query, k, 2) verification
    for (q_idx, q) in queries.iter().enumerate() {
        let post_res = index
            .search_at(q, K, 2)
            .await
            .expect("post-rebuild search_at at seq 2");
        let post_ids: Vec<DocId> = post_res.iter().map(|d| d.doc_id).collect();

        assert_eq!(
            post_ids, reference_results[q_idx].1,
            "Post-rebuild search_at result (q_idx {}) MUST match pre-rebuild reference search_at results (seed {})",
            q_idx, seed
        );
    }
}
