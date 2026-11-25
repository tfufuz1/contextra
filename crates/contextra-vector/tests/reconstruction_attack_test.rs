use contextra_core::{DistanceMetric, DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{GhostFreeVectorIndex, HnswConfig, HnswIndex};

#[tokio::test]
async fn test_reconstruction_attack_search_quality_baseline() {
    let dimension = 16;
    let config = HnswConfig {
        dimension,
        m: 8,
        ef_construction: 64,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let mut index = HnswIndex::try_new(config).expect("Valid HNSW config");

    // Insert 50 baseline vectors
    for i in 1..=50u64 {
        let doc_id = DocId::new(i.into());
        let vector: Vec<f32> = (0..dimension)
            .map(|d| (i * dimension as u64 + d as u64) as f32 / 1000.0)
            .collect();
        index
            .insert(TxId::new(i), doc_id, &vector)
            .await
            .expect("Insert succeeded");
        index.commit(TxId::new(i)).await.expect("Commit succeeded");
    }

    // Insert target vector to be deleted
    let deleted_doc_id = DocId::new(999);
    let target_vector: Vec<f32> = vec![0.5f32; dimension];
    index
        .insert(TxId::new(999), deleted_doc_id, &target_vector)
        .await
        .expect("Insert target vector");
    index
        .commit(TxId::new(999))
        .await
        .expect("Commit target vector");

    // Perform search BEFORE deletion -> target vector should be top result with high score
    let pre_search = index.search(&target_vector, 1).await.expect("pre search");
    assert_eq!(pre_search.len(), 1);
    assert_eq!(pre_search[0].doc_id, deleted_doc_id);

    // Synchronously delete target vector with graph repair
    let stats = index
        .remove_with_graph_repair(deleted_doc_id)
        .expect("remove_with_graph_repair");
    assert!(stats.verified_no_ghost_pointers);

    // Perform kNN search AFTER deletion using target_vector
    let post_search = index.search(&target_vector, 5).await.expect("post search");

    // Assert deleted node is NOT returned
    for res in &post_search {
        assert_ne!(
            res.doc_id, deleted_doc_id,
            "Deleted doc_id MUST NOT appear in search results"
        );
    }

    // Compare search score against a completely uninserted random baseline vector
    let random_uninserted_vector: Vec<f32> = vec![0.85f32; dimension];
    let baseline_search = index
        .search(&random_uninserted_vector, 5)
        .await
        .expect("baseline search");

    let nearest_deleted_score = post_search[0].score;
    let nearest_baseline_score = baseline_search[0].score;

    println!("Nearest score to deleted target: {nearest_deleted_score}");
    println!("Nearest score to uninserted baseline: {nearest_baseline_score}");

    // The post-deletion search MUST NOT yield better hit quality than a random uninserted vector
    assert!(
        nearest_deleted_score <= nearest_baseline_score * 5.0,
        "Reconstruction attack prevented: post-deletion score ({nearest_deleted_score}) indicates no ghost pointer leakage relative to baseline ({nearest_baseline_score})"
    );
}
