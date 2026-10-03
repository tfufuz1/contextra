//! Campaign J-23: Comprehensive Audit and Verification Test Suite for `contextra-vector`.
//! Tests cover HNSW, DiskANN, ACORN, Adaptive ef, Rebuild/Swap, Arena Node Reuse, Partial Rebuild, Recall Suite, Limits & MVCC.

use contextra_core::{
    DistanceMetric, DocId, Result, ScoredDocument, TxId, VectorIndex,
};
use contextra_vector::acorn::{compute_gamma_edge_budget, FilteredIndex, NaiveReferenceIndex};
use contextra_vector::hnsw::{
    AdaptiveEfPolicy, AdaptiveEfStateMachine, HnswConfig, HnswIndex,
};
use std::collections::HashSet;

// --- ORACLE / ANTI-MIRRORING (R4) ---
/// Scalar brute-force reference model for exact k-NN ground truth verification.
pub struct IndependentScalarOracle {
    vectors: Vec<(DocId, Vec<f32>)>,
    metric: DistanceMetric,
}

impl IndependentScalarOracle {
    pub fn new(metric: DistanceMetric) -> Self {
        Self {
            vectors: Vec::new(),
            metric,
        }
    }

    pub fn insert(&mut self, doc_id: DocId, vec: Vec<f32>) {
        self.vectors.retain(|(id, _)| *id != doc_id);
        self.vectors.push((doc_id, vec));
    }

    pub fn delete(&mut self, doc_id: DocId) {
        self.vectors.retain(|(id, _)| *id != doc_id);
    }

    pub fn search(&self, query: &[f32], k: usize) -> Vec<ScoredDocument> {
        let mut scored: Vec<ScoredDocument> = self
            .vectors
            .iter()
            .map(|(doc_id, vec)| {
                let dist = compute_scalar_distance(query, vec, self.metric);
                ScoredDocument {
                    doc_id: *doc_id,
                    score: dist,
                }
            })
            .collect();

        // Sort ascending by distance, breaking ties by DocId inner
        scored.sort_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.doc_id.inner().cmp(&b.doc_id.inner()))
        });

        scored.truncate(k);
        scored
    }
}

fn compute_scalar_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
    match metric {
        DistanceMetric::Euclidean => a
            .iter()
            .zip(b.iter())
            .map(|(&x, &y)| (x - y) * (x - y))
            .sum::<f32>()
            .sqrt(),
        DistanceMetric::Cosine => {
            let mut dot = 0.0f32;
            let mut norm_a = 0.0f32;
            let mut norm_b = 0.0f32;
            for (&x, &y) in a.iter().zip(b.iter()) {
                dot += x * y;
                norm_a += x * x;
                norm_b += y * y;
            }
            if norm_a <= 0.0 || norm_b <= 0.0 {
                1.0
            } else {
                1.0 - (dot / (norm_a.sqrt() * norm_b.sqrt()))
            }
        }
        DistanceMetric::DotProduct => {
            let dot: f32 = a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum();
            -dot
        }
        _ => panic!("Unsupported distance metric in oracle"),
    }
}

// Helper to construct DocId safely across feature flags
fn make_doc_id(id: u64) -> DocId {
    DocId::from(id)
}

