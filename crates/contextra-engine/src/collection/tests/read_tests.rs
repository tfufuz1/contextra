use super::fixtures::*;

#[tokio::test]
async fn test_collection_scan_prefix_batches_via_mock_storage() {
    use contextra_core::{BoxFuture, Result, StorageEngine, StorageStats, TxId};
    use contextra_graph::csr::CsrGraph;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct BoundedScanMockStorage {
        bounded_call_count: AtomicUsize,
    }

    impl StorageEngine for BoundedScanMockStorage {
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
            Box::pin(async move { Ok(0) })
        }
        fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
            Box::pin(async move { Ok(TxId(0)) })
        }
        fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn scan_prefix<'a>(
            &'a self,
            _: &'a [u8],
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move {
                panic!("scan_prefix should not be called directly when batching!");
            })
        }
        fn scan_prefix_bounded<'a>(
            &'a self,
            _prefix: &'a [u8],
            limit: usize,
            cursor: Option<&'a [u8]>,
        ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
            Box::pin(async move {
                self.bounded_call_count.fetch_add(1, Ordering::SeqCst);
                let start = if let Some(cur) = cursor {
                    let s = String::from_utf8_lossy(cur);
                    let idx: usize = s["item_".len()..].parse().unwrap();
                    idx + 1
                } else {
                    0
                };

                let total_items = 5000;
                let end = (start + limit).min(total_items);

                let val_bytes = serde_json::to_vec(&serde_json::json!({"test": "data"})).unwrap();
                let mut batch = Vec::new();
                for i in start..end {
                    let k = format!("item_{:05}", i).into_bytes();
                    batch.push((k, val_bytes.clone()));
                }

                let next_cursor = if end < total_items {
                    batch.last().map(|(k, _)| k.clone())
                } else {
                    None
                };

                Ok((batch, next_cursor))
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

    let mock_storage = Arc::new(BoundedScanMockStorage {
        bounded_call_count: AtomicUsize::new(0),
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
        mock_storage.clone(),
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(std::sync::atomic::AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    let items = col.scan_prefix("item_", None).await.unwrap();
    assert_eq!(items.len(), 5000);
    // With 5000 items and BATCH_SIZE = 1000, scan_prefix_bounded should be called 5 times
    assert_eq!(mock_storage.bounded_call_count.load(Ordering::SeqCst), 5);
}


#[tokio::test]
async fn test_maintenance_pagination_over_10k_documents() {
    use contextra_core::EXPIRY_METADATA_KEY;
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use serde_json::json;
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

    let total_docs = 10_500;
    // Insert 10,500 synthetic documents
    for i in 0..total_docs {
        let key = format!("doc_{:05}", i);
        // Put expired TTL/sequence metadata on document at index 10,250
        if i == 10_250 {
            col.put_kv(
                &key,
                &json!({
                    "created_at_ms": 1_000_000,
                    "ttl_ms": 100,
                    EXPIRY_METADATA_KEY: 0
                }),
            )
            .await
            .unwrap();
        } else {
            col.put_kv(&key, &json!({ "v": i })).await.unwrap();
        }
    }

    let expired_key = "doc_10250";
    assert!(
        col.get_kv(expired_key).await.unwrap().is_some(),
        "Document past 10,000 threshold must exist before cleanup"
    );

    // Call reap_expired_documents and verify document at index 10,250 is reaped
    let reaped = col.reap_expired_documents(100).await.unwrap();
    assert_eq!(
        reaped, 1,
        "reap_expired_documents must find and reap the expired document at index > 10,000"
    );
    assert!(
        col.get_kv(expired_key).await.unwrap().is_none(),
        "Reaped document past 10,000 threshold must be deleted"
    );

    // Now re-insert document at 10,250 with wall-clock expired TTL and test trigger_expiry_cleanup
    col.put_kv(
        expired_key,
        &json!({
            "created_at_ms": 1_000_000,
            "ttl_ms": 100
        }),
    )
    .await
    .unwrap();

    let cleaned = col.trigger_expiry_cleanup().await.unwrap();
    assert_eq!(
        cleaned, 1,
        "trigger_expiry_cleanup must find and clean the expired document at index > 10,000"
    );
    assert!(
        col.get_kv(expired_key).await.unwrap().is_none(),
        "Cleaned document past 10,000 threshold must be deleted"
    );

    // Test evict_decayed_chunks with decayed importance at index 10,300
    let decay_key = "doc_10300";
    let decay_controller = crate::decay_controller::AdaptiveDecayController::with_defaults();

    let imp = contextra_core::MemoryImportance::new(
        contextra_core::ImportanceScore::new(1.0),
        contextra_core::DecayFunction::Exponential { half_life_tx: 10 },
        contextra_core::TxId::new(0),
    );

    col.insert(
        decay_key,
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({
            "importance": imp,
            "created_at_tx": 0
        })),
    )
    .await
    .unwrap();

    col.next_tx
        .store(1_000_000, std::sync::atomic::Ordering::SeqCst);

    let evicted = col
        .evict_decayed_chunks(&decay_controller, 100)
        .await
        .unwrap();
    assert_eq!(
        evicted, 1,
        "evict_decayed_chunks must find and evict the decayed document at index > 10,000"
    );
    assert!(
        col.get_kv(decay_key).await.unwrap().is_none(),
        "Evicted decayed document past 10,000 threshold must be deleted"
    );
}


#[tokio::test]
async fn test_migrate_doc_keys_v1() {
    use contextra_core::{DocId, StorageEngine, TxId};
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // unwrap allowed (AGENT:04)
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap allowed (AGENT:04)
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap allowed (AGENT:04)
    );
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        "default".to_string(),
        storage.clone(),
        index,
        Arc::new(CsrGraph::new()),
        next_tx.clone(),
        4,
        contextra_text::Language::English,
    );

    // Inject legacy doc_key (containing embedding in StoredDocument)
    let doc_id = DocId::from_key("legacy_doc_1").unwrap(); // unwrap allowed (AGENT:04)
    let doc_key = col.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
    let legacy_doc = crate::collection::StoredDocument {
        id: "legacy_doc_1".to_string(),
        embedding: vec![1.0, 0.0, 0.0, 0.0],
        metadata: Some(json!({"topic": "legacy"})),
    };
    let legacy_bytes = serde_json::to_vec(&legacy_doc).unwrap(); // unwrap allowed (AGENT:04)

    // Put user_key and legacy doc_key in storage
    let tx = TxId::new(next_tx.fetch_add(1, Ordering::SeqCst));
    let user_key = col.namespaced_key(b"legacy_doc_1", 0);
    storage.put(tx, &user_key, &legacy_bytes).await.unwrap(); // unwrap allowed (AGENT:04)
    storage.put(tx, &doc_key, &legacy_bytes).await.unwrap(); // unwrap allowed (AGENT:04)
    storage.commit(tx).await.unwrap(); // unwrap allowed (AGENT:04)

    // Verify doc_key currently contains full StoredDocument
    let raw_before = storage.get(&doc_key).await.unwrap().unwrap(); // unwrap allowed (AGENT:04)
    assert!(serde_json::from_slice::<crate::collection::StoredDocument>(&raw_before).is_ok());

    // Run migration
    let count = col.migrate_doc_keys_v1().await.unwrap(); // unwrap allowed (AGENT:04)
    assert_eq!(count, 1);

    // Verify doc_key now contains StoredDocumentMeta (and fails parsing as StoredDocument due to missing embedding)
    let raw_after = storage.get(&doc_key).await.unwrap().unwrap(); // unwrap allowed (AGENT:04)
    let meta: crate::collection::StoredDocumentMeta = serde_json::from_slice(&raw_after).unwrap(); // unwrap allowed (AGENT:04)
    assert_eq!(meta.id, "legacy_doc_1");
    assert_eq!(meta.metadata.unwrap()["topic"], "legacy"); // unwrap allowed (AGENT:04)
    assert!(serde_json::from_slice::<crate::collection::StoredDocument>(&raw_after).is_err());

    // Idempotency check: running migration again returns 0
    let count_again = col.migrate_doc_keys_v1().await.unwrap(); // unwrap allowed (AGENT:04)
    assert_eq!(count_again, 0);
}
