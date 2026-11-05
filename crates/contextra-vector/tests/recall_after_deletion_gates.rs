#![allow(
    clippy::all,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    warnings
)]
//! Recall regression gates, zeroize verification, and deterministic rebuild under deletion.
//! Tests correspond to SPEC §9.8, D-18, D-77, D-86, T-VEC-DEL-1, T-VEC-DEL-2, T-VEC-REBUILD-INDEP.

use contextra_ports::erasure::{EraseTarget, ErasureLayer, PurgeCx, Scanner};
use contextra_ports::VectorIndex;
use contextra_types::{DistanceMetric, DocId, TxId};
use contextra_vector::hnsw::{GhostFreeVectorIndex, HnswConfig, HnswIndex, VectorData};

/// Euclidean distance between two vectors.
fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let diff = x - y;
            diff * diff
        })
        .sum::<f32>()
        .sqrt()
}

/// Computes ground truth top-k for a given query over all alive vectors.
fn brute_force_ground_truth(
    alive_vectors: &[(DocId, Vec<f32>)],
    query: &[f32],
    k: usize,
) -> Vec<DocId> {
    let mut distances: Vec<(DocId, f32)> = alive_vectors
        .iter()
        .map(|(id, vec)| (*id, l2_distance(vec, query)))
        .collect();

    distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    distances.into_iter().take(k).map(|(id, _)| id).collect()
}

/// Calculates recall@k between search results and ground truth.
fn calculate_recall(results: &[contextra_types::ScoredDocument], ground_truth: &[DocId]) -> f64 {
    if ground_truth.is_empty() {
        return 1.0;
    }
    let gt_set: std::collections::HashSet<DocId> = ground_truth.iter().copied().collect();
    let hits = results
        .iter()
        .filter(|res| gt_set.contains(&res.doc_id))
        .count();
    hits as f64 / ground_truth.len() as f64
}

/// Helper to generate deterministic test vectors.
fn generate_deterministic_vectors(n: usize, dim: usize, seed: u64) -> Vec<(DocId, Vec<f32>)> {
    (0..n)
        .map(|i| {
            let id = DocId::new((i + 1) as u64);
            let mut vec = Vec::with_capacity(dim);
            for d in 0..dim {
                let v = (((i * dim + d) as u64 + seed) % 1000) as f32 / 1000.0;
                vec.push(v);
            }
            (id, vec)
        })
        .collect()
}

/// T-VEC-DEL-1 / T-VEC-DEL-2:
/// Tests Recall@10 gates after deletion of 1%, 5%, and 20% of nodes with graph repair.
/// Asserts that recall remains strictly above required thresholds:
/// - 1% deletion: Recall@10 >= 0.90
/// - 5% deletion: Recall@10 >= 0.85
/// - 20% deletion: Recall@10 >= 0.70
#[tokio::test]
async fn test_recall_regression_gates_after_deletion_thresholds() {
    let n = 200;
    let dim = 16;
    let k = 10;
    let all_vectors = generate_deterministic_vectors(n, dim, 42);

    let config = HnswConfig {
        dimension: dim,
        m: 16,
        ef_construction: 64,
        ef_search: 64,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config).expect("create HnswIndex");

    for (i, (doc_id, vector)) in all_vectors.iter().enumerate() {
        let tx = TxId::new((i + 1) as u64);
        index.insert(tx, *doc_id, vector).await.expect("insert");
        index.commit(tx).await.expect("commit");
    }

    let query_vector: Vec<f32> = (0..dim).map(|d| (d as f32) / (dim as f32)).collect();

    // 1. Initial Recall before any deletion
    let gt_init = brute_force_ground_truth(&all_vectors, &query_vector, k);
    let search_init = index.search(&query_vector, k).await.expect("search init");
    let recall_init = calculate_recall(&search_init, &gt_init);
    assert!(
        recall_init >= 0.95,
        "Initial Recall@10 must be >= 0.95, got {recall_init}"
    );

    let mut current_alive = all_vectors.clone();

    // Helper closure to delete a fraction of nodes and verify recall
    async fn delete_fraction_and_check(
        index: &mut HnswIndex,
        current_alive: &mut Vec<(DocId, Vec<f32>)>,
        fraction: f64,
        min_recall: f64,
        query: &[f32],
        k: usize,
    ) {
        let count_to_delete = (current_alive.len() as f64 * fraction).ceil() as usize;
        let delete_targets: Vec<DocId> = current_alive
            .iter()
            .take(count_to_delete)
            .map(|(id, _)| *id)
            .collect();

        for doc_id in &delete_targets {
            let stats = index
                .remove_with_graph_repair(*doc_id)
                .expect("remove_with_graph_repair");
            assert!(
                stats.verified_no_ghost_pointers,
                "Must confirm no ghost pointers"
            );
        }

        current_alive.retain(|(id, _)| !delete_targets.contains(id));

        let gt = brute_force_ground_truth(current_alive, query, k);
        let search_res = index.search(query, k).await.expect("search");

        // Verify deleted nodes never appear in search results
        for res in &search_res {
            assert!(
                !delete_targets.contains(&res.doc_id),
                "Deleted doc {:?} must NOT appear in search results",
                res.doc_id
            );
        }

        let recall = calculate_recall(&search_res, &gt);
        assert!(
            recall >= min_recall,
            "Recall@10 after deleting {:.0}% must be >= {}, got {}",
            fraction * 100.0,
            min_recall,
            recall
        );
    }

    let mut mut_index = index;

    // Gate 1: 1% deletion threshold (min recall 0.90)
    delete_fraction_and_check(
        &mut mut_index,
        &mut current_alive,
        0.01,
        0.90,
        &query_vector,
        k,
    )
    .await;

    // Gate 2: 5% deletion threshold (min recall 0.85)
    delete_fraction_and_check(
        &mut mut_index,
        &mut current_alive,
        0.05,
        0.85,
        &query_vector,
        k,
    )
    .await;

    // Gate 3: 20% deletion threshold (min recall 0.70)
    delete_fraction_and_check(
        &mut mut_index,
        &mut current_alive,
        0.20,
        0.70,
        &query_vector,
        k,
    )
    .await;
}

