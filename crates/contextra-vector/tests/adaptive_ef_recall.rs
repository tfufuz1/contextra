// FILE-CONTEXT
// ZWECK: Integrationstests für adaptives ef_search (Ada-ef) Recall, Determinismus, Performance- und Lösch-Invarianten.
// INVARIANTEN: Zero-Panic in Produktion; Lints in Tests freigegeben.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{DistanceMetric, DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{AdaptiveEfPolicy, HnswConfig, HnswIndex};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[tokio::test]
async fn test_adaptive_ef_easy_queries_early_exit_and_recall() {
    let dim = 64;
    let num_vectors = 500;
    let num_queries = 20;

    let config = HnswConfig {
        dimension: dim,
        max_elements: 1000,
        m: 16,
        ef_construction: 200,
        ef_search: 64,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config).unwrap();
    let mut rng = StdRng::seed_from_u64(42);

    let mut data = Vec::with_capacity(num_vectors);
    for _ in 0..num_vectors {
        let mut v: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        for x in v.iter_mut() {
            *x /= norm;
        }
        data.push(v);
    }

    let tx = TxId::new(1);
    for (i, v) in data.iter().enumerate() {
        index.insert(tx, DocId::new((i as u64).into()), v).await.unwrap();
    }
    index.commit(tx).await.unwrap();

    let policy = AdaptiveEfPolicy::new(16, 128, 1.5, 2, 1.0).unwrap();

    let mut total_adaptive_hits = 0;
    let mut total_static_hits = 0;
    let mut total_rounds = 0;

    for _ in 0..num_queries {
        let query_idx = rng.gen_range(0..num_vectors);
        let query = &data[query_idx];

        let static_res = index.search(query, 10).await.unwrap();
        let (adaptive_res, stats) = index
            .search_adaptive_with_stats(query, 10, &policy, None)
            .await
            .unwrap();

        let static_set: std::collections::HashSet<_> =
            static_res.iter().map(|r| r.doc_id).collect();

        for r in &adaptive_res {
            if static_set.contains(&r.doc_id) {
                total_adaptive_hits += 1;
            }
        }
        total_static_hits += static_res.len();
        total_rounds += stats.rounds;

        assert!(
            stats.final_ef <= 64,
            "Easy query should terminate before reaching static ef=64 (got final_ef={})",
            stats.final_ef
        );
    }

    let recall = total_adaptive_hits as f64 / total_static_hits as f64;
    let avg_rounds = total_rounds as f64 / num_queries as f64;

    println!(
        "Easy query Recall@10 vs static ef=64: {:.4}, avg rounds: {:.2}",
        recall, avg_rounds
    );

    assert!(
        recall >= 0.95,
        "Adaptive ef recall on easy queries too low: {:.4}",
        recall
    );
    assert!(
        avg_rounds <= 4.0,
        "Easy queries should exit in few rounds (got avg {:.2})",
        avg_rounds
    );
}

#[tokio::test]
async fn test_adaptive_ef_difficult_queries_grows_to_max_ef() {
    let dim = 32;
    let num_clusters = 5;
    let cluster_size = 100;
    let total_vectors = num_clusters * cluster_size;

    let config = HnswConfig {
        dimension: dim,
        max_elements: 1000,
        m: 8,
        ef_construction: 64,
        ef_search: 128,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config).unwrap();
    let mut rng = StdRng::seed_from_u64(12345);

    let mut data = Vec::with_capacity(total_vectors);

    for c in 0..num_clusters {
        let center: Vec<f32> = (0..dim).map(|_| (c as f32 * 0.1)).collect();
        for _ in 0..cluster_size {
            let mut v: Vec<f32> = center
                .iter()
                .map(|&x| x + rng.gen_range(-0.5..0.5))
                .collect();
            let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for x in v.iter_mut() {
                    *x /= norm;
                }
            }
            data.push(v);
        }
    }

    let tx = TxId::new(1);
    for (i, v) in data.iter().enumerate() {
        index.insert(tx, DocId::new(i as u64), v).await.unwrap();
    }
    index.commit(tx).await.unwrap();

    let policy = AdaptiveEfPolicy::new(16, 128, 1.5, 3, 1.0).unwrap();

    let query: Vec<f32> = (0..dim).map(|_| rng.gen_range(-0.5..0.5)).collect();

    let (adaptive_res, stats) = index
        .search_adaptive_with_stats(&query, 10, &policy, None)
        .await
        .unwrap();

    let static_res = index.search(&query, 10).await.unwrap();

    let ground_truth: std::collections::HashSet<_> = static_res.iter().map(|r| r.doc_id).collect();
    let mut hits = 0;
    for r in &adaptive_res {
        if ground_truth.contains(&r.doc_id) {
            hits += 1;
        }
    }

    let recall = hits as f64 / 10.0;

    println!(
        "Difficult query Recall@10: {:.4}, final_ef: {}, rounds: {}",
        recall, stats.final_ef, stats.rounds
    );

    assert!(
        stats.final_ef >= 64,
        "Difficult query should expand ef significantly (got {})",
        stats.final_ef
    );
    assert!(
        recall >= 0.80,
        "Difficult query recall should be high (got {:.4})",
        recall
    );
}

