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

    fn next_unit_vector(&mut self, dim: usize) -> Vec<f32> {
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
    let mut scores: Vec<(DocId, f32)> = vectors
        .iter()
        .map(|(id, vec)| {
            let dot = query
                .iter()
                .zip(vec.iter())
                .map(|(&a, &b)| a * b)
                .sum::<f32>();
            (*id, dot)
        })
        .collect();

    scores.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    scores.truncate(k);
    scores
}

#[tokio::test]
async fn test_sq8_rerank_recall_measurement() {
    // Hinweis nach Governance-Regel 3: Messung beinhaltet nur Index-/Speicherlatenz, ohne Embedding-Inferenz.
    let dim = 128;
    let num_vecs = 5000;
    let num_queries = 50;
    let top_k = 10;
    let ef_candidates = 64;

    let config_f32 = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(128)
        .ef_search(ef_candidates)
        .quantize(false)
        .distance_metric(DistanceMetric::Cosine)
        .build()
        .expect("Valid f32 config");

    let config_sq8 = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(128)
        .ef_search(ef_candidates)
        .quantize(true)
        .distance_metric(DistanceMetric::Cosine)
        .build()
        .expect("Valid SQ8 config");

    let rng_seed = 123456789u64;
    let idx_f32 = HnswIndex::try_new_with_rng(config_f32, Arc::new(SeededRng::new(rng_seed)))
        .expect("idx_f32 creation");
    let idx_sq8 = HnswIndex::try_new_with_rng(config_sq8, Arc::new(SeededRng::new(rng_seed)))
        .expect("idx_sq8 creation");

    let mut data_rng = SimpleRng::new(4242);
    let mut dataset = Vec::with_capacity(num_vecs);
    for i in 1..=num_vecs {
        // Construct DocId using From<u64> to maintain compatibility across docid-128 and 64-bit modes
        let doc_id = DocId::from(i as u64);
        let vec = data_rng.next_unit_vector(dim);
        dataset.push((doc_id, vec));
    }

    let tx = TxId::new(1);
    for (doc_id, vec) in &dataset {
        idx_f32.insert(tx, *doc_id, vec).await.expect("insert f32");
        idx_sq8.insert(tx, *doc_id, vec).await.expect("insert sq8");
    }

    idx_f32.commit(tx).await.expect("commit f32");
    idx_sq8.commit(tx).await.expect("commit sq8");

    // Fast lookup table doc_id -> vector for exact reranking
    let dataset_map: std::collections::HashMap<DocId, Vec<f32>> = dataset.iter().cloned().collect();

    let mut query_rng = SimpleRng::new(9999);
    let mut hits_f32 = 0;
    let mut hits_sq8_no_rerank = 0;
    let mut hits_sq8_rerank = 0;

    for _ in 0..num_queries {
        let query = query_rng.next_unit_vector(dim);

        // Ground Truth
        let gt = compute_ground_truth(&dataset, &query, top_k);
        let gt_set: std::collections::HashSet<DocId> = gt.iter().map(|(id, _)| *id).collect();

        // 1. Float32 search
        let res_f32 = idx_f32.search(&query, top_k).await.expect("f32 search");
        for r in &res_f32 {
            if gt_set.contains(&r.doc_id) {
                hits_f32 += 1;
            }
        }

        // 2. SQ8 search without rerank
        let res_sq8_no_rerank = idx_sq8.search(&query, top_k).await.expect("sq8 search");
        for r in &res_sq8_no_rerank {
            if gt_set.contains(&r.doc_id) {
                hits_sq8_no_rerank += 1;
            }
        }

        // 3. SQ8 search with exact float32 rerank of top ef_candidates
        let candidates_sq8 = idx_sq8
            .search(&query, ef_candidates)
            .await
            .expect("sq8 candidates");
        let mut reranked: Vec<(DocId, f32)> = candidates_sq8
            .iter()
            .filter_map(|cand| {
                dataset_map.get(&cand.doc_id).map(|orig_vec| {
                    let exact_dot = query
                        .iter()
                        .zip(orig_vec.iter())
                        .map(|(&a, &b)| a * b)
                        .sum::<f32>();
                    (cand.doc_id, exact_dot)
                })
            })
            .collect();

        reranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        reranked.truncate(top_k);

        for (doc_id, _) in &reranked {
            if gt_set.contains(doc_id) {
                hits_sq8_rerank += 1;
            }
        }
    }

    let recall_f32 = (hits_f32 as f32) / ((num_queries * top_k) as f32);
    let recall_sq8_no_rerank = (hits_sq8_no_rerank as f32) / ((num_queries * top_k) as f32);
    let recall_sq8_rerank = (hits_sq8_rerank as f32) / ((num_queries * top_k) as f32);

    println!("=== SQ8 + Rerank Recall Measurement (only index/memory latency, no embedding inference) ===");
    println!("Float32 Recall@10:              {recall_f32:.4}");
    println!("SQ8 (no rerank) Recall@10:      {recall_sq8_no_rerank:.4}");
    println!("SQ8 + Exact Rerank Recall@10:   {recall_sq8_rerank:.4}");

    assert!(
        recall_sq8_rerank >= recall_sq8_no_rerank,
        "SQ8 + Rerank recall ({recall_sq8_rerank:.4}) must be >= SQ8 without rerank ({recall_sq8_no_rerank:.4})"
    );
    assert!(
        recall_sq8_rerank >= 0.95,
        "SQ8 + Rerank recall must be >= 0.95, got {recall_sq8_rerank:.4}"
    );
}
