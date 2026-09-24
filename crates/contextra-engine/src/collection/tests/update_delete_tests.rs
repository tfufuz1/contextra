use super::fixtures::*;

#[tokio::test]
async fn test_update_document_importance_persists_model_id_provenance() {
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
    col.insert("doc_test_prov", &vec, None).await.unwrap(); // unwrap

    col.update_document_importance("doc_test_prov", 0.92, "llama3:latest")
        .await
        .unwrap(); // unwrap

    let doc = col.get("doc_test_prov").await.unwrap().unwrap(); // unwrap
    let meta = doc.metadata.unwrap(); // unwrap

    let imp = meta.get("importance").unwrap();
    let imp_score: contextra_types::MemoryImportance = serde_json::from_value(imp.clone()).unwrap();
    assert_eq!(imp_score.base_score.value(), 0.92);
    assert_eq!(
        meta.get("model_id").and_then(|v| v.as_str()),
        Some("llama3:latest")
    );
}


#[tokio::test]
async fn test_expiry_cleanup_deletes_decayed_working_memory() {
    use contextra_types::{DecayFunction, ImportanceScore, MemoryImportance, TxId};
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};
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
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        next_tx.clone(),
        4,
        contextra_text::Language::English,
    );

    let created_tx = TxId::new(10);
    let imp = MemoryImportance::new(
        ImportanceScore::new(0.5),
        DecayFunction::Exponential { half_life_tx: 5 },
        created_tx,
    );

    let vec = vec![1.0, 0.0, 0.0, 0.0];
    col.insert(
        "doc_decayed",
        &vec,
        Some(json!({
            "importance": imp
        })),
    )
    .await
    .unwrap(); // unwrap

    // Advance TxId far enough so effective_score < 0.05
    // At created_tx=10, half_life=5:
    // Tx 10: 0.5 * 1.0 = 0.5
    // Tx 15: 0.5 * 0.5 = 0.25
    // Tx 20: 0.5 * 0.25 = 0.125
    // Tx 25: 0.5 * 0.125 = 0.0625
    // Tx 30: 0.5 * 0.0625 = 0.03125 (< 0.05)
    next_tx.store(35, Ordering::SeqCst);

    let count = col.trigger_expiry_cleanup().await.unwrap(); // unwrap
    assert_eq!(count, 1, "Decayed working memory document should be reaped");
    assert!(col.get("doc_decayed").await.unwrap().is_none()); // unwrap
}


#[tokio::test]
async fn test_expiry_cleanup_never_deletes_semantic_no_decay() {
    use contextra_types::{DecayFunction, ImportanceScore, MemoryImportance, TxId};
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};
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
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        next_tx.clone(),
        4,
        contextra_text::Language::English,
    );

    let created_tx = TxId::new(10);
    let imp = MemoryImportance::new(
        ImportanceScore::new(0.01), // even with base score < 0.05!
        DecayFunction::None,
        created_tx,
    );

    let vec = vec![1.0, 0.0, 0.0, 0.0];
    col.insert(
        "doc_semantic",
        &vec,
        Some(json!({
            "importance": imp
        })),
    )
    .await
    .unwrap(); // unwrap

    // Advance TxId very far
    next_tx.store(100_000, Ordering::SeqCst);

    let count = col.trigger_expiry_cleanup().await.unwrap(); // unwrap
    assert_eq!(
        count, 0,
        "Semantic document with DecayFunction::None must never be deleted"
    );
    assert!(col.get("doc_semantic").await.unwrap().is_some()); // unwrap
}


#[tokio::test]
async fn test_ttl_missing_created_at_does_not_expire() {
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use serde_json::json;
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
        "doc_no_created_at",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"ttl_ms": 10})),
    )
    .await
    .unwrap(); // unwrap
    let reaped = col.trigger_expiry_cleanup().await.unwrap(); // unwrap
    assert_eq!(reaped, 0);
    assert!(col.get("doc_no_created_at").await.unwrap().is_some()); // unwrap
}


#[tokio::test]
async fn test_ttl_zero_does_not_expire() {
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use serde_json::json;
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
        "doc_zero_ttl",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"created_at_ms": 100, "ttl_ms": 0})),
    )
    .await
    .unwrap(); // unwrap
    let reaped = col.trigger_expiry_cleanup().await.unwrap(); // unwrap
    assert_eq!(reaped, 0);
    assert!(col.get("doc_zero_ttl").await.unwrap().is_some()); // unwrap
}


#[tokio::test]
async fn test_ttl_overflow_does_not_expire() {
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use serde_json::json;
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
        "doc_overflow",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"created_at_ms": u64::MAX - 10, "ttl_ms": 100})),
    )
    .await
    .unwrap(); // unwrap
    let reaped = col.trigger_expiry_cleanup().await.unwrap(); // unwrap
    assert_eq!(reaped, 0);
    assert!(col.get("doc_overflow").await.unwrap().is_some()); // unwrap
}
