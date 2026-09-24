use super::fixtures::*;

#[tokio::test]
async fn hybrid_search_caps_k_at_max_search_k() {
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // unwrap
    let lsm_config = contextra_store::LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap()); // unwrap
    let hnsw_config = contextra_vector::HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = Arc::new(HnswIndex::try_new(hnsw_config).unwrap()); // unwrap
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    let res = col
        .hybrid_search("test", &[0.1, 0.2, 0.3, 0.4], 100_000, None)
        .await
        .unwrap(); // unwrap

    assert!(
        res.len() <= contextra_core::MAX_SEARCH_K,
        "Results length {} should be <= MAX_SEARCH_K ({})",
        res.len(),
        contextra_core::MAX_SEARCH_K
    );
}


#[tokio::test]
async fn test_hybrid_search_k_clamping_boundaries() {
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // unwrap
    let lsm_config = contextra_store::LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap()); // unwrap
    let hnsw_config = contextra_vector::HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = Arc::new(HnswIndex::try_new(hnsw_config).unwrap()); // unwrap
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    // 1. k = 0 boundary check (must short-circuit to empty results without panic)
    let res_zero = col
        .hybrid_search("test", &[0.1, 0.2, 0.3, 0.4], 0, None)
        .await
        .unwrap(); // unwrap
    assert!(res_zero.is_empty(), "k=0 must return empty result list");

    // 2. k = usize::MAX boundary check (must clamp to MAX_SEARCH_K without panic/overflow)
    let res_max = col
        .hybrid_search("test", &[0.0, 0.0, 0.0, 0.0], usize::MAX, None)
        .await
        .unwrap(); // unwrap
    assert!(
        res_max.is_empty(),
        "k=usize::MAX on empty DB must return empty without overflow panic"
    );
}


#[tokio::test]
#[cfg(feature = "reranking")]
async fn test_hybrid_search_reranked_none() {
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // unwrap
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );
    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "d1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust language"})),
    )
    .await
    .unwrap(); // unwrap
    col.insert(
        "d2",
        &[0.9, 0.1, 0.0, 0.0],
        Some(serde_json::json!({"text": "python language"})),
    )
    .await
    .unwrap(); // unwrap

    let res = col
        .hybrid_search_reranked("rust", &[1.0, 0.0, 0.0, 0.0], 1, None, None)
        .await
        .unwrap(); // unwrap

    assert_eq!(res.len(), 1);
    assert_eq!(res[0].id, "d1");
}


