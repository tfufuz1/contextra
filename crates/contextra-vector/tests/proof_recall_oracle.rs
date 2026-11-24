// FILE-CONTEXT
// ZWECK: Recall@10 Beweis gegen Brute-Force-Orakel für deterministische Korpora (als Konstante eingefroren, Toleranz 0).

use contextra_ports::VectorIndex;
use contextra_types::{DistanceMetric, DocId, TxId};
use contextra_vector::distance::compute_distance;
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};
use std::collections::HashSet;

fn generate_vectors(n: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut state = seed;
    let mut vecs = Vec::with_capacity(n);
    for _ in 0..n {
        let mut v = Vec::with_capacity(dim);
        for _ in 0..dim {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = ((state >> 33) as f32) / ((1u32 << 31) as f32) - 0.5;
            v.push(val);
        }
        vecs.push(v);
    }
    vecs
}

fn brute_force_knn(
    query: &[f32],
    corpus: &[Vec<f32>],
    k: usize,
    metric: DistanceMetric,
) -> Vec<usize> {
    let mut dists: Vec<(usize, f32)> = corpus
        .iter()
        .enumerate()
        .map(|(i, v)| (i, compute_distance(query, v, metric).unwrap()))
        .collect();

    dists.sort_by(|a, b| {
        a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0))
    });
    dists.truncate(k);
    dists.into_iter().map(|(i, _)| i).collect()
}

const EXPECTED_HITS_COSINE: usize = 100;
const EXPECTED_HITS_EUCLIDEAN: usize = 100;

#[tokio::test]
async fn test_proof_recall_oracle_cosine() {
    let dim = 16;
    let n = 200;
    let num_queries = 10;
    let k = 10;

    let corpus = generate_vectors(n, dim, 1111);
    let queries = generate_vectors(num_queries, dim, 2222);

    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(64)
        .ef_search(32)
        .distance_metric(DistanceMetric::Cosine)
        .build()
        .unwrap();

    let idx = HnswIndex::try_new(config).unwrap();
    idx.set_layer_seed(42);

    let tx = TxId::new(1);
    for (i, v) in corpus.iter().enumerate() {
        idx.insert(tx, DocId::new((i + 1) as u64), v).await.unwrap();
    }
    idx.commit(tx).await.unwrap();

    let mut total_hits = 0;
    for q in &queries {
        let exact_ids = brute_force_knn(q, &corpus, k, DistanceMetric::Cosine);
        let exact_set: HashSet<u64> = exact_ids.into_iter().map(|i| (i + 1) as u64).collect();

        let results = idx.search(q, k).await.unwrap();
        for r in results {
            if exact_set.contains(&(r.doc_id.inner() as u64)) {
                total_hits += 1;
            }
        }
    }

    assert_eq!(
        total_hits, EXPECTED_HITS_COSINE,
        "Recall mismatch for Cosine: got {total_hits}, expected {EXPECTED_HITS_COSINE}"
    );
}

#[tokio::test]
async fn test_proof_recall_oracle_euclidean() {
    let dim = 16;
    let n = 200;
    let num_queries = 10;
    let k = 10;

    let corpus = generate_vectors(n, dim, 3333);
    let queries = generate_vectors(num_queries, dim, 4444);

    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(64)
        .ef_search(32)
        .distance_metric(DistanceMetric::Euclidean)
        .build()
        .unwrap();

    let idx = HnswIndex::try_new(config).unwrap();
    idx.set_layer_seed(42);

    let tx = TxId::new(1);
    for (i, v) in corpus.iter().enumerate() {
        idx.insert(tx, DocId::new((i + 1) as u64), v).await.unwrap();
    }
    idx.commit(tx).await.unwrap();

    let mut total_hits = 0;
    for q in &queries {
        let exact_ids = brute_force_knn(q, &corpus, k, DistanceMetric::Euclidean);
        let exact_set: HashSet<u64> = exact_ids.into_iter().map(|i| (i + 1) as u64).collect();

        let results = idx.search(q, k).await.unwrap();
        for r in results {
            if exact_set.contains(&(r.doc_id.inner() as u64)) {
                total_hits += 1;
            }
        }
    }

    assert_eq!(
        total_hits, EXPECTED_HITS_EUCLIDEAN,
        "Recall mismatch for Euclidean: got {total_hits}, expected {EXPECTED_HITS_EUCLIDEAN}"
    );
}
