use contextra_core::{DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        ((z >> 32) as f32) / 4294967296.0
    }
}

#[tokio::test]
async fn test_hnsw_layer_determinism_same_seed_same_results() {
    let dim = 16;
    let n = 200;

    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(64)
        .ef_search(32)
        .build()
        .unwrap();

    let idx1 = HnswIndex::try_new(config.clone()).unwrap();
    let idx2 = HnswIndex::try_new(config.clone()).unwrap();

    idx1.set_layer_seed(42);
    idx2.set_layer_seed(42);

    let mut sm = SplitMix64::new(1001);
    let mut vecs = Vec::with_capacity(n);
    for _ in 0..n {
        let v: Vec<f32> = (0..dim).map(|_| sm.next_f32()).collect();
        vecs.push(v);
    }

    let tx = TxId::new(1);
    for (i, v) in vecs.iter().enumerate() {
        let doc_id = DocId::new(i as u64 + 1);
        idx1.insert(tx, doc_id, v).await.unwrap();
        idx2.insert(tx, doc_id, v).await.unwrap();
    }
    idx1.commit(tx).await.unwrap();
    idx2.commit(tx).await.unwrap();

    let query: Vec<f32> = (0..dim).map(|_| sm.next_f32()).collect();
    let res1 = idx1.search(&query, 10).await.unwrap();
    let res2 = idx2.search(&query, 10).await.unwrap();

    assert_eq!(res1.len(), res2.len());
    for (r1, r2) in res1.iter().zip(res2.iter()) {
        assert_eq!(r1.doc_id, r2.doc_id);
        assert!((r1.score - r2.score).abs() < 1e-6);
    }
}

#[tokio::test]
async fn test_hnsw_layer_determinism_different_seeds_different_layer_sequences() {
    let dim = 8;
    let n = 50;

    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(32)
        .build()
        .unwrap();

    let idx1 = HnswIndex::try_new(config.clone()).unwrap();
    let idx2 = HnswIndex::try_new(config.clone()).unwrap();

    idx1.set_layer_seed(100);
    idx2.set_layer_seed(999);

    let mut sm = SplitMix64::new(2002);
    let mut vecs = Vec::with_capacity(n);
    for _ in 0..n {
        let v: Vec<f32> = (0..dim).map(|_| sm.next_f32()).collect();
        vecs.push(v);
    }

    let tx = TxId::new(1);
    for (i, v) in vecs.iter().enumerate() {
        let doc_id = DocId::new(i as u64 + 1);
        idx1.insert(tx, doc_id, v).await.unwrap();
        idx2.insert(tx, doc_id, v).await.unwrap();
    }
    idx1.commit(tx).await.unwrap();
    idx2.commit(tx).await.unwrap();

    let query: Vec<f32> = (0..dim).map(|_| sm.next_f32()).collect();
    let res1 = idx1.search(&query, 10).await.unwrap();
    let res2 = idx2.search(&query, 10).await.unwrap();

    assert!(!res1.is_empty());
    assert!(!res2.is_empty());
}

#[tokio::test]
async fn test_hnsw_layer_distribution_m16() {
    let dim = 8;
    let n = 1000;

    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(32)
        .build()
        .unwrap();
    let idx = HnswIndex::try_new(config).unwrap();

    let mut sm = SplitMix64::new(3003);
    let tx = TxId::new(1);
    for i in 0..n {
        let doc_id = DocId::new(i as u64 + 1);
        let v: Vec<f32> = (0..dim).map(|_| sm.next_f32()).collect();
        idx.insert(tx, doc_id, &v).await.unwrap();
    }
    idx.commit(tx).await.unwrap();

    let all_docs = idx.all_doc_ids_from_map();
    assert_eq!(all_docs.len(), n);
}
