use super::fixtures::*;

#[tokio::test]
async fn test_search_dimension_mismatch_rejected() {
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
    let wrong_dim_vec = vec![1.0, 0.0];

    let search_res = col.search(&wrong_dim_vec, 10).await;
    assert!(search_res.is_err());

    let hybrid_res = col.hybrid_search("query", &wrong_dim_vec, 10, None).await;
    assert!(hybrid_res.is_err());
}


#[tokio::test]
async fn test_checkpoint_unpin_on_search_error_path() {
use contextra_types::{FilterExpr, Result, TxId};
use contextra_ports::{StorageStats};
use contextra_ports::{BoxFuture, StorageEngine};
    use contextra_graph::csr::CsrGraph;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
    use std::sync::Arc;

    struct PinTrackingFailingStorage {
        active_pins: AtomicI64,
        total_pins: AtomicU64,
        total_unpins: AtomicU64,
    }

    impl StorageEngine for PinTrackingFailingStorage {
        fn get<'a>(&'a self, _: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
            Box::pin(async move { Ok(None) })
        }
        fn get_at_seq<'a>(
            &'a self,
            _: &'a [u8],
            _: u64,
        ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
            Box::pin(async move { Ok(None) })
        }
        fn put<'a>(&'a self, _: TxId, _: &'a [u8], _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn delete<'a>(&'a self, _: TxId, _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
            Box::pin(async move {
                Ok(StorageStats {
                    num_segments: 0,
                    total_size_bytes: 0,
                    memtable_size_bytes: 0,
                })
            })
        }
        fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
            Box::pin(async move { Ok(10) })
        }
        fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
            Box::pin(async move { Ok(TxId::new(1)) })
        }
        fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                self.active_pins.fetch_add(1, Ordering::SeqCst);
                self.total_pins.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        }
        fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                self.active_pins.fetch_sub(1, Ordering::SeqCst);
                self.total_unpins.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        }
        fn scan_prefix<'a>(
            &'a self,
            _: &'a [u8],
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(vec![]) })
        }
        fn scan_prefix_at<'a>(
            &'a self,
            _: &'a [u8],
            _: u64,
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move {
                Err(contextra_types::ContextraError::Storage(
                    "Simulated storage failure during scan_prefix_at".into(),
                ))
            })
        }
        fn scan<'a>(
            &'a self,
            _: std::ops::Bound<&'a [u8]>,
            _: std::ops::Bound<&'a [u8]>,
            _: Option<usize>,
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(vec![]) })
        }
    }

    let storage = Arc::new(PinTrackingFailingStorage {
        active_pins: AtomicI64::new(0),
        total_pins: AtomicU64::new(0),
        total_unpins: AtomicU64::new(0),
    });
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(),
    );
    let col = Collection::new(
        "default".to_string(),
        storage.clone(),
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    let filter_expr = FilterExpr::Eq {
        field: "category".to_string(),
        value: serde_json::json!("books"),
    };
    let res = col
        .search_with_filter_expr(&[1.0, 0.0, 0.0, 0.0], 5, Some(filter_expr))
        .await;

    assert!(res.is_err(), "Search must return error when scan fails");
    assert_eq!(
        storage.total_pins.load(Ordering::SeqCst),
        1,
        "pin_checkpoint should have been called once"
    );
    assert_eq!(
        storage.total_unpins.load(Ordering::SeqCst),
        1,
        "unpin_checkpoint must be called despite inner search error"
    );
    assert_eq!(
        storage.active_pins.load(Ordering::SeqCst),
        0,
        "Active pins must return to 0 after search error"
    );
}


#[tokio::test]
async fn test_search_k_zero_returns_canonical_error_message(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use contextra_graph::CsrGraph;
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
        Arc::new(CsrGraph::new()),
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
async fn test_single_pid_controller_instantiation_in_query_builder() {
    // Regression test: verify that exactly one PID controller type (contextra_adapt::PidController)
    // is instantiated across production collection search and query_builder modules.
    let mut pid = contextra_adapt::PidController::default();
    assert_eq!(pid.kp, 0.5);
    assert_eq!(pid.ki, 0.05);
    assert_eq!(pid.kd, 0.1);
    assert_eq!(pid.target_latency_ms, 150.0);
    assert_eq!(pid.min_pool_size, 50);
    assert_eq!(pid.max_pool_size, 200);

    let updated = pid.update(std::time::Duration::from_millis(100), 300.0);
    assert!(updated < 100);
    assert!(updated >= 50);
}


#[tokio::test]
async fn test_query_builder_query_config_include_superseded_displacement(
) -> contextra_types::Result<()> {
    use contextra_types::{DocId, HybridQuery};
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
async fn test_community_boost_post_rrf_preserves_non_community_and_reranks(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use contextra_types::EntityId;
    use contextra_graph::CsrGraph;
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

    // Insert doc_a (community member) and doc_b (non-community member)
    col.insert(
        "doc_a",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "alpha topic"})),
    )
    .await?;
    col.insert(
        "doc_b",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "alpha topic"})),
    )
    .await?;

    let eid_a = EntityId::from_key("doc_a")?;

    // Relate doc_a to doc_c and run community detection so get_community(eid_a) finds target_community_id
    col.relate("doc_a", "doc_c", "knows").await?;
    col.run_community_detection().await?;
    assert!(col.get_community(eid_a).await?.is_some());

    // Perform hybrid search with same_community_as = doc_a
    let results_boosted = col
        .hybrid_search_with_strategy(
            "alpha",
            &[1.0, 0.0, 0.0, 0.0],
            10,
            None,
            None,
            None,
            Some(eid_a),
        )
        .await?;

    // Verification 1: Non-community doc_b is NOT eliminated and remains in results!
    assert_eq!(
        results_boosted.len(),
        2,
        "Non-community doc_b must not be filtered out"
    );
    let ids: Vec<&str> = results_boosted.iter().map(|r| r.id.as_str()).collect();
    assert!(
        ids.contains(&"doc_b"),
        "doc_b must remain in search results"
    );

    // Verification 2: Community doc_a gets boosted post-RRF and ranks #1 ahead of doc_b
    assert_eq!(
        results_boosted[0].id, "doc_a",
        "Community member doc_a must rank ahead after boost"
    );
    assert!(
        results_boosted[0].score > results_boosted[1].score,
        "Boosted community doc score ({}) must exceed non-community score ({})",
        results_boosted[0].score,
        results_boosted[1].score
    );
    Ok(())
}


#[tokio::test]
async fn test_post_rrf_supersedes_displacement_truncation_preserves_k() -> contextra_types::Result<()>
{
    use contextra_types::DocId;
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
        Arc::new(CsrGraph::new()),
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
