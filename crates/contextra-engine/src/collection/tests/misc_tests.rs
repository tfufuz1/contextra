use super::fixtures::*;

#[tokio::test]
async fn test_collection_embedder_async_embed() {
    use contextra_ports::TextEmbeddingEngine;
    use std::sync::Arc;

    use contextra_ports::BoxFuture;

    struct FakeEmbedder;

    impl TextEmbeddingEngine for FakeEmbedder {
        fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, contextra_types::Result<Vec<f32>>> {
            Box::pin(async move { Ok(vec![text.len() as f32 / 100.0; 4]) })
        }
    }

    // Verify: compile-time proof that the method signature is async and
    // accepts Arc<dyn TextEmbeddingEngine>.
    let embedder: Arc<dyn TextEmbeddingEngine> = Arc::new(FakeEmbedder);
    let result = embedder.embed("hello").await.unwrap(); // unwrap
    assert_eq!(result.len(), 4);
}

#[tokio::test]
async fn test_input_guards_boundary_validation() {
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

    let vec = vec![1.0, 0.0, 0.0, 0.0];

    // 0. Empty inputs in relate()
    let err_relate_empty_from = col.relate("", "doc2", "knows").await;
    assert!(matches!(
        err_relate_empty_from,
        Err(contextra_types::ContextraError::InvalidInput(_))
    ));

    let err_relate_empty_to = col.relate("doc1", "", "knows").await;
    assert!(matches!(
        err_relate_empty_to,
        Err(contextra_types::ContextraError::InvalidInput(_))
    ));

    // 1. Empty ID guard on insert / upsert
    let err_empty_id = col.insert("", &vec, None).await;
    assert!(matches!(
        err_empty_id,
        Err(contextra_types::ContextraError::InvalidInput(_))
    ));

    let err_empty_id_upsert = col.upsert("", &vec, None).await;
    assert!(matches!(
        err_empty_id_upsert,
        Err(contextra_types::ContextraError::InvalidInput(_))
    ));

    // 2. Oversized ID guard (>1024 bytes)
    let long_id = "a".repeat(1025);
    let err_long_id = col.insert(&long_id, &vec, None).await;
    assert!(matches!(
        err_long_id,
        Err(contextra_types::ContextraError::InvalidInput(_))
    ));

    // 3. insert_many / upsert_many empty batch guard
    let err_empty_batch = col.insert_many(&[]).await;
    assert!(matches!(
        err_empty_batch,
        Err(contextra_types::ContextraError::InvalidInput(_))
    ));

    let err_empty_batch_upsert = col.upsert_many(&[]).await;
    assert!(matches!(
        err_empty_batch_upsert,
        Err(contextra_types::ContextraError::InvalidInput(_))
    ));

    // 4. insert_many / upsert_many oversized batch guard (>10,000)
    let huge_batch: Vec<_> = (0..10_001)
        .map(|i| (format!("d_{i}"), vec.clone(), None))
        .collect();
    let err_huge_batch = col.insert_many(&huge_batch).await;
    assert!(matches!(
        err_huge_batch,
        Err(contextra_types::ContextraError::InvalidInput(_))
    ));

    let err_huge_batch_upsert = col.upsert_many(&huge_batch).await;
    assert!(matches!(
        err_huge_batch_upsert,
        Err(contextra_types::ContextraError::InvalidInput(_))
    ));

    // 5. search / search_with_filter_expr k = 0 guard
    let err_search_k_zero = col.search(&vec, 0).await;
    assert!(matches!(
        err_search_k_zero,
        Err(contextra_types::ContextraError::InvalidInput(_))
    ));
}

#[tokio::test]
async fn test_doc_id_collision_rejected() {
    use contextra_graph::CsrGraph;
    use contextra_ports::StorageEngine;
    use contextra_store::LsmStorage;
    use contextra_types::{ContextraError, DocId, TxId};
    use contextra_vector::HnswIndex;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // unwrap
    let lsm_config = contextra_store::LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap()); // unwrap
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        graph,
        next_tx.clone(),
        4,
        contextra_text::Language::English,
    );

    // 1. Insert first document normally
    let id1 = "key_alpha";
    let emb1 = vec![1.0, 0.0, 0.0, 0.0];
    col.insert(id1, &emb1, None).await.unwrap(); // unwrap

    // Verify key_alpha exists
    let doc1 = col.get(id1).await.unwrap(); // unwrap
    assert!(doc1.is_some());

    // 2. Synthetically inject a mapping for a fixed DocId (e.g. DocId::new(42)) pointing to "key_existing"
    let synthetic_doc_id = DocId::new(42);
    let tx = TxId::new(next_tx.fetch_add(1, Ordering::SeqCst));
    let doc_key = col.namespaced_key(&synthetic_doc_id.inner().to_le_bytes(), 1);
    let existing_meta = crate::collection::StoredDocumentMeta {
        id: "key_existing".to_string(),
        metadata: None,
    };
    let meta_bytes = serde_json::to_vec(&existing_meta).unwrap(); // unwrap
    col.storage.put(tx, &doc_key, &meta_bytes).await.unwrap(); // unwrap
    col.storage.commit(tx).await.unwrap(); // unwrap

    // 3. Directly test check_doc_id_collision with a different string key (e.g., "key_new")
    let collision_res = col
        .check_doc_id_collision(synthetic_doc_id, "key_new")
        .await;
    assert!(collision_res.is_err());
    match collision_res {
        Err(ContextraError::Internal(msg)) => {
            assert!(
                msg.contains("DocId-Kollision erkannt für Schlüssel 'key_new'"),
                "Unexpected error message: {}",
                msg
            );
        }
        res => panic!("Expected ContextraError::Internal, got {:?}", res),
    }

    // 4. Same key string should NOT be treated as a collision
    let same_key_res = col
        .check_doc_id_collision(synthetic_doc_id, "key_existing")
        .await;
    assert!(same_key_res.is_ok());
}