#[tokio::test]
#[cfg(feature = "experimental-diskann")]
async fn test_collection_with_diskann_index_hybrid_search() {
    use contextra_core::{DocId, StorageEngine, TextIndex};
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_text::Language;
    use contextra_vector::{DiskAnnConfig, DiskAnnIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // unwrap
    let lsm_path = dir.path().join("lsm");
    let diskann_path = dir.path().join("diskann.idx");

    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: lsm_path,
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap
    );

    let diskann_config = DiskAnnConfig {
        index_path: diskann_path,
        dimension: 4,
        max_degree: 8,
        beam_width: 8,
        sector_size: 4096,
        ..DiskAnnConfig::default()
    };

    let diskann = Arc::new(DiskAnnIndex::try_new(diskann_config).unwrap()); // unwrap

    let doc1_id = DocId::from_key("doc1").unwrap(); // unwrap
    let doc2_id = DocId::from_key("doc2").unwrap(); // unwrap

    let vectors = vec![vec![1.0, 0.0, 0.0, 0.0], vec![0.0, 1.0, 0.0, 0.0]];
    let ids = vec![doc1_id, doc2_id];

    diskann.build(&vectors, &ids).await.unwrap(); // unwrap

    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::<LsmStorage, DiskAnnIndex>::new(
        "diskann_test".to_string(),
        storage.clone(),
        diskann,
        graph,
        next_tx,
        4,
        Language::English,
    );

    let tx = col.allocate_tx().unwrap(); // unwrap

    let doc1_user_key = col.namespaced_key(b"doc1", 0);
    let doc1_meta_key = col.namespaced_key(&doc1_id.inner().to_le_bytes(), 1);

    let doc2_user_key = col.namespaced_key(b"doc2", 0);
    let doc2_meta_key = col.namespaced_key(&doc2_id.inner().to_le_bytes(), 1);

    let doc1_data = crate::collection::StoredDocument {
        id: "doc1".to_string(),
        embedding: vec![1.0, 0.0, 0.0, 0.0],
        metadata: Some(serde_json::json!({ "text": "rust database systems" })),
    };
    let doc1_meta = crate::collection::StoredDocumentMeta::from(&doc1_data);

    let doc2_data = crate::collection::StoredDocument {
        id: "doc2".to_string(),
        embedding: vec![0.0, 1.0, 0.0, 0.0],
        metadata: Some(serde_json::json!({ "text": "python scripting language" })),
    };
    let doc2_meta = crate::collection::StoredDocumentMeta::from(&doc2_data);

    storage
        .put(tx, &doc1_user_key, &serde_json::to_vec(&doc1_data).unwrap()) // unwrap
        .await
        .unwrap(); // unwrap
    storage
        .put(tx, &doc1_meta_key, &serde_json::to_vec(&doc1_meta).unwrap()) // unwrap
        .await
        .unwrap(); // unwrap

    storage
        .put(tx, &doc2_user_key, &serde_json::to_vec(&doc2_data).unwrap()) // unwrap
        .await
        .unwrap(); // unwrap
    storage
        .put(tx, &doc2_meta_key, &serde_json::to_vec(&doc2_meta).unwrap()) // unwrap
        .await
        .unwrap(); // unwrap

    col.text_index
        .upsert_document(tx, doc1_id, "rust database systems")
        .await
        .unwrap(); // unwrap
    col.text_index
        .upsert_document(tx, doc2_id, "python scripting language")
        .await
        .unwrap(); // unwrap

    storage.commit(tx).await.unwrap(); // unwrap
    col.text_index.commit(tx).await.unwrap(); // unwrap

    let query_vector = vec![1.0, 0.0, 0.0, 0.0];
    let results = col
        .hybrid_search("rust", &query_vector, 5, None)
        .await
        .unwrap(); // unwrap

    assert!(
        !results.is_empty(),
        "Hybrid search with DiskANN should return results"
    );
    assert_eq!(
        results[0].id, "doc1",
        "Doc1 should be top result for rust & vector [1,0,0,0]"
    );
}


#[tokio::test]
async fn test_hybrid_search_with_query_memory_type_filter() {
    use contextra_core::{HybridQuery, MemoryType};
    use contextra_graph::CsrGraph;
    use contextra_store::{LsmConfig, LsmStorage};
    use contextra_vector::HnswIndex;
    use serde_json::json;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::tempdir().unwrap(); // unwrap
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );
    let col = Collection::new(
        "test_filter".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert_typed(
        "ep1",
        &[1.0, 0.0, 0.0, 0.0],
        MemoryType::Episodic,
        Some(json!({"text": "episode meeting alpha"})),
    )
    .await
    .unwrap(); // unwrap

    col.insert_typed(
        "ep2",
        &[0.9, 0.1, 0.0, 0.0],
        MemoryType::Episodic,
        Some(json!({"text": "episode meeting beta"})),
    )
    .await
    .unwrap(); // unwrap

    col.insert_typed(
        "sem1",
        &[0.95, 0.05, 0.0, 0.0],
        MemoryType::Semantic,
        Some(json!({"text": "episode definition gamma"})),
    )
    .await
    .unwrap(); // unwrap

    col.insert_typed(
        "sem2",
        &[0.85, 0.15, 0.0, 0.0],
        MemoryType::Semantic,
        Some(json!({"text": "episode theory delta"})),
    )
    .await
    .unwrap(); // unwrap

    // Query with memory_type_filter = Episodic
    let query_ep = HybridQuery::builder()
        .with_text_query("episode")
        .with_vector_query(vec![1.0, 0.0, 0.0, 0.0])
        .with_memory_type_filter(vec![MemoryType::Episodic])
        .with_k(10)
        .build()
        .unwrap(); // unwrap

    let results_ep = col.hybrid_search_with_query(&query_ep).await.unwrap(); // unwrap
    assert_eq!(
        results_ep.len(),
        2,
        "Must return exactly 2 episodic results"
    );
    for res in &results_ep {
        assert!(
            res.id == "ep1" || res.id == "ep2",
            "Returned result {} is not Episodic!",
            res.id
        );
    }

    // Query with memory_type_filter = Semantic
    let query_sem = HybridQuery::builder()
        .with_text_query("episode")
        .with_vector_query(vec![1.0, 0.0, 0.0, 0.0])
        .with_memory_type_filter(vec![MemoryType::Semantic])
        .with_k(10)
        .build()
        .unwrap(); // unwrap

    let results_sem = col.hybrid_search_with_query(&query_sem).await.unwrap(); // unwrap
    assert_eq!(
        results_sem.len(),
        2,
        "Must return exactly 2 semantic results"
    );
    for res in &results_sem {
        assert!(
            res.id == "sem1" || res.id == "sem2",
            "Returned result {} is not Semantic!",
            res.id
        );
    }
}


