#![cfg(not(loom))]

use contextra_engine::Collection;
use contextra_graph::CsrGraph;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::Result;
use contextra_vector::{HnswConfig, HnswIndex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

async fn create_test_collection(
    name: &str,
) -> Result<(Arc<Collection<LsmStorage, HnswIndex>>, tempfile::TempDir)> {
    let dir = tempdir()?;
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await?);
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Arc::new(Collection::new(
        name.to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    ));

    Ok((col, dir))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_insert_and_delete_same_id_consistency() -> Result<()> {
    let (col, _dir) = create_test_collection("delete_race").await?;
    let doc_id = "concurrent_target_doc";

    let stop_flag = Arc::new(AtomicBool::new(false));
    let iterations = 200;

    // Task 1: Writer doing repeated inserts/upserts
    let col_writer = col.clone();
    let stop_writer = stop_flag.clone();
    let writer_handle = tokio::spawn(async move {
        for i in 0..iterations {
            let embedding = vec![i as f32, 0.0, 0.0, 0.0];
            let meta = serde_json::json!({ "iteration": i });
            let _ = col_writer.insert(doc_id, &embedding, Some(meta)).await;
            tokio::task::yield_now().await;
        }
        stop_writer.store(true, Ordering::SeqCst);
    });

    // Task 2: Deleter doing repeated deletes on the exact same ID
    let col_deleter = col.clone();
    let stop_deleter = stop_flag.clone();
    let deleter_handle = tokio::spawn(async move {
        while !stop_deleter.load(Ordering::SeqCst) {
            let _ = col_deleter.delete(doc_id).await;
            tokio::task::yield_now().await;
        }
    });

    // Task 3: Concurrent reader verifying state consistency
    let col_reader = col.clone();
    let stop_reader = stop_flag.clone();
    let reader_handle = tokio::spawn(async move {
        while !stop_reader.load(Ordering::SeqCst) {
            if let Ok(Some(doc)) = col_reader.get(doc_id).await {
                // Ensure document payload is valid and not corrupted
                assert_eq!(doc.id, doc_id);
                assert!(doc.metadata.is_some());
            }
            tokio::task::yield_now().await;
        }
    });

    let (r1, r2, r3) = tokio::join!(writer_handle, deleter_handle, reader_handle);
    r1.map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;
    r2.map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;
    r3.map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;

    // Final state must be consistent: get() returns either Some or None cleanly
    let final_get = col.get(doc_id).await?;
    if let Some(doc) = final_get {
        assert_eq!(doc.id, doc_id);
    }

    Ok(())
}

#[tokio::test]
async fn test_delete_non_existent_id_releases_lock() -> Result<()> {
    let (col, _dir) = create_test_collection("non_existent_delete").await?;
    let non_existent_id = "non_existent_doc_xyz";

    // Deleting a non-existent document should succeed cleanly or return without holding lock
    let res = col.delete(non_existent_id).await;
    assert!(res.is_ok(), "Deleting a non-existent document should succeed (no-op)");

    // Verify lock was released: subsequent insert on the exact same ID succeeds immediately
    let insert_res = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        col.insert(non_existent_id, &[1.0, 0.0, 0.0, 0.0], None),
    )
    .await;

    assert!(
        insert_res.is_ok(),
        "Subsequent insert must not hang/deadlock after deleting non-existent ID"
    );
    assert!(insert_res.unwrap().is_ok());

    // Verify subsequent delete on the newly inserted ID also succeeds and releases lock
    let delete_res = col.delete(non_existent_id).await;
    assert!(delete_res.is_ok());

    let final_get = col.get(non_existent_id).await?;
    assert!(final_get.is_none(), "Document must be deleted");

    Ok(())
}

// NOTE ON LOOM TESTS:
// Lower-level lock acquisition and deadlock-freedom for `KvKeyLocks` are tested under Loom in
// `crates/contextra-engine/src/collection/kv_lock.rs::loom_tests`. Higher-level `Collection`
// integrates Tokio Mutexes, LsmStorage, and HnswIndex which require full multi-threaded Tokio
// execution tested above in `test_concurrent_insert_and_delete_same_id_consistency`.