#[tokio::test]
async fn test_extract_text_with_contextual_prefix() {
    use serde_json::json;

    let meta = Some(json!({
        "contextual_prefix": "Dokumenten-Kontext-Präfix",
        "text": "Chunk Haupttext"
    }));

    let extracted = crate::collection::extract_text(&meta);
    assert!(extracted.is_some());
    let text = extracted.unwrap(); // unwrap
    assert!(text.contains("Dokumenten-Kontext-Präfix"));
    assert!(text.contains("Chunk Haupttext"));
    assert_eq!(text, "Dokumenten-Kontext-Präfix\n\nChunk Haupttext");
}

#[test]
fn test_importance_score_parser_robust() {
    assert_eq!(crate::collection::parse_importance_score("0.8"), 0.8);
    assert_eq!(crate::collection::parse_importance_score("0.8\n"), 0.8);
    assert_eq!(crate::collection::parse_importance_score("Score: 0.8"), 0.8);
    assert_eq!(
        crate::collection::parse_importance_score("0.8 (high importance)"),
        0.8
    );
    assert_eq!(crate::collection::parse_importance_score("1.5"), 1.0);
    assert_eq!(crate::collection::parse_importance_score("-0.2"), 0.0);
    assert_eq!(
        crate::collection::parse_importance_score("invalid text"),
        0.5
    );
}

#[test]
fn test_compute_default_importance_entropy_and_clamping() {
    let score_empty = crate::collection::compute_default_importance(None);
    assert_eq!(score_empty.value(), 0.5);

    let score_simple = crate::collection::compute_default_importance(Some("aaaaa"));
    assert!(score_simple.value() >= 0.0 && score_simple.value() <= 1.0);

    let score_rich = crate::collection::compute_default_importance(Some(
        "The quick brown fox jumps over the lazy dog with high entropy and long text.",
    ));
    assert!(score_rich.value() > score_simple.value());
}

#[test]
fn test_extract_effective_importance_defaults() {
    use contextra_types::TxId;

    let none_meta = None;
    assert_eq!(
        crate::collection::extract_effective_importance(&none_meta, TxId::new(10)),
        1.0
    );

    let meta_with_imp = Some(serde_json::json!({
        "importance": 0.85
    }));
    assert_eq!(
        crate::collection::extract_effective_importance(&meta_with_imp, TxId::new(10)),
        0.85
    );
}

