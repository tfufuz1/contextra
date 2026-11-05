#![allow(
    clippy::all,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    warnings
)]
//! SPEC: T-VEC-DEL-1, T-VEC-DEL-2, D-77, D-94
//! Test suite verifying ghost-free deletion with zero ghost pointers
//! and index loading via BufferIndex (non-mmap).

use contextra_ports::VectorIndex;
use contextra_types::{DistanceMetric, DocId, TxId};
use contextra_vector::{
    hnsw::{DeletionStats, GhostFreeVectorIndex},
    BufferIndex, HnswConfig, HnswIndex,
};

#[tokio::test]
async fn test_r1_ghost_free_deletion_verification() {
    let dimension = 8;
    let config = HnswConfig {
        dimension,
        m: 8,
        ef_construction: 64,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let mut index = HnswIndex::try_new(config).expect("valid config");

    for i in 1..=20u64 {
        let doc_id = DocId::new(i);
        let vec: Vec<f32> = (0..dimension).map(|d| (i * 5 + d as u64) as f32).collect();
        index
            .insert(TxId::new(i), doc_id, &vec)
            .await
            .expect("insert");
        index.commit(TxId::new(i)).await.expect("commit");
    }

    // Delete a node and verify graph repair stats
    let target = DocId::new(10);
    let stats: DeletionStats = index
        .remove_with_graph_repair(target)
        .expect("deletion succeeds");
    assert_eq!(stats.doc_id, target);
    assert!(
        stats.verified_no_ghost_pointers,
        "Zero ghost pointers must be verified"
    );
}

#[tokio::test]
async fn test_r1_buffer_index_persistence_load() {
    let dimension = 4;
    let config = HnswConfig {
        dimension,
        m: 4,
        ef_construction: 16,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let temp_dir = tempfile::tempdir().expect("tempdir");
    let file_path = temp_dir.path().join("index.hnsw");

    let index = HnswIndex::try_new(config.clone()).expect("valid config");
    for i in 1..=10u64 {
        let doc_id = DocId::new(i);
        let vec: Vec<f32> = (0..dimension).map(|d| (i * 2 + d as u64) as f32).collect();
        index
            .insert(TxId::new(i), doc_id, &vec)
            .await
            .expect("insert");
        index.commit(TxId::new(i)).await.expect("commit");
    }

    index.save(&file_path).await.expect("save succeeds");

    // Open via BufferIndex directly
    let buffer_idx = BufferIndex::open(&file_path).expect("buffer index open");
    assert_eq!(buffer_idx.header.dimension(), dimension as u32);
    assert_eq!(buffer_idx.header.node_count(), 10);

    // Load into a fresh index
    let new_index = HnswIndex::try_new(config).expect("new index");
    new_index
        .load_buffer(&file_path)
        .await
        .expect("load buffer succeeds");

    let q = vec![1.0, 2.0, 3.0, 4.0];
    let res = new_index.search(&q, 3).await.expect("search succeeds");
    assert!(
        !res.is_empty(),
        "Search in buffer-loaded index returned results"
    );
}