#[tokio::test]
async fn test_hybrid_search_fusion_capping_and_resilient_anchors() -> contextra_core::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_core::ContextraError::from)?;
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await?);
    let hnsw_config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = Arc::new(HnswIndex::try_new(hnsw_config)?);
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        "test_fusion_capping".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    // Insert documents
    col.insert(
        "doc_valid_1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust vector search", "type": "semantic"})),
    )
    .await?;
    col.insert(
        "doc_valid_2",
        &[0.9, 0.1, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust text search", "type": "semantic"})),
    )
    .await?;

    // Call hybrid_search with k=1
    let results = col
        .hybrid_search("rust", &[1.0, 0.0, 0.0, 0.0], 1, None)
        .await?;

    assert_eq!(results.len(), 1, "Fusion result must be capped to k=1");

    Ok(())
}


#[tokio::test]
async fn test_hybrid_search_snapshot_unsupported_strategies() -> contextra_core::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_core::ContextraError::from)?;
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await?);
    let hnsw_config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = Arc::new(HnswIndex::try_new(hnsw_config)?);
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        "test_snapshot_unsupported".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc_1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "graph node 1"})),
    )
    .await?;

    let ppr_strat = contextra_core::GraphTraversalStrategy::PersonalizedPageRank(
        contextra_core::PprConfig::default(),
    );
    let ppr_res = col
        .hybrid_search_with_strategy(
            "graph",
            &[1.0, 0.0, 0.0, 0.0],
            5,
            None,
            None,
            Some(&ppr_strat),
            None,
        )
        .await;

    assert!(
        ppr_res.is_ok(),
        "PPR under snapshot isolation must succeed, got {:?}",
        ppr_res
    );

    let eid_1 = contextra_core::EntityId::from_key("doc_1")?;
    let anchors = vec![eid_1];
    let path_rag_strat = contextra_core::GraphTraversalStrategy::PathRag {
        max_hops: 2,
        sufficiency_threshold: 0.5,
    };
    let path_res = col
        .hybrid_search_with_strategy(
            "graph",
            &[1.0, 0.0, 0.0, 0.0],
            5,
            Some(&anchors),
            None,
            Some(&path_rag_strat),
            None,
        )
        .await;

    assert!(
        path_res.is_err(),
        "PathRag under snapshot isolation must fail"
    );
    match path_res.unwrap_err() {
        contextra_core::ContextraError::SnapshotUnsupportedForSignal(msg) => {
            assert!(msg.contains("PathRag"));
        }
        other => panic!("Expected SnapshotUnsupportedForSignal, got: {:?}", other),
    }

    Ok(())
}
