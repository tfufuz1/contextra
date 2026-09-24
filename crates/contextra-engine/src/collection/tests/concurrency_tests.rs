use super::fixtures::*;

#[tokio::test]
async fn test_insert_does_not_block_on_collection_wide_lock() -> contextra_types::Result<()> {
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
    let col = Arc::new(Collection::new(
        "parallel_locks".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    ));

    // Find two keys that map to different shards
    let _locks = crate::collection::kv_lock::KvKeyLocks::new();
    let key_a = "key_0".to_string();
    let mut key_b = "key_1".to_string();
    let mut i = 0;
    while col.kv_locks.shard_idx(&key_a) == col.kv_locks.shard_idx(&key_b) {
        i += 1;
        key_b = format!("key_{i}");
    }

    // Acquire lock on key_a's shard
    let guard_a = col.kv_locks.lock_for(&key_a).await;

    // Concurrent insert for key_b (different shard) must NOT block
    let col_clone = col.clone();
    let key_b_clone = key_b.clone();
    let handle = tokio::spawn(async move {
        col_clone
            .insert(&key_b_clone, &[1.0, 0.0, 0.0, 0.0], None)
            .await
    });

    let res = tokio::time::timeout(std::time::Duration::from_millis(500), handle).await;
    assert!(
        res.is_ok(),
        "Insert for key_b on different shard must not block when key_a is locked"
    );
    assert!(res.unwrap().unwrap().is_ok());

    drop(guard_a);
    Ok(())
}


#[tokio::test]
async fn test_batch_insert_deterministic_lock_order_no_deadlock() -> contextra_types::Result<()> {
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
    let col = Arc::new(Collection::new(
        "batch_order_no_deadlock".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    ));

    let k1 = "key_alpha".to_string();
    let k2 = "key_beta".to_string();
    let k3 = "key_gamma".to_string();

    let batch_1 = vec![
        (k3.clone(), vec![1.0, 0.0, 0.0, 0.0], None),
        (k1.clone(), vec![0.0, 1.0, 0.0, 0.0], None),
        (k2.clone(), vec![0.0, 0.0, 1.0, 0.0], None),
    ];

    let batch_2 = vec![
        (k2.clone(), vec![0.5, 0.0, 0.0, 0.0], None),
        (k3.clone(), vec![0.0, 0.5, 0.0, 0.0], None),
        (k1.clone(), vec![0.0, 0.0, 0.5, 0.0], None),
    ];

    let col_1 = col.clone();
    let h1 = tokio::spawn(async move { col_1.insert_many(&batch_1).await });

    let col_2 = col.clone();
    let h2 = tokio::spawn(async move { col_2.insert_many(&batch_2).await });

    let (r1, r2) = tokio::join!(h1, h2);
    assert!(r1.unwrap().is_ok());
    assert!(r2.unwrap().is_ok());

    Ok(())
}


#[tokio::test]
#[allow(deprecated)]
async fn test_collection_next_tx_sequence() {
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
        next_tx,
        4,
        contextra_text::Language::English,
    );

    let tx1 = col.next_tx().unwrap(); // unwrap allowed
    let tx2 = col.next_tx().unwrap(); // unwrap allowed
    let tx3 = col.next_tx().unwrap(); // unwrap allowed

    assert_eq!(tx1.inner(), 1);
    assert_eq!(tx2.inner(), 2);
    assert_eq!(tx3.inner(), 3);
}


#[tokio::test]
async fn test_collection_allocate_tx_sequence() {
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
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(100));

    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    let tx1 = col.allocate_tx().unwrap(); // unwrap allowed
    let tx2 = col.allocate_tx().unwrap(); // unwrap allowed
    let tx3 = col.allocate_tx().unwrap(); // unwrap allowed

    assert_eq!(tx1.inner(), 100);
    assert_eq!(tx2.inner(), 101);
    assert_eq!(tx3.inner(), 102);
}


#[tokio::test]
async fn test_concurrent_insert_and_write_ops_lock_safety() {
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
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Arc::new(Collection::new(
        "default".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    ));

    let mut handles = Vec::new();

    // Task 1: Single inserts
    {
        let c = col.clone();
        handles.push(tokio::spawn(async move {
            for i in 0..10 {
                let id = format!("single_doc_{i}");
                c.insert(&id, &[1.0, 0.0, 0.0, 0.0], None).await.unwrap(); // unwrap
            }
        }));
    }

    // Task 2: Insert many
    {
        let c = col.clone();
        handles.push(tokio::spawn(async move {
            let docs: Vec<_> = (0..5)
                .map(|i| (format!("batch_doc_{i}"), vec![0.0, 1.0, 0.0, 0.0], None))
                .collect();
            c.insert_many(&docs).await.unwrap(); // unwrap
        }));
    }

    // Task 3: Upsert & Update
    {
        let c = col.clone();
        handles.push(tokio::spawn(async move {
            for i in 0..5 {
                let id = format!("upsert_doc_{i}");
                c.upsert(&id, &[0.0, 0.0, 1.0, 0.0], None).await.unwrap(); // unwrap
                c.update(&id, &[0.0, 0.0, 1.0, 1.0], None).await.unwrap(); // unwrap
            }
        }));
    }

    // Task 4: Upsert many
    {
        let c = col.clone();
        handles.push(tokio::spawn(async move {
            let docs: Vec<_> = (0..5)
                .map(|i| (format!("upsert_batch_{i}"), vec![0.5, 0.5, 0.0, 0.0], None))
                .collect();
            c.upsert_many(&docs).await.unwrap(); // unwrap
        }));
    }

    for h in handles {
        h.await.unwrap(); // unwrap
    }

    assert!(col.len().await > 0);
}