#[test]
fn test_importance_metadata_integration_and_filtering() {
    use contextra_types::{DecayFunction, ImportanceScore, MemoryImportance, TxId};
    use serde_json::json;

    let created_tx = TxId::new(10);
    let now_tx = TxId::new(30);

    let mut meta1 = Some(json!({"text": "Important factual doc"}));
    crate::collection::ensure_importance_metadata(
        &mut meta1,
        created_tx,
        Some("Important factual doc"),
    );

    // Override with explicit exponential decay
    let imp1 = MemoryImportance::new(
        ImportanceScore::new(0.9),
        DecayFunction::Exponential { half_life_tx: 10 },
        created_tx,
    );
    meta1.as_mut().unwrap().as_object_mut().unwrap().insert(
        // unwrap
        "importance".to_string(),
        serde_json::to_value(imp1).unwrap(), // unwrap
    );

    // Effective score at now_tx (2 half-lives elapsed) -> 0.9 * 0.25 = 0.225
    let eff1 = crate::collection::extract_effective_importance(&meta1, now_tx);
    assert!((eff1 - 0.225).abs() < 1e-4);

    let mut meta2 = Some(json!({"text": "Critical doc"}));
    let imp2 = MemoryImportance::new(ImportanceScore::new(1.0), DecayFunction::None, created_tx);
    meta2.as_mut().unwrap().as_object_mut().unwrap().insert(
        // unwrap
        "importance".to_string(),
        serde_json::to_value(imp2).unwrap(), // unwrap
    );

    let results = vec![
        crate::SearchResult {
            id: "doc1".to_string(),
            score: 0.95,
            metadata: meta1,
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
        crate::SearchResult {
            id: "doc2".to_string(),
            score: 0.85,
            metadata: meta2,
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
    ];

    // Filter out results with effective importance < 0.5
    let filtered =
        Collection::<contextra_store::LsmStorage>::filter_by_importance(results, 0.5, now_tx);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "doc2");
    assert_eq!(filtered[0].score, 0.85); // Order and original RRF/CE score preserved
}

#[tokio::test]
async fn test_invalid_doc_ids_rejected() {
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
    let vec = vec![1.0, 0.0, 0.0, 0.0];

    // Empty ID
    assert!(col.insert("", &vec, None).await.is_err());
    assert!(col.get("").await.is_err());
    assert!(col.delete("").await.is_err());

    // Null byte in ID
    assert!(col.insert("doc\0invalid", &vec, None).await.is_err());
    assert!(col.get("doc\0invalid").await.is_err());

    // Too long ID (>256 bytes)
    let long_id = "a".repeat(257);
    assert!(col.insert(&long_id, &vec, None).await.is_err());
    assert!(col.get(&long_id).await.is_err());
}

#[tokio::test]
async fn test_collection_mandatory_matrix_happy_path_hand_calculated() -> contextra_types::Result<()>
{
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await?,
    );
    let index = Arc::new(HnswIndex::try_new(contextra_vector::HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "matrix_happy".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    let doc_id = "doc_matrix_1";
    let vec = vec![1.0, 0.0, 0.0, 0.0];
    let meta = serde_json::json!({ "title": "hand_calculated_constant", "value": 42 });

    col.insert(doc_id, &vec, Some(meta.clone())).await?;

    let retrieved = col.get(doc_id).await?;
    assert!(retrieved.is_some());
    let doc = retrieved.unwrap();
    assert_eq!(doc.id, doc_id);
    let meta_obj = doc.metadata.as_ref().unwrap().as_object().unwrap();
    assert_eq!(meta_obj.get("title").unwrap(), "hand_calculated_constant");
    assert_eq!(meta_obj.get("value").unwrap(), 42);

    let search_res = col.search(&vec, 1).await?;
    assert_eq!(search_res.len(), 1);
    assert_eq!(search_res[0].id, doc_id);
    Ok(())
}

#[tokio::test]
async fn test_collection_mandatory_matrix_empty_inputs() -> contextra_types::Result<()> {
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await?,
    );
    let index = Arc::new(HnswIndex::try_new(contextra_vector::HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "matrix_empty".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    // Empty collection search must return Ok(Vec::new()) without error
    let res = col.search(&[1.0, 0.0, 0.0, 0.0], 10).await?;
    assert!(res.is_empty());

    // Empty text query in builder must return Ok(Vec::new())
    let builder_res = col.query().k(5).execute().await?;
    assert!(builder_res.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_collection_mandatory_matrix_error_paths() -> contextra_types::Result<()> {
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await?,
    );
    let index = Arc::new(HnswIndex::try_new(contextra_vector::HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "matrix_errors".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    // Dimension mismatch
    let err_dim = col.insert("d1", &[1.0, 0.0], None).await;
    assert!(matches!(
        err_dim,
        Err(contextra_types::ContextraError::InvalidInput(_))
    ));

    // Empty string ID
    let err_id = col.insert("", &[1.0, 0.0, 0.0, 0.0], None).await;
    assert!(matches!(
        err_id,
        Err(contextra_types::ContextraError::InvalidInput(_))
    ));

    Ok(())
}

#[tokio::test]
async fn test_apm7_utf8_multibyte_boundary_handling() -> contextra_types::Result<()> {
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await?,
    );
    let index = Arc::new(HnswIndex::try_new(contextra_vector::HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "apm7_utf8".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::German,
    );

    // Document keys and content containing German umlauts, CJK characters, and Emojis
    let doc_id = "doc_üöä_🦀_中文";
    let text_content = "Spezielle Retrieval-Engine mit Übereinstimmung & Emojis 🚀";
    let vec = vec![0.5, 0.5, 0.0, 0.0];

    col.insert(
        doc_id,
        &vec,
        Some(serde_json::json!({ "text": text_content })),
    )
    .await?;

    let retrieved = col.get(doc_id).await?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, doc_id);

    let search_res = col.query().text("Übereinstimmung").k(5).execute().await?;
    assert!(!search_res.is_empty());
    assert_eq!(search_res[0].id, doc_id);

    Ok(())
}

proptest::proptest! {
    #[test]
    fn prop_markdown_chunker_never_panics_on_arbitrary_utf8(
        input in proptest::prelude::any::<String>()
    ) {
        let chunker = crate::chunker::MarkdownChunker::with_defaults();
        let doc_id = contextra_types::DocId::new(1);
        let _ = chunker.chunk(doc_id, &input);
    }
}