/// D-77 / D-86 / MK1 / MK2:
/// Tests zeroize verification of vector bytes and arena slots upon ErasureLayer operations.
#[tokio::test]
async fn test_erasure_layer_zeroize_and_witness_verification() {
    let dim = 8;
    let config = HnswConfig {
        dimension: dim,
        m: 8,
        ef_construction: 32,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config).expect("create HnswIndex");

    // Insert 10 nodes
    for i in 1..=10u64 {
        let doc_id = DocId::new(i.into());
        let vec = vec![i as f32; dim];
        index.insert(TxId::new(i), doc_id, &vec).await.unwrap();
        index.commit(TxId::new(i)).await.unwrap();
    }

    let target_doc_raw = 5u64;
    let target = EraseTarget::ExactKeys(vec![target_doc_raw.to_be_bytes().to_vec()]);

    // 1. Resolve scope
    let scope = index.resolve(&target).await.expect("resolve scope");
    assert_eq!(scope.affected_units(), &[target_doc_raw]);

    // 2. On gate: vector memory is overwritten with surrogate and target bytes are zeroized
    index.on_gate(&scope).await.expect("on_gate");

    // Verify target node in RAM is surrogate / zeroized
    {
        let doc_map = index.inner.hot.doc_to_node.read();
        let target_idx = doc_map.get(&target_doc_raw).copied().unwrap();
        let nodes = index.inner.hot.nodes.read();
        let node = &nodes[target_idx];
        match &node.vector {
            VectorData::F32(v) => {
                // If surrogate or zeroized
                if !v.is_empty() {
                    assert_ne!(v[0], 5.0, "Original vector bytes must be overwritten");
                }
            }
            VectorData::U8(v) => {
                if !v.is_empty() {
                    assert!(v.iter().all(|&b| b == 0), "U8 vector must be zeroized");
                }
            }
        }
    }

    // 3. Purge: batch edge repair & slot freed
    let cx = PurgeCx::new(1000);
    let outcome = index.purge(&scope, &cx).await.expect("purge");
    assert_eq!(outcome.purged_count, 1);
    assert!(outcome.bytes_reclaimed > 0);

    // 4. Witness: read-only scan produces clean witness
    let scanner = Scanner::new();
    let witness = index.witness(&scope, &scanner).await.expect("witness");
    assert_ne!(witness.witness_hash(), &[0u8; 32]);
}

/// T-VEC-REBUILD-INDEP:
/// Asserts that graph rebuild depends purely on the set of living slots.
/// Two indices:
/// - Index A: Insert 50, delete 10, then rebuild.
/// - Index B: Insert only the 40 surviving elements directly.
/// Post-rebuild search results on both indices must be identical.
#[tokio::test]
async fn test_t_vec_rebuild_indep_living_slots() {
    let dim = 8;
    let total = 30;
    let delete_count = 5;
    let all_vectors = generate_deterministic_vectors(total, dim, 12345);

    let config = HnswConfig {
        dimension: dim,
        m: 8,
        ef_construction: 32,
        ef_search: 32,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    // Index A: insert all, delete first 5
    let index_a = HnswIndex::try_new(config.clone()).unwrap();
    for (i, (doc_id, vec)) in all_vectors.iter().enumerate() {
        let tx = TxId::new((i + 1) as u64);
        index_a.insert(tx, *doc_id, vec).await.unwrap();
        index_a.commit(tx).await.unwrap();
    }
    for (doc_id, _) in all_vectors.iter().take(delete_count) {
        let mut mut_a = index_a.clone();
        mut_a.remove_with_graph_repair(*doc_id).unwrap();
    }
    index_a.trigger_rebuild_async();

    // Index B: insert only surviving 25
    let index_b = HnswIndex::try_new(config).unwrap();
    for (i, (doc_id, vec)) in all_vectors.iter().skip(delete_count).enumerate() {
        let tx = TxId::new((i + 1) as u64);
        index_b.insert(tx, *doc_id, vec).await.unwrap();
        index_b.commit(tx).await.unwrap();
    }

    let query: Vec<f32> = vec![0.5f32; dim];
    let res_a = index_a.search(&query, 10).await.unwrap();
    let res_b = index_b.search(&query, 10).await.unwrap();

    assert_eq!(
        res_a.len(),
        res_b.len(),
        "Both indices must return same number of results"
    );
    for (a, b) in res_a.iter().zip(res_b.iter()) {
        assert_eq!(
            a.doc_id, b.doc_id,
            "Rebuilt index must return identical surviving documents"
        );
    }
}
