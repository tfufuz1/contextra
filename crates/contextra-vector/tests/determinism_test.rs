use std::sync::Arc;

use contextra_core::{DocId, TxId, VectorIndex};

use contextra_ports::{Rng, SeededRng};
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};

#[tokio::test]
async fn test_hnsw_determinism_explicit_rng_same_topology() {
    let dim = 16;
    let n = 100;

    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(64)
        .ef_search(32)
        .build()
        .unwrap();

    let rng1: Arc<dyn Rng> = Arc::new(SeededRng::new(12345));
    let rng2: Arc<dyn Rng> = Arc::new(SeededRng::new(12345));

    let idx1 = HnswIndex::try_new_with_rng(config.clone(), rng1).unwrap();
    let idx2 = HnswIndex::try_new_with_rng(config.clone(), rng2).unwrap();

    let input_rng = SeededRng::new(999);
    let mut vecs = Vec::with_capacity(n);
    for _ in 0..n {
        let mut v = vec![0.0f32; dim];
        for val in v.iter_mut() {
            *val = input_rng.next_unit_f64() as f32;
        }
        vecs.push(v);
    }

    let tx = TxId::new(1);
    for (i, v) in vecs.iter().enumerate() {
        let doc_id = DocId::new((i as u64 + 1).into());
        idx1.insert(tx, doc_id, v).await.unwrap();
        idx2.insert(tx, doc_id, v).await.unwrap();
    }
    idx1.commit(tx).await.unwrap();
    idx2.commit(tx).await.unwrap();

    let layers1 = idx1.all_doc_ids_and_layers();
    let layers2 = idx2.all_doc_ids_and_layers();

    assert_eq!(layers1.len(), n);
    assert_eq!(layers2.len(), n);
    assert_eq!(
        layers1, layers2,
        "HNSW indices with identical SeededRng must produce bit-identical layer assignments"
    );
}

#[tokio::test]
async fn test_hnsw_determinism_default_try_new_reproducible() {
    let dim = 8;
    let n = 50;

    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(32)
        .build()
        .unwrap();

    let idx1 = HnswIndex::try_new(config.clone()).unwrap();
    let idx2 = HnswIndex::try_new(config.clone()).unwrap();

    let input_rng = SeededRng::new(888);
    let tx = TxId::new(1);
    for i in 0..n {
        let doc_id = DocId::new((i as u64 + 1).into());
        let mut v = vec![0.0f32; dim];
        for val in v.iter_mut() {
            *val = input_rng.next_unit_f64() as f32;
        }
        idx1.insert(tx, doc_id, &v).await.unwrap();
        idx2.insert(tx, doc_id, &v).await.unwrap();
    }
    idx1.commit(tx).await.unwrap();
    idx2.commit(tx).await.unwrap();

    let layers1 = idx1.all_doc_ids_and_layers();
    let layers2 = idx2.all_doc_ids_and_layers();

    assert_eq!(
        layers1, layers2,
        "HNSW default try_new() instances must produce identical graph topology"
    );
}

#[tokio::test]
async fn test_hnsw_determinism_different_seeds_produce_different_layers() {
    let dim = 16;
    let n = 100;

    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(64)
        .build()
        .unwrap();

    let rng1: Arc<dyn Rng> = Arc::new(SeededRng::new(1001));
    let rng2: Arc<dyn Rng> = Arc::new(SeededRng::new(9009));

    let idx1 = HnswIndex::try_new_with_rng(config.clone(), rng1).unwrap();
    let idx2 = HnswIndex::try_new_with_rng(config.clone(), rng2).unwrap();

    let input_rng = SeededRng::new(777);
    let tx = TxId::new(1);
    for i in 0..n {
        let doc_id = DocId::new((i as u64 + 1).into());
        let mut v = vec![0.0f32; dim];
        for val in v.iter_mut() {
            *val = input_rng.next_unit_f64() as f32;
        }
        idx1.insert(tx, doc_id, &v).await.unwrap();
        idx2.insert(tx, doc_id, &v).await.unwrap();
    }
    idx1.commit(tx).await.unwrap();
    idx2.commit(tx).await.unwrap();

    let layers1 = idx1.all_doc_ids_and_layers();
    let layers2 = idx2.all_doc_ids_and_layers();

    assert_eq!(layers1.len(), n);
    assert_eq!(layers2.len(), n);
    assert_ne!(
        layers1, layers2,
        "Different RNG seeds must produce different layer assignments for nodes"
    );
}