#[tokio::test]
async fn test_adaptive_ef_determinism() {
    let dim = 32;
    let num_vectors = 200;

    let config = HnswConfig {
        dimension: dim,
        max_elements: 500,
        m: 16,
        ef_construction: 100,
        ef_search: 64,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config).unwrap();
    let mut rng = StdRng::seed_from_u64(9999);

    let tx = TxId::new(1);
    for i in 0..num_vectors {
        let mut v: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        for x in v.iter_mut() {
            *x /= norm;
        }
        index.insert(tx, DocId::new((i as u64).into()), &v).await.unwrap();
    }
    index.commit(tx).await.unwrap();

    let policy = AdaptiveEfPolicy::new(16, 128, 1.5, 2, 1.0).unwrap();
    let query: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();

    let (res_1, stats_1) = index
        .search_adaptive_with_stats(&query, 10, &policy, None)
        .await
        .unwrap();

    let (res_2, stats_2) = index
        .search_adaptive_with_stats(&query, 10, &policy, None)
        .await
        .unwrap();

    assert_eq!(
        stats_1, stats_2,
        "Execution stats must be identical across runs"
    );
    assert_eq!(res_1.len(), res_2.len(), "Result lengths must match");

    for (r1, r2) in res_1.iter().zip(res_2.iter()) {
        assert_eq!(r1.doc_id, r2.doc_id, "Doc IDs must match");
        assert_eq!(r1.score, r2.score, "Scores must match");
    }
}

#[tokio::test]
async fn test_adaptive_ef_invalid_policy_errors() {
    assert!(AdaptiveEfPolicy::new(0, 64, 1.5, 2, 1.0).is_err());
    assert!(AdaptiveEfPolicy::new(64, 16, 1.5, 2, 1.0).is_err());
    assert!(AdaptiveEfPolicy::new(16, 64, 1.0, 2, 1.0).is_err());
    assert!(AdaptiveEfPolicy::new(16, 64, 0.5, 2, 1.0).is_err());
    assert!(AdaptiveEfPolicy::new(16, 64, f32::NAN, 2, 1.0).is_err());
    assert!(AdaptiveEfPolicy::new(16, 64, 1.5, 0, 1.0).is_err());
    assert!(AdaptiveEfPolicy::new(16, 64, 1.5, 2, -0.1).is_err());
    assert!(AdaptiveEfPolicy::new(16, 64, 1.5, 2, 1.1).is_err());

    let invalid_policy = AdaptiveEfPolicy {
        min_ef: 0,
        max_ef: 64,
        growth_factor: 1.5,
        convergence_window: 2,
        stability_threshold: 1.0,
    };

    let config = HnswConfig {
        dimension: 16,
        max_elements: 100,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config).unwrap();
    let query = vec![0.1f32; 16];

    assert!(index
        .search_adaptive(&query, 10, &invalid_policy)
        .await
        .is_err());
}

#[tokio::test]
async fn test_adaptive_ef_deleted_nodes_never_returned() {
    let dim = 16;
    let num_vectors = 100;

    let config = HnswConfig {
        dimension: dim,
        max_elements: 200,
        m: 16,
        ef_construction: 100,
        ef_search: 64,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config).unwrap();
    let mut rng = StdRng::seed_from_u64(777);

    let tx1 = TxId::new(1);
    for i in 0..num_vectors {
        let v: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
        index.insert(tx1, DocId::new((i as u64).into()), &v).await.unwrap();
    }
    index.commit(tx1).await.unwrap();

    let deleted_ids: std::collections::HashSet<DocId> =
        (0..30).map(|i| DocId::new((i as u64).into())).collect();

    let tx2 = TxId::new(2);
    for &doc_id in &deleted_ids {
        index.delete(tx2, doc_id).await.unwrap();
    }
    index.commit(tx2).await.unwrap();

    let policy = AdaptiveEfPolicy::new(16, 128, 1.5, 2, 1.0).unwrap();

    for _ in 0..10 {
        let query: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let results = index.search_adaptive(&query, 10, &policy).await.unwrap();

        for r in &results {
            assert!(
                !deleted_ids.contains(&r.doc_id),
                "Deleted DocId {} was returned in search_adaptive results",
                r.doc_id
            );
        }
    }
}
