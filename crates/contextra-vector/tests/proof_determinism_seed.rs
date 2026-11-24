// FILE-CONTEXT
// ZWECK: Beweis für deterministische Grapherzeugung und Suchresultate bei gleichem Seed.

use contextra_ports::VectorIndex;
use contextra_types::{DistanceMetric, DocId, TxId};
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};

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

#[tokio::test]
async fn test_proof_determinism_seed_identical_graph_search() {
    let dim = 16;
    let n = 100;
    let seed = 424242u64;

    let config = HnswConfigBuilder::new(dim)
        .m(8)
        .ef_construction(32)
        .ef_search(16)
        .distance_metric(DistanceMetric::Cosine)
        .build()
        .unwrap();

    let idx1 = HnswIndex::try_new(config.clone()).unwrap();
    let idx2 = HnswIndex::try_new(config.clone()).unwrap();

    idx1.set_layer_seed(seed);
    idx2.set_layer_seed(seed);

    let vecs = generate_vectors(n, dim, 12345);
    let tx = TxId::new(1);
    for (i, v) in vecs.iter().enumerate() {
        let doc_id = DocId::new((i + 1) as u64);
        idx1.insert(tx, doc_id, v).await.unwrap();
        idx2.insert(tx, doc_id, v).await.unwrap();
    }
    idx1.commit(tx).await.unwrap();
    idx2.commit(tx).await.unwrap();

    let queries = generate_vectors(50, dim, 99999);
    for q in &queries {
        let res1 = idx1.search(q, 10).await.unwrap();
        let res2 = idx2.search(q, 10).await.unwrap();

        assert_eq!(res1.len(), res2.len());
        for (r1, r2) in res1.iter().zip(res2.iter()) {
            assert_eq!(r1.doc_id, r2.doc_id, "DocId mismatch in deterministic search");
            assert!(
                (r1.score - r2.score).abs() < 1e-6,
                "Score mismatch: {} vs {}",
                r1.score,
                r2.score
            );
        }
    }
}
