//! # Modul: search_tests.rs
//!
//! **Zweck:** Tests für Einzel-Signal-Suchen (Text/Vektor).
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, strikte Mandanten- und Indexisolation.

use super::fixtures::*;

#[tokio::test]
async fn test_search_dimension_mismatch_rejected() {
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
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );
    let wrong_dim_vec = vec![1.0, 0.0];

    let search_res = col.search(&wrong_dim_vec, 10).await;
    assert!(search_res.is_err());

    let hybrid_res = col.hybrid_search("query", &wrong_dim_vec, 10).await;
    assert!(hybrid_res.is_err());
}

#[tokio::test]
async fn test_search_k_zero_returns_canonical_error_message(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir()?;
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
        "default".to_string(),
        storage,
        index,
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    let query_vec = vec![1.0, 0.0, 0.0, 0.0];

    let err_search = col.search(&query_vec, 0).await;
    assert!(err_search.is_err());
    let err_msg_1 = match err_search {
        Err(e) => e.to_string(),
        Ok(_) => unreachable!(),
    };
    assert!(
        err_msg_1.contains("Search k must be greater than 0"),
        "Expected 'Search k must be greater than 0', got: {err_msg_1}"
    );

    let err_expr = col.search_with_filter_expr(&query_vec, 0, None).await;
    assert!(err_expr.is_err());
    let err_msg_2 = match err_expr {
        Err(e) => e.to_string(),
        Ok(_) => unreachable!(),
    };
    assert!(
        err_msg_2.contains("Search k must be greater than 0"),
        "Expected 'Search k must be greater than 0', got: {err_msg_2}"
    );
    Ok(())
}

#[tokio::test]
async fn test_query_builder_query_config_include_superseded_displacement(
) -> contextra_types::Result<()> {
    use contextra_store::LsmStorage;
    use contextra_types::{DocId, HybridQuery};
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
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "old_doc",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "outdated information"})),
    )
    .await?;

    col.insert(
        "new_doc",
        &[0.95, 0.05, 0.0, 0.0],
        Some(serde_json::json!({"text": "updated information"})),
    )
    .await?;

    col.link_memories(
        DocId::from_key("new_doc")?,
        DocId::from_key("old_doc")?,
        contextra_types::domain::LinkRelation::Supersedes,
    )
    .await?;

    let hybrid_query = HybridQuery::builder()
        .with_vector_query(vec![1.0, 0.0, 0.0, 0.0])
        .with_include_superseded(false)
        .with_k(10)
        .build()
        .unwrap(); // unwrap

    let results = col.query().query_config(&hybrid_query).execute().await?;

    assert!(
        !results.iter().any(|r| r.id == "old_doc"),
        "old_doc must be displaced when executed via QueryBuilder with include_superseded=false"
    );
    assert!(
        results.iter().any(|r| r.id == "new_doc"),
        "new_doc must be included in results"
    );

    Ok(())
}

#[tokio::test]
async fn test_post_rrf_supersedes_displacement_truncation_preserves_k(
) -> contextra_types::Result<()> {
    use contextra_store::LsmStorage;
    use contextra_types::DocId;
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
        .await
        .unwrap(),
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(),
    );
    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    // Insert 4 docs: doc1, doc2, doc3, doc4
    col.insert(
        "doc1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "alpha"})),
    )
    .await?;
    col.insert(
        "doc2",
        &[0.9, 0.1, 0.0, 0.0],
        Some(serde_json::json!({"text": "beta"})),
    )
    .await?;
    col.insert(
        "doc3",
        &[0.8, 0.2, 0.0, 0.0],
        Some(serde_json::json!({"text": "gamma"})),
    )
    .await?;
    col.insert(
        "doc4",
        &[0.7, 0.3, 0.0, 0.0],
        Some(serde_json::json!({"text": "delta"})),
    )
    .await?;

    // doc2 supersedes doc1
    col.link_memories(
        DocId::from_key("doc2")?,
        DocId::from_key("doc1")?,
        contextra_types::domain::LinkRelation::Supersedes,
    )
    .await?;

    // When searching with k = 2 and include_superseded = false,
    // doc1 is displaced by doc2.
    // With 3*k candidate pool, doc3 advances into top-2 so we still get 2 results!
    let query = contextra_types::HybridQuery::builder()
        .with_vector_query(vec![1.0, 0.0, 0.0, 0.0])
        .with_k(2)
        .with_include_superseded(false)
        .build()
        .unwrap();
    let results = col.hybrid_search_with_query(&query).await?;

    assert_eq!(
        results.len(),
        2,
        "Must return full requested k=2 even after supersedes displacement"
    );
    assert!(
        !results.iter().any(|r| r.id == "doc1"),
        "doc1 must be displaced"
    );
    assert!(
        results.iter().any(|r| r.id == "doc2"),
        "doc2 must be retained"
    );
    assert!(
        results.iter().any(|r| r.id == "doc3"),
        "doc3 must move up into top-2"
    );

    Ok(())
}