#[tokio::test]
async fn test_concurrent_insert_many_collision_safety() {
    use contextra_types::{DocId, ContextraError, TxId};
use contextra_ports::{StorageEngine};
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
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
    let hnsw_config = contextra_vector::HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = Arc::new(HnswIndex::try_new(hnsw_config).unwrap()); // unwrap
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Arc::new(Collection::new(
        "default".to_string(),
        storage.clone(),
        index,
        graph,
        next_tx.clone(),
        4,
        contextra_text::Language::English,
    ));

    // 1. Parallel insert_many calls with overlapping document keys across tasks
    let mut tasks = Vec::new();
    for task_idx in 0..8 {
        let col_clone = col.clone();
        tasks.push(tokio::spawn(async move {
            let docs: Vec<(String, Vec<f32>, Option<serde_json::Value>)> = (0..20)
                .map(|i| {
                    let key = format!("batch_doc_{i}");
                    let val = (task_idx * 100 + i + 1) as f32;
                    (
                        key,
                        vec![val, 0.0, 0.0, 0.0],
                        Some(serde_json::json!({ "task": task_idx, "i": i })),
                    )
                })
                .collect();
            col_clone.insert_many(&docs).await.unwrap(); // unwrap
        }));
    }

    for task in tasks {
        task.await.unwrap(); // unwrap
    }

    // All 20 document keys must exist and be valid
    for i in 0..20 {
        let key = format!("batch_doc_{i}");
        let doc = col.get(&key).await.unwrap(); // unwrap
        assert!(
            doc.is_some(),
            "Document {key} must exist after concurrent insert_many"
        );
    }

    // 2. Synthetically test DocId collision rejection within insert_many
    // Seed an initial document key "existing_key"
    col.insert("existing_key", &[1.0, 0.0, 0.0, 0.0], None)
        .await
        .unwrap(); // unwrap

    // Map fixed synthetic DocId (e.g. 999) to "existing_key"
    let synthetic_doc_id = DocId::from_key("colliding_target_key").unwrap(); // unwrap
    let tx = TxId::new(next_tx.fetch_add(1, Ordering::SeqCst));
    let doc_key = col.namespaced_key(&synthetic_doc_id.inner().to_le_bytes(), 1);
    let existing_meta = crate::collection::StoredDocumentMeta {
        id: "existing_key".to_string(),
        metadata: None,
    };
    let meta_bytes = serde_json::to_vec(&existing_meta).unwrap(); // unwrap
    storage.put(tx, &doc_key, &meta_bytes).await.unwrap(); // unwrap
    storage.commit(tx).await.unwrap(); // unwrap

    // Attempt insert_many with a batch containing "colliding_target_key"
    let batch_with_collision = vec![
        ("safe_doc_1".to_string(), vec![1.0, 0.0, 0.0, 0.0], None),
        (
            "colliding_target_key".to_string(),
            vec![2.0, 0.0, 0.0, 0.0],
            None,
        ),
        ("safe_doc_2".to_string(), vec![3.0, 0.0, 0.0, 0.0], None),
    ];

    let err_res = col.insert_many(&batch_with_collision).await;
    assert!(
        err_res.is_err(),
        "insert_many must fail when DocId collision is detected"
    );
    assert!(matches!(err_res, Err(ContextraError::Internal(_))));

    // Verify all-or-nothing rollback (Option a): safe_doc_1 and safe_doc_2 must NOT exist
    assert!(
        col.get("safe_doc_1").await.unwrap().is_none(), // unwrap
        "safe_doc_1 must be rolled back on collision error in insert_many"
    );
    assert!(
        col.get("safe_doc_2").await.unwrap().is_none(), // unwrap
        "safe_doc_2 must be rolled back on collision error in insert_many"
    );
}


#[tokio::test]
async fn test_graph_mapping_invariant_missing_entity_graceful_degradation(
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

    // Insert text document "doc_text_only" without creating any graph entities
    col.insert(
        "doc_text_only",
        &[0.5, 0.5, 0.0, 0.0],
        Some(serde_json::json!({"text": "specialized retrieval architecture"})),
    )
    .await?;

    // Perform hybrid search where text signal finds "doc_text_only", but graph index has no node for it.
    // The graph signal will become empty, but the overall search must succeed using vector and text signals.
    let results = col
        .hybrid_search_with_strategy(
            "retrieval",
            &[0.5, 0.5, 0.0, 0.0],
            10,
            None,
            None,
            None,
            None,
        )
        .await?;

    assert!(
        !results.is_empty(),
        "Hybrid search must return results from remaining signals"
    );
    assert_eq!(results[0].id, "doc_text_only");
    Ok(())
}


#[tokio::test]
async fn test_begin_transaction_returns_active_db_transaction() {
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
        next_tx,
        4,
        contextra_text::Language::English,
    );

    let tx = col.begin_transaction();
    assert!(tx.is_ok());
}


#[tokio::test]
async fn test_apm3_lock_contention_fallback() -> contextra_types::Result<()> {
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
    let col = Arc::new(Collection::new(
        "apm3_lock".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    ));

    // Acquire key lock on "d_blocked"
    let lock_guard = col.kv_locks.lock_for("d_blocked").await;

    // Concurrent insert attempt while key lock is held
    let col_clone = col.clone();
    let handle = tokio::spawn(async move {
        col_clone
            .insert("d_blocked", &[1.0, 0.0, 0.0, 0.0], None)
            .await
    });

    // Release key lock and verify task completes cleanly
    drop(lock_guard);
    let res = handle.await.unwrap();
    assert!(
        res.is_ok(),
        "Insert task must complete cleanly after lock release"
    );

    Ok(())
}
