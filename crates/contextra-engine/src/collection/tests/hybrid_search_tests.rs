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
        res.len() <= contextra_types::MAX_SEARCH_K,
        "Results length {} should be <= MAX_SEARCH_K ({})",
        res.len(),
        contextra_types::MAX_SEARCH_K
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
    use contextra_graph::CsrGraph;
    use contextra_ports::{StorageEngine, TextIndex};
    use contextra_store::LsmStorage;
    use contextra_text::Language;
    use contextra_types::DocId;
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
    use contextra_graph::CsrGraph;
    use contextra_store::{LsmConfig, LsmStorage};
    use contextra_types::{HybridQuery, MemoryType};
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
async fn test_hybrid_search_fusion_capping_and_resilient_anchors() -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
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
async fn test_hybrid_search_with_strategy_score_normalized_ranking() -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_types::{FusionStrategy, SignalFusionStrategies};
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let storage = Arc::new(LsmStorage::new(LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    }).await?);
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "score_norm_ranking".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc_alpha",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust database search engine"})),
    )
    .await?;

    col.insert(
        "doc_beta",
        &[0.2, 0.8, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust database search engine rust rust"})),
    )
    .await?;

    let score_norm_strat = SignalFusionStrategies::uniform(FusionStrategy::ScoreNormalized);
    let norm_res = col
        .hybrid_search_with_strategy(
            "rust",
            &[1.0, 0.0, 0.0, 0.0],
            2,
            None,
            None,
            None,
            None,
            Some(score_norm_strat),
        )
        .await?;

    let rrf_strat = SignalFusionStrategies::uniform(FusionStrategy::Rrf);
    let rrf_res = col
        .hybrid_search_with_strategy(
            "rust",
            &[1.0, 0.0, 0.0, 0.0],
            2,
            None,
            None,
            None,
            None,
            Some(rrf_strat),
        )
        .await?;

    assert_eq!(norm_res.len(), 2);
    assert_eq!(rrf_res.len(), 2);

    // Verify ScoreNormalized score computation differs from RRF
    assert_ne!(norm_res[0].score, rrf_res[0].score);
    assert!(norm_res[0].score.is_finite());
    assert!(norm_res[1].score.is_finite());

    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_mixed_signal_fusion_strategies() -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_types::{FusionStrategy, SignalFusionStrategies};
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let storage = Arc::new(LsmStorage::new(LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    }).await?);
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "mixed_fusion_strategies".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc_v1_t2",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "vector top match"})),
    )
    .await?;

    col.insert(
        "doc_v2_t1",
        &[0.1, 0.9, 0.0, 0.0],
        Some(serde_json::json!({"text": "vector top match text top match text top match"})),
    )
    .await?;

    let pure_rrf = SignalFusionStrategies::uniform(FusionStrategy::Rrf);
    let pure_norm = SignalFusionStrategies::uniform(FusionStrategy::ScoreNormalized);
    let mixed_strat = SignalFusionStrategies {
        vector: FusionStrategy::Rrf,
        text: FusionStrategy::ScoreNormalized,
        graph: FusionStrategy::Rrf,
    };

    let run_pure_rrf = col
        .hybrid_search_with_strategy(
            "text",
            &[1.0, 0.0, 0.0, 0.0],
            2,
            None,
            None,
            None,
            None,
            Some(pure_rrf),
        )
        .await?;

    let run_pure_norm = col
        .hybrid_search_with_strategy(
            "text",
            &[1.0, 0.0, 0.0, 0.0],
            2,
            None,
            None,
            None,
            None,
            Some(pure_norm),
        )
        .await?;

    let run_mixed = col
        .hybrid_search_with_strategy(
            "text",
            &[1.0, 0.0, 0.0, 0.0],
            2,
            None,
            None,
            None,
            None,
            Some(mixed_strat),
        )
        .await?;

    assert_eq!(run_pure_rrf.len(), 2);
    assert_eq!(run_pure_norm.len(), 2);
    assert_eq!(run_mixed.len(), 2);

    // Verify mixed strategy combines vector RRF contribution + text ScoreNormalized contribution
    // Prove that mixed score for doc_v1_t2 is distinct from pure_rrf and pure_norm
    let mixed_doc0_score = run_mixed[0].score;
    let rrf_doc0_score = run_pure_rrf[0].score;
    let norm_doc0_score = run_pure_norm[0].score;

    assert_ne!(
        mixed_doc0_score, rrf_doc0_score,
        "Mixed strategy score must differ from pure RRF score"
    );
    assert_ne!(
        mixed_doc0_score, norm_doc0_score,
        "Mixed strategy score must differ from pure ScoreNormalized score"
    );

    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_single_fusion_strategy_regression_parity() -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_types::{FusionStrategy, SignalFusionStrategies};
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let storage = Arc::new(LsmStorage::new(LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    }).await?);
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "regression_parity".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc_1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "regression parity test document one"})),
    )
    .await?;

    col.insert(
        "doc_2",
        &[0.5, 0.5, 0.0, 0.0],
        Some(serde_json::json!({"text": "regression parity test document two"})),
    )
    .await?;

    // Calling with single FusionStrategy converted via From<FusionStrategy> (.into())
    let single_rrf: SignalFusionStrategies = FusionStrategy::Rrf.into();
    let uniform_rrf = SignalFusionStrategies::uniform(FusionStrategy::Rrf);

    let res_single = col
        .hybrid_search_with_strategy(
            "regression",
            &[1.0, 0.0, 0.0, 0.0],
            2,
            None,
            None,
            None,
            None,
            Some(single_rrf),
        )
        .await?;

    let res_uniform = col
        .hybrid_search_with_strategy(
            "regression",
            &[1.0, 0.0, 0.0, 0.0],
            2,
            None,
            None,
            None,
            None,
            Some(uniform_rrf),
        )
        .await?;

    assert_eq!(res_single.len(), res_uniform.len());
    for (a, b) in res_single.iter().zip(res_uniform.iter()) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.score.to_bits(), b.score.to_bits());
    }

    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_fusion_strategy_rrf_golden_parity() -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_types::FusionStrategy;
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let storage = Arc::new(LsmStorage::new(LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    }).await?);
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "golden_parity".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust vector search engine"})),
    )
    .await?;

    col.insert(
        "doc2",
        &[0.8, 0.2, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust database storage"})),
    )
    .await?;

    // Default strategy (None) vs explicit FusionStrategy::Rrf
    let default_res = col
        .hybrid_search_with_strategy(
            "rust",
            &[1.0, 0.0, 0.0, 0.0],
            5,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

    let explicit_rrf_res = col
        .hybrid_search_with_strategy(
            "rust",
            &[1.0, 0.0, 0.0, 0.0],
            5,
            None,
            None,
            None,
            None,
            Some(FusionStrategy::Rrf.into()),
        )
        .await?;

    assert_eq!(default_res.len(), explicit_rrf_res.len());
    for (a, b) in default_res.iter().zip(explicit_rrf_res.iter()) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.score, b.score);
    }

    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_score_normalized_reorders_results() -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_types::FusionStrategy;
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let storage = Arc::new(LsmStorage::new(LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    }).await?);
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "score_norm_reorder".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    // Insert doc_a, doc_b, doc_c configured to demonstrate RRF vs ScoreNormalized reordering.
    // doc_a: high vector similarity [1.0, 0, 0, 0], moderate text match
    // doc_b: low vector similarity [0.1, 0.9, 0, 0], highest text match
    // doc_c: low vector similarity [0.0, 1.0, 0, 0], medium text match
    col.insert(
        "doc_a",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "search query target alpha"})),
    )
    .await?;

    col.insert(
        "doc_b",
        &[0.1, 0.9, 0.0, 0.0],
        Some(serde_json::json!({"text": "search query target alpha query query"})),
    )
    .await?;

    col.insert(
        "doc_c",
        &[0.0, 1.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "search query target alpha query"})),
    )
    .await?;

    let weights = contextra_types::FusionWeights::new(0.40, 0.30, 0.30).map_err(|e| contextra_types::ContextraError::InvalidInput(e.to_string()))?;

    let rrf_results = col
        .hybrid_search_with_strategy(
            "query",
            &[1.0, 0.0, 0.0, 0.0],
            3,
            None,
            Some(&weights),
            None,
            None,
            Some(FusionStrategy::Rrf.into()),
        )
        .await?;

    let norm_results = col
        .hybrid_search_with_strategy(
            "query",
            &[1.0, 0.0, 0.0, 0.0],
            3,
            None,
            Some(&weights),
            None,
            None,
            Some(FusionStrategy::ScoreNormalized.into()),
        )
        .await?;

    assert_eq!(rrf_results.len(), 3);
    assert_eq!(norm_results.len(), 3);

    // Verify that ScoreNormalized and RRF result orderings differ due to CombSUM score normalization
    let rrf_ids: Vec<&str> = rrf_results.iter().map(|r| r.id.as_str()).collect();
    let norm_ids: Vec<&str> = norm_results.iter().map(|r| r.id.as_str()).collect();

    assert_ne!(
        rrf_ids, norm_ids,
        "ScoreNormalized must produce a different result ordering than RRF in this constructed scenario. RRF: {rrf_ids:?}, ScoreNormalized: {norm_ids:?}"
    );

    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_score_normalized_constant_scores_fallback_to_rrf() -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_types::FusionStrategy;
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let storage = Arc::new(LsmStorage::new(LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    }).await?);
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "constant_scores_fallback".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    // Insert 2 documents with identical embeddings and identical text content -> constant score distribution
    col.insert(
        "doc_same_1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "identical text content"})),
    )
    .await?;

    col.insert(
        "doc_same_2",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "identical text content"})),
    )
    .await?;

    let norm_res = col
        .hybrid_search_with_strategy(
            "identical",
            &[1.0, 0.0, 0.0, 0.0],
            2,
            None,
            None,
            None,
            None,
            Some(FusionStrategy::ScoreNormalized.into()),
        )
        .await?;

    assert_eq!(norm_res.len(), 2);
    // Degenerate score distribution triggers fallback to RRF without panic
    assert!(norm_res[0].score.is_finite());
    assert!(norm_res[1].score.is_finite());

    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_score_normalized_determinism_100_runs() -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_types::FusionStrategy;
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let storage = Arc::new(LsmStorage::new(LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    }).await?);
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "determinism_100_runs".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc_1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "deterministic search alpha"})),
    )
    .await?;

    col.insert(
        "doc_2",
        &[0.7, 0.3, 0.0, 0.0],
        Some(serde_json::json!({"text": "deterministic search beta"})),
    )
    .await?;

    col.insert(
        "doc_3",
        &[0.5, 0.5, 0.0, 0.0],
        Some(serde_json::json!({"text": "deterministic search gamma"})),
    )
    .await?;

    let baseline = col
        .hybrid_search_with_strategy(
            "deterministic",
            &[1.0, 0.0, 0.0, 0.0],
            3,
            None,
            None,
            None,
            None,
            Some(FusionStrategy::ScoreNormalized.into()),
        )
        .await?;

    for run_idx in 1..=100 {
        let run_res = col
            .hybrid_search_with_strategy(
                "deterministic",
                &[1.0, 0.0, 0.0, 0.0],
                3,
                None,
                None,
                None,
                None,
                Some(FusionStrategy::ScoreNormalized.into()),
            )
            .await?;

        assert_eq!(
            baseline.len(),
            run_res.len(),
            "Run #{run_idx} result count mismatched baseline"
        );

        for (a, b) in baseline.iter().zip(run_res.iter()) {
            assert_eq!(
                a.id, b.id,
                "Run #{run_idx} document ID order mismatch: expected {}, got {}",
                a.id, b.id
            );
            assert_eq!(
                a.score.to_bits(),
                b.score.to_bits(),
                "Run #{run_idx} score bit representation mismatch for doc {}: expected {}, got {}",
                a.id,
                a.score,
                b.score
            );
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_community_boost_consistency_across_fusion_strategies() -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_types::{EntityId, FusionStrategy};
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let storage = Arc::new(LsmStorage::new(LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    }).await?);
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "community_boost_consistency".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc_comm_target",
        &[0.8, 0.2, 0.0, 0.0],
        Some(serde_json::json!({"text": "community boosted topic"})),
    )
    .await?;

    col.insert(
        "doc_comm_other",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "community boosted topic"})),
    )
    .await?;

    let eid_target = EntityId::from_key("doc_comm_target")?;

    col.relate("doc_comm_target", "doc_neighbor", "knows").await?;
    col.run_community_detection().await?;

    // Test with FusionStrategy::Rrf
    let rrf_boosted = col
        .hybrid_search_with_strategy(
            "community",
            &[1.0, 0.0, 0.0, 0.0],
            5,
            None,
            None,
            None,
            Some(eid_target),
            Some(FusionStrategy::Rrf.into()),
        )
        .await?;

    // Test with FusionStrategy::ScoreNormalized
    let norm_boosted = col
        .hybrid_search_with_strategy(
            "community",
            &[1.0, 0.0, 0.0, 0.0],
            5,
            None,
            None,
            None,
            Some(eid_target),
            Some(FusionStrategy::ScoreNormalized.into()),
        )
        .await?;

    assert!(!rrf_boosted.is_empty());
    assert!(!norm_boosted.is_empty());

    // In both strategies, community boosting must boost doc_comm_target to rank #1
    assert_eq!(
        rrf_boosted[0].id, "doc_comm_target",
        "Community member must rank #1 under RRF post-fusion boost"
    );
    assert_eq!(
        norm_boosted[0].id, "doc_comm_target",
        "Community member must rank #1 under ScoreNormalized post-fusion boost"
    );

    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_snapshot_unsupported_strategies() -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
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

    let ppr_strat = contextra_types::GraphTraversalStrategy::PersonalizedPageRank(
        contextra_types::PprConfig::default(),
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
            None,
        )
        .await;

    assert!(
        ppr_res.is_ok(),
        "PPR under snapshot isolation must succeed, got {:?}",
        ppr_res
    );

    let eid_1 = contextra_types::EntityId::from_key("doc_1")?;
    let anchors = vec![eid_1];
    let path_rag_strat = contextra_types::GraphTraversalStrategy::PathRag {
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
            None,
        )
        .await;

    assert!(
        path_res.is_err(),
        "PathRag under snapshot isolation must fail"
    );
    match path_res.unwrap_err() {
        contextra_types::ContextraError::SnapshotUnsupportedForSignal(msg) => {
            assert!(msg.contains("PathRag"));
        }
        other => panic!("Expected SnapshotUnsupportedForSignal, got: {:?}", other),
    }

    Ok(())
}
