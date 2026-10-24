#![allow(clippy::unwrap_used, clippy::expect_used, clippy::needless_range_loop)]

use contextra_core::{DistanceMetric, DocId, TxId, VectorIndex};
use contextra_ports::SeededRng;
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};
use std::sync::Arc;

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 32) as f32) / 4294967296.0
    }

    fn next_vector(&mut self, dim: usize) -> Vec<f32> {
        let v: Vec<f32> = (0..dim).map(|_| self.next_f32() * 2.0 - 1.0).collect();
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-6 {
            v.into_iter().map(|x| x / norm).collect()
        } else {
            v
        }
    }
}

fn compute_ground_truth(
    vectors: &[(DocId, Vec<f32>)],
    query: &[f32],
    k: usize,
) -> Vec<(DocId, f32)> {
    let mut distances: Vec<(DocId, f32)> = vectors
        .iter()
        .map(|(id, vec)| {
            let dot = query
                .iter()
                .zip(vec.iter())
                .map(|(&a, &b)| a * b)
                .sum::<f32>();
            let score = dot;
            (*id, score)
        })
        .collect();

    distances.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    distances.truncate(k);
    distances
}

#[tokio::test]
async fn test_hnsw_seed_determinism_identical_indices() {
    let dim = 32;
    let num_vecs = 300;
    let seed = 424242u64;

    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(64)
        .ef_search(32)
        .distance_metric(DistanceMetric::Cosine)
        .build()
        .expect("Valid config");

    let rng1 = Arc::new(SeededRng::new(seed));
    let rng2 = Arc::new(SeededRng::new(seed));

    let idx1 = HnswIndex::try_new_with_rng(config.clone(), rng1).expect("idx1 creation");
    let idx2 = HnswIndex::try_new_with_rng(config.clone(), rng2).expect("idx2 creation");

    let mut data_rng = SimpleRng::new(1001);
    let mut dataset = Vec::with_capacity(num_vecs);
    for i in 1..=num_vecs {
        let doc_id = DocId::new(i as u64);
        let vec = data_rng.next_vector(dim);
        dataset.push((doc_id, vec));
    }

    let tx = TxId::new(1);
    for (doc_id, vec) in &dataset {
        idx1.insert(tx, *doc_id, vec).await.expect("insert idx1");
        idx2.insert(tx, *doc_id, vec).await.expect("insert idx2");
    }

    idx1.commit(tx).await.expect("commit idx1");
    idx2.commit(tx).await.expect("commit idx2");

    // 1. Compare layer assignments
    let layers1 = idx1.all_doc_ids_and_layers();
    let layers2 = idx2.all_doc_ids_and_layers();

    assert_eq!(layers1.len(), layers2.len());
    assert_eq!(
        layers1, layers2,
        "Layer assignments must be identical given same seed and order"
    );

    // 2. Compare 100 queries bitwise
    let mut query_rng = SimpleRng::new(9999);
    for q_idx in 0..100 {
        let query = query_rng.next_vector(dim);
        let res1 = idx1.search(&query, 10).await.expect("search idx1");
        let res2 = idx2.search(&query, 10).await.expect("search idx2");

        assert_eq!(
            res1.len(),
            res2.len(),
            "Query {q_idx}: result count mismatch"
        );
        for (i, (r1, r2)) in res1.iter().zip(res2.iter()).enumerate() {
            assert_eq!(
                r1.doc_id, r2.doc_id,
                "Query {q_idx}, Rank {i}: doc_id mismatch ({:?} vs {:?})",
                r1.doc_id, r2.doc_id
            );
            assert_eq!(
                r1.score.to_bits(),
                r2.score.to_bits(),
                "Query {q_idx}, Rank {i}: bitwise score mismatch ({} vs {})",
                r1.score,
                r2.score
            );
        }
    }
}

#[tokio::test]
async fn test_hnsw_seed_determinism_different_seeds_different_layouts() {
    let dim = 32;
    let num_vecs = 300;

    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(64)
        .ef_search(32)
        .distance_metric(DistanceMetric::Cosine)
        .build()
        .expect("Valid config");

    let rng1 = Arc::new(SeededRng::new(11111));
    let rng2 = Arc::new(SeededRng::new(99999));

    let idx1 = HnswIndex::try_new_with_rng(config.clone(), rng1).expect("idx1 creation");
    let idx2 = HnswIndex::try_new_with_rng(config.clone(), rng2).expect("idx2 creation");

    let mut data_rng = SimpleRng::new(2002);
    let mut dataset = Vec::with_capacity(num_vecs);
    for i in 1..=num_vecs {
        let doc_id = DocId::new(i as u64);
        let vec = data_rng.next_vector(dim);
        dataset.push((doc_id, vec));
    }

    let tx = TxId::new(1);
    for (doc_id, vec) in &dataset {
        idx1.insert(tx, *doc_id, vec).await.expect("insert idx1");
        idx2.insert(tx, *doc_id, vec).await.expect("insert idx2");
    }

    idx1.commit(tx).await.expect("commit idx1");
    idx2.commit(tx).await.expect("commit idx2");

    let layers1 = idx1.all_doc_ids_and_layers();
    let layers2 = idx2.all_doc_ids_and_layers();

    assert_ne!(
        layers1, layers2,
        "Different seeds must result in different HNSW layer layouts"
    );

    // Evaluate recall@10 for both indices against ground truth
    let mut query_rng = SimpleRng::new(8888);
    let num_queries = 50;
    let top_k = 10;
    let mut hits1 = 0;
    let mut hits2 = 0;

    for _ in 0..num_queries {
        let query = query_rng.next_vector(dim);
        let gt = compute_ground_truth(&dataset, &query, top_k);
        let gt_set: std::collections::HashSet<DocId> = gt.iter().map(|(id, _)| *id).collect();

        let res1 = idx1.search(&query, top_k).await.expect("search idx1");
        let res2 = idx2.search(&query, top_k).await.expect("search idx2");

        for r in &res1 {
            if gt_set.contains(&r.doc_id) {
                hits1 += 1;
            }
        }
        for r in &res2 {
            if gt_set.contains(&r.doc_id) {
                hits2 += 1;
            }
        }
    }

    let recall1 = (hits1 as f32) / ((num_queries * top_k) as f32);
    let recall2 = (hits2 as f32) / ((num_queries * top_k) as f32);

    println!("Index 1 (seed 11111) Recall@10: {recall1:.4}");
    println!("Index 2 (seed 99999) Recall@10: {recall2:.4}");

    assert!(
        recall1 >= 0.85,
        "Index 1 recall must be >= 0.85, got {recall1:.4}"
    );
    assert!(
        recall2 >= 0.85,
        "Index 2 recall must be >= 0.85, got {recall2:.4}"
    );
}

