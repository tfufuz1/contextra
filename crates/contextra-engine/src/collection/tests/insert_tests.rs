use super::fixtures::*;

#[tokio::test]
async fn test_insert_with_ttl_and_reap_expired_documents() {
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

    // Insert document with TTL = 5 committed ops
    col.insert_with_ttl("temp_doc", &vec, None, 5)
        .await
        .unwrap(); // unwrap

    // 1. Immediately after insert, document should be retrievable
    let doc = col.get("temp_doc").await.unwrap(); // unwrap
    assert!(doc.is_some(), "Document must exist before TTL expiration");

    // 2. Perform 5 dummy commits (inserts)
    for i in 0..5 {
        col.insert(&format!("dummy_{i}"), &vec, None).await.unwrap(); // unwrap
    }

    // 3. Trigger expiry cleanup
    let reaped = col.reap_expired_documents(100).await.unwrap(); // unwrap
    assert_eq!(reaped, 1, "Expired document should be reaped");

    // 4. Verify document is gone from storage and search
    let doc_after = col.get("temp_doc").await.unwrap(); // unwrap
    assert!(
        doc_after.is_none(),
        "Document must be deleted after TTL expiry"
    );

    let search_res = col.search(&vec, 10).await.unwrap(); // unwrap
    assert!(
        search_res.iter().all(|r| r.id != "temp_doc"),
        "Expired document must not appear in search results"
    );
}


#[tokio::test]
async fn test_insert_typed_episodic_has_decay_metadata() {
    use contextra_graph::CsrGraph;
    use contextra_store::{LsmConfig, LsmStorage};
    use contextra_vector::HnswIndex;
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
    let graph_index = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        "test".to_string(),
        storage,
        index,
        graph_index,
        next_tx,
        4,
        contextra_text::Language::German,
    );

    col.insert_typed(
        "ep1",
        &[1.0, 0.0, 0.0, 0.0],
        contextra_core::MemoryType::Episodic,
        None,
    )
    .await
    .unwrap(); // unwrap

    let doc = col.get("ep1").await.unwrap().unwrap(); // unwrap
    let meta = doc.metadata.unwrap(); // unwrap
    assert_eq!(meta.get("memory_type").unwrap(), "episodic"); // unwrap
    assert!(meta.get("decay_function").is_some());
}


#[tokio::test]
async fn test_insert_typed_working_has_ttl_metadata() {
    use contextra_graph::CsrGraph;
    use contextra_store::{LsmConfig, LsmStorage};
    use contextra_vector::HnswIndex;
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
    let graph_index = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        "test".to_string(),
        storage,
        index,
        graph_index,
        next_tx,
        4,
        contextra_text::Language::German,
    );

    col.insert_typed(
        "wk1",
        &[1.0, 0.0, 0.0, 0.0],
        contextra_core::MemoryType::Working,
        None,
    )
    .await
    .unwrap(); // unwrap

    let doc = col.get("wk1").await.unwrap().unwrap(); // unwrap
    let meta = doc.metadata.unwrap(); // unwrap
    assert_eq!(meta.get("memory_type").unwrap(), "working"); // unwrap
    assert_eq!(meta.get("ttl_tx").unwrap(), 50_000); // unwrap
}


#[tokio::test]
async fn test_insert_backward_compatible_has_semantic_default() {
    use contextra_graph::CsrGraph;
    use contextra_store::{LsmConfig, LsmStorage};
    use contextra_vector::HnswIndex;
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
    let graph_index = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        "test".to_string(),
        storage,
        index,
        graph_index,
        next_tx,
        4,
        contextra_text::Language::German,
    );

    col.insert(
        "plain1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "hello"})),
    )
    .await
    .unwrap(); // unwrap

    let doc = col.get("plain1").await.unwrap().unwrap(); // unwrap
    assert_eq!(
        crate::filter::extract_memory_type(&doc.metadata),
        contextra_core::MemoryType::Semantic
    );
}


#[tokio::test]
async fn test_put_kv_if_absent_rollback_failure_returns_conflict_error() {
    use contextra_core::{BoxFuture, Result, StorageEngine, StorageStats, TxId};
    use contextra_graph::csr::CsrGraph;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    struct FailingRollbackMockStorage;

    impl StorageEngine for FailingRollbackMockStorage {
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
        fn put_if_absent<'a>(
            &'a self,
            _: TxId,
            _: &'a [u8],
            _: &'a [u8],
        ) -> BoxFuture<'a, Result<bool>> {
            Box::pin(async move { Ok(false) })
        }
        fn delete<'a>(&'a self, _: TxId, _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                Err(contextra_core::ContextraError::Internal(
                    "Simulated rollback failure".into(),
                ))
            })
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
            Box::pin(async move { Ok(0) })
        }
        fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
            Box::pin(async move { Ok(TxId::new(0)) })
        }
        fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn scan<'a>(
            &'a self,
            _: std::ops::Bound<&'a [u8]>,
            _: std::ops::Bound<&'a [u8]>,
            _: Option<usize>,
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(vec![]) })
        }
        fn scan_prefix<'a>(
            &'a self,
            _: &'a [u8],
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(vec![]) })
        }
        fn scan_prefix_bounded<'a>(
            &'a self,
            _: &'a [u8],
            _: usize,
            _: Option<&'a [u8]>,
        ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
            Box::pin(async move { Ok((vec![], None)) })
        }
    }

    let storage = Arc::new(FailingRollbackMockStorage);
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(),
    );
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
        .put_kv_if_absent("existing_key", &serde_json::json!({"test": "data"}))
        .await;

    assert!(res.is_err(), "put_kv_if_absent must fail");
    match res.unwrap_err() {
        contextra_core::ContextraError::Conflict(msg) => {
            assert!(
                msg.contains("existing_key"),
                "Conflict message must reference key, got: {}",
                msg
            );
        }
        other => panic!("Expected ContextraError::Conflict, got: {:?}", other),
    }
}
