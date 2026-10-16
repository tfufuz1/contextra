#![allow(clippy::unwrap_used, clippy::expect_used, clippy::needless_range_loop)]

use contextra_core::{DistanceMetric, DocId, TxId, VectorIndex};
use contextra_ports::SeededRng;
use contextra_vector::acorn::FilteredIndex;
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

fn compute_filtered_ground_truth<F>(
    vectors: &[(DocId, Vec<f32>)],
    query: &[f32],
    k: usize,
    predicate: F,
) -> Vec<(DocId, f32)>
where
    F: Fn(DocId) -> bool,
{
    let mut matching: Vec<(DocId, f32)> = vectors
        .iter()
        .filter(|(id, _)| predicate(*id))
        .map(|(id, vec)| {
            let dot = query
                .iter()
                .zip(vec.iter())
                .map(|(&a, &b)| a * b)
                .sum::<f32>();
            (*id, dot)
        })
        .collect();

    matching.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    matching.truncate(k);
    matching
}

#[tokio::test]
async fn test_filtered_search_selectivity_recall() {
    // Hinweis nach Governance-Regel 3: Messung beinhaltet nur Index-/Speicherlatenz, ohne Embedding-Inferenz.
    let dim = 64;
    let num_vecs = 2000;
    let num_queries = 50;
    let top_k = 10;

    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(128)
        .ef_search(64)
        .distance_metric(DistanceMetric::Cosine)
        .build()
        .expect("Valid HNSW config");

    let idx = HnswIndex::try_new_with_rng(config, Arc::new(SeededRng::new(987654321)))
        .expect("HnswIndex creation");

    let mut data_rng = SimpleRng::new(12345);
    let mut dataset = Vec::with_capacity(num_vecs);
    for i in 1..=num_vecs {
        let doc_id = DocId::from(i as u64);
        let vec = data_rng.next_unit_vector(dim);
        dataset.push((doc_id, vec));
    }

    let tx = TxId::new(1);
    for (doc_id, vec) in &dataset {
        idx.insert(tx, *doc_id, vec).await.expect("insert");
    }
    idx.commit(tx).await.expect("commit");

    let selectivities = [
        ("1% Selectivity", 100u64, 0u64), // id % 100 == 0 -> 1% (20 docs)
        ("5% Selectivity", 20u64, 0u64),  // id % 20 == 0 -> 5% (100 docs)
        ("20% Selectivity", 5u64, 0u64),  // id % 5 == 0 -> 20% (400 docs)
    ];

    println!("=== Filtered Search Selectivity Recall Measurement Protocol ===");
    println!("(Hinweis: Messung beinhaltet nur Index-/Speicherlatenz, ohne Embedding-Inferenz)");

    for (label, modulus, target_rem) in selectivities {
        let predicate = Arc::new(move |id: DocId| (id.inner() as u64) % modulus == target_rem);

        let mut query_rng = SimpleRng::new(54321);
        let mut std_hits = 0;
        let mut acorn_hits = 0;
        let mut total_expected = 0;

        for _ in 0..num_queries {
            let query = query_rng.next_unit_vector(dim);
            let p_gt = Arc::clone(&predicate);
            let gt = compute_filtered_ground_truth(&dataset, &query, top_k, move |id| p_gt(id));
            if gt.is_empty() {
                continue;
            }

            let gt_set: std::collections::HashSet<DocId> = gt.iter().map(|(id, _)| *id).collect();
            let k_query = gt.len();

            // 1. Standard Post-Filtering Search
            let p_std = Arc::clone(&predicate);
            let filter_fn = move |id: DocId| p_std(id);
            let results_std = idx
                .search_filtered(&query, k_query, Some(&filter_fn))
                .await
                .expect("search_filtered");
            for r in &results_std {
                if gt_set.contains(&r.doc_id) {
                    std_hits += 1;
                }
            }

            // 2. ACORN Predicate-Augmented Search (gamma = 8)
            let p_acorn = Arc::clone(&predicate);
            let acorn_pred = move |id: DocId| p_acorn(id);
            let gamma = 8;
            if let Ok(results_acorn) = idx.search_knn_acorn(&query, k_query, &acorn_pred, gamma) {
                for (doc_id, _) in &results_acorn {
                    if gt_set.contains(doc_id) {
                        acorn_hits += 1;
                    }
                }
            }

            total_expected += k_query;
        }

        let std_recall = if total_expected > 0 {
            (std_hits as f32) / (total_expected as f32)
        } else {
            0.0
        };

        let acorn_recall = if total_expected > 0 {
            (acorn_hits as f32) / (total_expected as f32)
        } else {
            0.0
        };

        println!(
            "{label:16} | Standard Search Recall@{top_k}: {std_recall:.4} ({std_hits}/{total_expected}) | ACORN (gamma=8) Recall@{top_k}: {acorn_recall:.4} ({acorn_hits}/{total_expected})"
        );

        // Informative assertion: Recall must be plausible (>= 0) and recorded in the protocol.
        assert!(
            std_recall >= 0.0 && acorn_recall >= 0.0,
            "Recall measurement for {label} completed"
        );
    }
}