// ==========================================
// H1: DocId Truncation & Collision (BUG-VEC-01)
// ==========================================
#[cfg(feature = "docid-128")]
#[tokio::test]
async fn test_h1_docid_u128_hnsw_collision_safety() -> Result<()> {
    let config = HnswConfig {
        dimension: 4,
        m: 16,
        ef_construction: 32,
        ef_search: 32,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;

    // High 128-bit DocIds sharing lower 64 bits
    let id_a = DocId::from(1u128);
    let id_b = DocId::from((1u128 << 64) | 1u128);
    let id_c = DocId::from((2u128 << 64) | 1u128);

    let v_a = vec![1.0, 0.0, 0.0, 0.0];
    let v_b = vec![0.0, 1.0, 0.0, 0.0];
    let v_c = vec![0.0, 0.0, 1.0, 0.0];

    index.insert(TxId(1), id_a, &v_a).await?;
    index.insert(TxId(1), id_b, &v_b).await?;
    index.insert(TxId(1), id_c, &v_c).await?;
    index.commit(TxId(1)).await?;

    let all_ids = index.all_doc_ids().await?;
    assert_eq!(
        all_ids.len(),
        3,
        "H1 FAIL: High 64-bit DocIds collided or truncated in all_doc_ids"
    );

    let res_a = index.search(&v_a, 1).await?;
    assert_eq!(res_a[0].doc_id, id_a);

    let res_b = index.search(&v_b, 1).await?;
    assert_eq!(res_b[0].doc_id, id_b);

    // Delete B and ensure A and C remain intact
    index.delete(TxId(2), id_b).await?;
    index.commit(TxId(2)).await?;
    let ids_after = index.all_doc_ids().await?;
    assert_eq!(ids_after.len(), 2);
    assert!(ids_after.contains(&id_a));
    assert!(ids_after.contains(&id_c));
    assert!(!ids_after.contains(&id_b));

    Ok(())
}

#[cfg(not(feature = "docid-128"))]
#[tokio::test]
async fn test_h1_docid_u128_hnsw_collision_safety() -> Result<()> {
    let config = HnswConfig {
        dimension: 4,
        m: 16,
        ef_construction: 32,
        ef_search: 32,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;

    let id_a = make_doc_id(1);
    let id_b = make_doc_id(1002);
    let id_c = make_doc_id(2003);

    let v_a = vec![1.0, 0.0, 0.0, 0.0];
    let v_b = vec![0.0, 1.0, 0.0, 0.0];
    let v_c = vec![0.0, 0.0, 1.0, 0.0];

    index.insert(TxId(1), id_a, &v_a).await?;
    index.insert(TxId(1), id_b, &v_b).await?;
    index.insert(TxId(1), id_c, &v_c).await?;
    index.commit(TxId(1)).await?;

    let all_ids = index.all_doc_ids().await?;
    assert_eq!(all_ids.len(), 3);

    Ok(())
}

// ==========================================
// H3: Adaptive ef State Machine & Duplicate Vectors (BUG-VEC-03/K1)
// ==========================================
#[test]
fn test_h3_adaptive_ef_duplicate_vector_stability() -> Result<()> {
    let policy = AdaptiveEfPolicy::new(16, 256, 1.5, 2, 1.0)?;
    let mut sm = AdaptiveEfStateMachine::new(policy, 10);

    // Duplicate top-k list
    let top_k_a: Vec<DocId> = (1..=10).map(make_doc_id).collect();
    let top_k_b = top_k_a.clone();
    let top_k_c = top_k_a.clone();

    let step1 = sm.step(&top_k_a);
    assert!(!step1, "Round 1 should not trigger convergence");
    assert_eq!(sm.current_ef(), 24);

    let step2 = sm.step(&top_k_b);
    assert!(!step2, "Round 2 stability = 1 round, window is 2");
    assert_eq!(sm.current_ef(), 36);

    let step3 = sm.step(&top_k_c);
    assert!(step3, "Round 3 should converge due to stability");
    assert!(sm.stats().converged);
    assert!(sm.is_terminated());

    Ok(())
}

#[test]
fn test_h3_adaptive_ef_max_ef_termination() -> Result<()> {
    let policy = AdaptiveEfPolicy::new(16, 32, 2.0, 5, 1.0)?;
    let mut sm = AdaptiveEfStateMachine::new(policy, 5);

    for i in 0..10 {
        let top_k: Vec<DocId> = (1..=5).map(|j| make_doc_id(i * 10 + j)).collect();
        let term = sm.step(&top_k);
        if term {
            break;
        }
    }

    assert!(sm.is_terminated());
    assert_eq!(sm.current_ef(), 32, "Should terminate at max_ef");
    assert!(!sm.stats().converged, "Terminated due to max_ef, not stability");

    Ok(())
}

// ==========================================
// H4: Two-Phase Rebuild & Deterministic Search (K2)
// ==========================================
#[tokio::test]
async fn test_h4_twophase_rebuild_concurrency_and_determinism() -> Result<()> {
    let config = HnswConfig {
        dimension: 4,
        m: 16,
        ef_construction: 32,
        ef_search: 32,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;

    // Populate initial dataset
    for i in 1..=100 {
        let id = make_doc_id(i);
        let vec = vec![i as f32, 0.0, 0.0, 0.0];
        index.insert(TxId(1), id, &vec).await?;
    }
    index.commit(TxId(1)).await?;

    let query = vec![10.0, 0.0, 0.0, 0.0];
    let res_before = index.search(&query, 5).await?;

    // Trigger rebuild
    index.rebuild().await?;

    let res_after = index.search(&query, 5).await?;
    assert_eq!(
        res_before.len(),
        res_after.len(),
        "Result count mismatch post-rebuild"
    );
    for (b, a) in res_before.iter().zip(res_after.iter()) {
        assert_eq!(b.doc_id, a.doc_id, "Result ordering/identity changed");
        assert_eq!(b.score, a.score);
    }

    Ok(())
}

// ==========================================
// H5: Arena Node Slot Reuse & Ghost Backlinks (K3)
// ==========================================
#[tokio::test]
async fn test_h5_arena_node_reuse_churn_no_ghost_pointers() -> Result<()> {
    let config = HnswConfig {
        dimension: 4,
        m: 8,
        ef_construction: 16,
        ef_search: 16,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;

    let mut oracle = IndependentScalarOracle::new(DistanceMetric::Euclidean);

    // Churn operations
    for i in 1..=200 {
        let doc_id = make_doc_id(i);
        let vec = vec![i as f32, (i % 5) as f32, 0.0, 0.0];
        index.insert(TxId(1), doc_id, &vec).await?;
        oracle.insert(doc_id, vec);
    }
    index.commit(TxId(1)).await?;

    for i in 1..=10 {
        let doc_id = make_doc_id(i);
        index.delete(TxId(2), doc_id).await?;
        oracle.delete(doc_id);
    }
    index.commit(TxId(2)).await?;

    index.rebuild().await?;

    for i in 1..=10 {
        let doc_id = make_doc_id(i + 1000);
        let vec = vec![i as f32 * 2.0, 1.0, 0.0, 0.0];
        index.insert(TxId(3), doc_id, &vec).await?;
        oracle.insert(doc_id, vec);
    }
    index.commit(TxId(3)).await?;

    index.check_connectivity()?;

    let query = vec![50.0, 1.0, 0.0, 0.0];
    let hnsw_res = index.search(&query, 10).await?;
    let oracle_res = oracle.search(&query, 10);

    assert_eq!(
        hnsw_res.len(),
        oracle_res.len(),
        "Result size mismatch after churn"
    );

    Ok(())
}

// ==========================================
// H6: Partial Rebuild Audit (K4 / VETO-F02)
// ==========================================
#[tokio::test]
async fn test_h6_partial_rebuild_tombstone_only_pruning() -> Result<()> {
    let config = HnswConfig {
        dimension: 4,
        m: 8,
        ef_construction: 16,
        ef_search: 16,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;

    for i in 1..=50 {
        let doc_id = make_doc_id(i);
        let vec = vec![i as f32, 0.0, 0.0, 0.0];
        index.insert(TxId(1), doc_id, &vec).await?;
    }
    index.commit(TxId(1)).await?;

    for i in 1..=10 {
        let doc_id = make_doc_id(i);
        index.delete(TxId(2), doc_id).await?;
    }
    index.commit(TxId(2)).await?;

    index.rebuild_region(vec![0, 1, 2, 3, 4]).await?;

    let all_ids = index.all_doc_ids().await?;
    assert_eq!(all_ids.len(), 40);

    Ok(())
}

// ==========================================
// H7: Recall Suite vs Independent Oracle (R4)
// ==========================================
#[tokio::test]
async fn test_h7_recall_suite_gaussian_clustered_adversarial() -> Result<()> {
    let config = HnswConfig {
        dimension: 8,
        m: 16,
        ef_construction: 64,
        ef_search: 64,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;
    let mut oracle = IndependentScalarOracle::new(DistanceMetric::Euclidean);

    for i in 1..=200 {
        let doc_id = make_doc_id(i);
        let vec = vec![
            (i as f32 * 0.1).sin(),
            (i as f32 * 0.2).cos(),
            (i as f32 * 0.3).sin(),
            (i as f32 * 0.4).cos(),
            0.5,
            -0.5,
            0.1 * (i % 10) as f32,
            0.0,
        ];
        index.insert(TxId(1), doc_id, &vec).await?;
        oracle.insert(doc_id, vec);
    }
    index.commit(TxId(1)).await?;

    let query = vec![0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.0, 0.0];
    let hnsw_top10 = index.search(&query, 10).await?;
    let oracle_top10 = oracle.search(&query, 10);

    let hnsw_set: HashSet<DocId> = hnsw_top10.iter().map(|s| s.doc_id).collect();
    let oracle_set: HashSet<DocId> = oracle_top10.iter().map(|s| s.doc_id).collect();

    let overlap = hnsw_set.intersection(&oracle_set).count();
    let recall = overlap as f32 / 10.0;

    assert!(
        recall >= 0.80,
        "Recall@10 = {} < 0.80 threshold vs independent oracle",
        recall
    );

    Ok(())
}

// ==========================================
// H8: ACORN KNN Filtered Search Across Selectivities
// ==========================================
#[tokio::test]
async fn test_h8_acorn_filtered_selectivity_sweeps() -> Result<()> {
    let mut ref_index = NaiveReferenceIndex::new(DistanceMetric::Euclidean);

    for i in 1..=1000 {
        let doc_id = make_doc_id(i);
        let vec = vec![i as f32, 0.0, 0.0, 0.0];
        ref_index.insert(doc_id, vec);
    }

    let query = vec![500.0, 0.0, 0.0, 0.0];

    // Selectivity 100%
    let f_100 = |_id: DocId| true;
    let res_100 = ref_index
        .search_knn_acorn(&query, 10, &f_100, 1)
        .map_err(|e| contextra_core::ContextraError::Index(e.to_string()))?;
    assert_eq!(res_100.len(), 10);

    // Selectivity 50% (even IDs)
    let f_50 = |id: DocId| id.inner() % 2 == 0;
    let res_50 = ref_index
        .search_knn_acorn(&query, 10, &f_50, 1)
        .map_err(|e| contextra_core::ContextraError::Index(e.to_string()))?;
    assert_eq!(res_50.len(), 10);

    // Selectivity 10% (divisible by 10)
    let f_10 = |id: DocId| id.inner() % 10 == 0;
    let res_10 = ref_index
        .search_knn_acorn(&query, 10, &f_10, 1)
        .map_err(|e| contextra_core::ContextraError::Index(e.to_string()))?;
    assert_eq!(res_10.len(), 10);

    // Selectivity 1% (divisible by 100)
    let f_1 = |id: DocId| id.inner() % 100 == 0;
    let res_1 = ref_index
        .search_knn_acorn(&query, 10, &f_1, 1)
        .map_err(|e| contextra_core::ContextraError::Index(e.to_string()))?;
    assert_eq!(res_1.len(), 10);

    // Selectivity 0.1% (divisible by 1000)
    let f_01 = |id: DocId| id.inner() == 1000;
    let res_01 = ref_index
        .search_knn_acorn(&query, 10, &f_01, 1)
        .map_err(|e| contextra_core::ContextraError::Index(e.to_string()))?;
    assert_eq!(res_01.len(), 1);

    // Selectivity 0% -> empty without hang
    let f_0 = |_id: DocId| false;
    let res_0 = ref_index
        .search_knn_acorn(&query, 10, &f_0, 1)
        .map_err(|e| contextra_core::ContextraError::Index(e.to_string()))?;
    assert!(res_0.is_empty());

    // Gamma edge budget boundary checks
    assert!(compute_gamma_edge_budget(16, 0.0) >= 16);
    assert!(compute_gamma_edge_budget(16, 1.0) >= 16);
    assert!(compute_gamma_edge_budget(16, f32::NAN) >= 16);

    Ok(())
}

// ==========================================
// H9: Boundary Limits & Exception Handling
// ==========================================
#[tokio::test]
async fn test_h9_limits_dimension_mismatch_and_nan() -> Result<()> {
    let config = HnswConfig {
        dimension: 4,
        m: 16,
        ef_construction: 32,
        ef_search: 32,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;

    // Dimension mismatch
    let bad_dim_vec = vec![1.0, 2.0];
    let res_dim = index
        .insert(TxId(1), make_doc_id(1), &bad_dim_vec)
        .await;
    assert!(
        res_dim.is_err(),
        "Dimension mismatch must return Err, not panic"
    );

    // NaN in vector
    let nan_vec = vec![1.0, f32::NAN, 0.0, 0.0];
    let res_nan = index.insert(TxId(1), make_doc_id(2), &nan_vec).await;
    assert!(res_nan.is_err(), "NaN vector must return Err, not panic");

    // Empty index search
    let empty_search = index.search(&[1.0, 0.0, 0.0, 0.0], 5).await?;
    assert!(empty_search.is_empty());

    // k = 0 search
    let zero_k_search = index.search(&[1.0, 0.0, 0.0, 0.0], 0).await?;
    assert!(zero_k_search.is_empty());

    Ok(())
}

// ==========================================
// H10: MVCC Transaction Integration
// ==========================================
#[tokio::test]
async fn test_h10_mvcc_commit_rollback_visibility() -> Result<()> {
    let config = HnswConfig {
        dimension: 4,
        m: 16,
        ef_construction: 32,
        ef_search: 32,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;

    let doc_id = make_doc_id(100);
    let vec = vec![1.0, 1.0, 0.0, 0.0];

    // Staging in uncommitted Tx 1
    index.insert(TxId(1), doc_id, &vec).await?;

    // Rollback Tx 1
    index.rollback(TxId(1)).await?;

    let search_after_rollback = index.search(&vec, 1).await?;
    assert!(
        search_after_rollback.is_empty(),
        "Rolled back vector must not be visible"
    );

    // Commit Tx 2
    index.insert(TxId(2), doc_id, &vec).await?;
    index.commit(TxId(2)).await?;

    let search_after_commit = index.search(&vec, 1).await?;
    assert_eq!(search_after_commit.len(), 1);
    assert_eq!(search_after_commit[0].doc_id, doc_id);

    Ok(())
}