#[tokio::test]
async fn test_hnsw_seed_determinism_rebuild_reproducibility() {
    let dim = 32;
    let num_vecs = 300;
    let seed = 77777u64;

    // Set rebuild_threshold = 0.0 so auto-rebuild is disabled during commit
    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(64)
        .ef_search(32)
        .distance_metric(DistanceMetric::Cosine)
        .rebuild_threshold(0.0)
        .build()
        .expect("Valid config");

    let rng1 = Arc::new(SeededRng::new(seed));
    let rng2 = Arc::new(SeededRng::new(seed));

    let idx1 = HnswIndex::try_new_with_rng(config.clone(), rng1).expect("idx1 creation");
    let idx2 = HnswIndex::try_new_with_rng(config.clone(), rng2).expect("idx2 creation");

    let mut data_rng = SimpleRng::new(3003);
    let mut dataset = Vec::with_capacity(num_vecs);
    for i in 1..=num_vecs {
        let doc_id = DocId::new(i as u64);
        let vec = data_rng.next_vector(dim);
        dataset.push((doc_id, vec));
    }

    let tx1 = TxId::new(1);
    for (doc_id, vec) in &dataset {
        idx1.insert(tx1, *doc_id, vec).await.expect("insert idx1");
        idx2.insert(tx1, *doc_id, vec).await.expect("insert idx2");
    }
    idx1.commit(tx1).await.expect("commit idx1 tx1");
    idx2.commit(tx1).await.expect("commit idx2 tx1");

    // Delete 40 documents (~13% > 10% rebuild threshold)
    let tx2 = TxId::new(2);
    let to_delete_count = 40;
    for item in dataset.iter().take(to_delete_count) {
        let doc_id = item.0;
        idx1.delete(tx2, doc_id).await.expect("delete idx1");
        idx2.delete(tx2, doc_id).await.expect("delete idx2");
    }
    idx1.commit(tx2).await.expect("commit idx1 tx2");
    idx2.commit(tx2).await.expect("commit idx2 tx2");

    // Rebuild both indices explicitly given identical initial seed
    idx1.rebuild().await.expect("rebuild idx1");
    idx2.rebuild().await.expect("rebuild idx2");

    // Compare layer assignments of rebuilt indices
    let layers1 = idx1.all_doc_ids_and_layers();
    let layers2 = idx2.all_doc_ids_and_layers();

    assert_eq!(
        layers1, layers2,
        "Rebuilt indices with identical seed and dataset must produce identical layer layouts"
    );

    // Verify search query results are bitwise identical between the two rebuilt indices
    let mut query_rng = SimpleRng::new(5555);
    for q_idx in 0..50 {
        let query = query_rng.next_vector(dim);
        let res1 = idx1.search(&query, 10).await.expect("search rebuilt idx1");
        let res2 = idx2.search(&query, 10).await.expect("search rebuilt idx2");

        assert_eq!(
            res1.len(),
            res2.len(),
            "Query {q_idx}: rebuilt result count mismatch"
        );
        for (i, (r1, r2)) in res1.iter().zip(res2.iter()).enumerate() {
            assert_eq!(
                r1.doc_id, r2.doc_id,
                "Query {q_idx}, Rank {i}: rebuilt doc_id mismatch ({:?} vs {:?})",
                r1.doc_id, r2.doc_id
            );
            assert_eq!(
                r1.score.to_bits(),
                r2.score.to_bits(),
                "Query {q_idx}, Rank {i}: rebuilt bitwise score mismatch ({} vs {})",
                r1.score,
                r2.score
            );
        }
    }

    // Ground truth recall check for rebuilt index
    let remaining_dataset: Vec<(DocId, Vec<f32>)> = dataset[to_delete_count..].to_vec();
    let mut gt_query_rng = SimpleRng::new(7777);
    let num_queries = 30;
    let top_k = 10;
    let mut total_hits = 0;

    for _ in 0..num_queries {
        let query = gt_query_rng.next_vector(dim);
        let gt = compute_ground_truth(&remaining_dataset, &query, top_k);
        let gt_set: std::collections::HashSet<DocId> = gt.iter().map(|(id, _)| *id).collect();

        let res = idx1
            .search(&query, top_k)
            .await
            .expect("search rebuilt index");
        for r in &res {
            if gt_set.contains(&r.doc_id) {
                total_hits += 1;
            }
        }
    }

    let recall = (total_hits as f32) / ((num_queries * top_k) as f32);
    println!("Rebuilt Index Recall@10: {recall:.4}");

    assert!(
        recall >= 0.85,
        "Rebuilt index recall must be >= 0.85, got {recall:.4}"
    );
}
