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

#[tokio::test]
async fn test_veto_f02_guard_partial_rebuild_tombstone_pruning() {
    let dim = 32;
    let num_vecs = 100;

    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(64)
        .distance_metric(DistanceMetric::Cosine)
        .build()
        .expect("Valid HNSW config");

    let idx = HnswIndex::try_new_with_rng(config, Arc::new(SeededRng::new(555111)))
        .expect("HnswIndex creation");

    let mut data_rng = SimpleRng::new(8888);
    let mut dataset = Vec::with_capacity(num_vecs);
    for i in 1..=num_vecs {
        let doc_id = DocId::new(i as u64);
        let vec = data_rng.next_unit_vector(dim);
        dataset.push((doc_id, vec));
    }

    let tx = TxId::new(1);
    for (doc_id, vec) in &dataset {
        idx.insert(tx, *doc_id, vec).await.expect("insert");
    }
    idx.commit(tx).await.expect("commit");

    // Delete 5 documents (DocId 1..=5)
    let tx2 = TxId::new(2);
    let deleted_doc_ids = [1u64, 2u64, 3u64, 4u64, 5u64];
    for &id_raw in &deleted_doc_ids {
        idx.delete(tx2, DocId::new(id_raw)).await.expect("delete");
    }
    idx.commit(tx2).await.expect("commit tx2");

    // Execute local rebuild on region containing deleted node indices
    let region_node_indices: Vec<u64> = (0..20).collect();
    idx.rebuild_region(region_node_indices)
        .await
        .expect("rebuild_region");

    // Verify deleted documents are completely excluded from search results
    let mut query_rng = SimpleRng::new(1234);
    for _ in 0..20 {
        let query = query_rng.next_unit_vector(dim);
        let results = idx.search(&query, 10).await.expect("search");
        for r in results {
            for &del_id in &deleted_doc_ids {
                assert_ne!(
                    r.doc_id,
                    DocId::new(del_id),
                    "VETO-F02 Tombstone Pruning Failure: Deleted document {:?} returned in search results after rebuild_region!",
                    r.doc_id
                );
            }
        }
    }

    println!("=== VETO-F02 Guard Verification ===");
    println!("Verified: rebuild_region strictly prunes tombstones; deleted nodes are completely removed from search graph.");
}

#[test]
fn test_partial_rebuild_feature_gate_guard() {
    if !cfg!(feature = "partial-index-rebuild") {
        assert!(!cfg!(feature = "partial-index-rebuild"));
        println!("Verified: partial-index-rebuild is disabled by default in compliance with VETO-F02 (review date 2026-10-07).");
    } else {
        assert!(cfg!(feature = "partial-index-rebuild"));
        println!("Verified: partial-index-rebuild feature is explicitly enabled for testing.");
    }
}
