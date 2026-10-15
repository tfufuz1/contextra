// FILE-CONTEXT
// ZWECK: Unit-Tests für background_workers Submodule.

use super::*;
use contextra_mvcc::tx_buffer::IndexOp;
use contextra_types::{DocId, TxId};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[test]
fn test_calculate_rebuild_cooldown_exponential() {
    let base = Duration::from_secs(5);
    let max = Duration::from_secs(300);

    assert_eq!(calculate_rebuild_cooldown(0, base, max), Duration::ZERO);
    assert_eq!(
        calculate_rebuild_cooldown(1, base, max),
        Duration::from_secs(5)
    );
    assert_eq!(
        calculate_rebuild_cooldown(2, base, max),
        Duration::from_secs(10)
    );
    assert_eq!(
        calculate_rebuild_cooldown(3, base, max),
        Duration::from_secs(20)
    );
    assert_eq!(
        calculate_rebuild_cooldown(4, base, max),
        Duration::from_secs(40)
    );
    assert_eq!(
        calculate_rebuild_cooldown(10, base, max),
        Duration::from_secs(300)
    );
}

struct MockDegradedHnswIndex {
    connectivity_ok: std::sync::atomic::AtomicBool,
    rebuild_calls: Arc<AtomicU32>,
    rebuild_should_restore: std::sync::atomic::AtomicBool,
}

impl OrphanCleanupIndex for MockDegradedHnswIndex {
    fn check_connectivity(&self) -> contextra_types::Result<()> {
        if self.connectivity_ok.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(contextra_types::ContextraError::HnswConnectivityDegraded {
                deleted_ratio: 50.0,
            })
        }
    }

    fn rebuild(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = contextra_types::Result<()>> + Send + '_>>
    {
        let calls = self.rebuild_calls.clone();
        let should_restore = self.rebuild_should_restore.load(Ordering::SeqCst);
        Box::pin(async move {
            calls.fetch_add(1, Ordering::SeqCst);
            if should_restore {}
            Ok(())
        })
    }
}

#[tokio::test]
async fn test_orphan_cleanup_rebuild_backoff_and_alert() {
    use contextra_mvcc::tx_buffer::TxBuffer;

    let buffer = Arc::new(TxBuffer::<String>::new_with_config(
        64,
        Duration::from_millis(500),
    ));
    let cancel_token = tokio_util::sync::CancellationToken::new();

    let rebuild_calls = Arc::new(AtomicU32::new(0));
    let mock_index = Arc::new(MockDegradedHnswIndex {
        connectivity_ok: std::sync::atomic::AtomicBool::new(false),
        rebuild_calls: rebuild_calls.clone(),
        rebuild_should_restore: std::sync::atomic::AtomicBool::new(false),
    });

    let backoff_config = OrphanCleanupBackoffConfig {
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_millis(1000),
        alert_threshold: 3,
    };

    let (handle, failures_counter) = start_orphan_cleanup_worker_with_config::<_, _, contextra_store::LsmStorage>(
        buffer,
        mock_index.clone(),
        None,
        Duration::from_millis(10),
        backoff_config,
        cancel_token.clone(),
    );

    sleep(Duration::from_millis(40)).await;
    assert_eq!(
        rebuild_calls.load(Ordering::SeqCst),
        1,
        "First rebuild attempt should fire immediately"
    );
    assert_eq!(
        failures_counter.load(Ordering::SeqCst),
        1,
        "First failed rebuild incremented failure counter"
    );

    sleep(Duration::from_millis(50)).await;
    assert_eq!(
        rebuild_calls.load(Ordering::SeqCst),
        1,
        "Backoff must prevent rebuild on subsequent ticks during cooldown"
    );

    sleep(Duration::from_millis(80)).await;
    assert_eq!(
        rebuild_calls.load(Ordering::SeqCst),
        2,
        "Second rebuild attempt should fire after 100ms cooldown"
    );
    assert_eq!(
        failures_counter.load(Ordering::SeqCst),
        2,
        "Second failed rebuild incremented failure counter"
    );

    sleep(Duration::from_millis(220)).await;
    assert_eq!(
        rebuild_calls.load(Ordering::SeqCst),
        3,
        "Third rebuild attempt should fire after 200ms cooldown"
    );
    assert_eq!(
        failures_counter.load(Ordering::SeqCst),
        3,
        "Failure counter should reach alert threshold 3"
    );

    mock_index
        .rebuild_should_restore
        .store(true, Ordering::SeqCst);
    mock_index.connectivity_ok.store(true, Ordering::SeqCst);

    sleep(Duration::from_millis(50)).await;
    assert_eq!(
        failures_counter.load(Ordering::SeqCst),
        0,
        "Healthy connectivity must reset failure counter to 0"
    );

    cancel_token.cancel();
    let _ = handle.await;
}

#[tokio::test]
async fn test_expiry_cleanup_worker_task_cleans_documents() {
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
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
    let col = Arc::new(crate::Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    ));

    let vec = vec![1.0, 0.0, 0.0, 0.0];
    col.insert_with_ttl("doc_task_ttl", &vec, None, 2)
        .await
        .unwrap();

    col.insert("d1", &vec, None).await.unwrap();
    col.insert("d2", &vec, None).await.unwrap();

    let cancel_token = tokio_util::sync::CancellationToken::new();
    let handle =
        start_expiry_cleanup_worker(col.clone(), Duration::from_millis(10), cancel_token.clone());

    let mut cleaned = false;
    for _ in 0..50 {
        sleep(Duration::from_millis(10)).await;
        if col.get("doc_task_ttl").await.unwrap().is_none() {
            cleaned = true;
            break;
        }
    }

    cancel_token.cancel();
    let _ = handle.await;

    assert!(
        cleaned,
        "Expiry cleanup worker task should delete expired document"
    );
}

#[tokio::test]
async fn test_orphan_cleanup_worker_removes_expired() {
    use contextra_mvcc::tx_buffer::TxBuffer;

    let buffer = Arc::new(TxBuffer::<String>::new_with_config(
        64,
        Duration::from_millis(50),
    ));
    let tx1 = TxId::new(1);

    buffer.begin(tx1);
    let _ = buffer.stage(
        tx1,
        IndexOp::Insert {
            doc_id: DocId::new(1),
            data: "old".to_string(),
        },
    );

    let cancel_token = tokio_util::sync::CancellationToken::new();
    let config = contextra_vector::hnsw::HnswConfig::default();
    let hnsw_index = Arc::new(contextra_vector::hnsw::HnswIndex::try_new(config).unwrap());
    let _worker = start_orphan_cleanup_worker(
        buffer.clone(),
        hnsw_index.clone(),
        Duration::from_millis(10),
        cancel_token.clone(),
    );
    assert!(buffer.has_tx(tx1));

    let mut removed = false;
    for _ in 0..50 {
        sleep(Duration::from_millis(10)).await;
        if !buffer.has_tx(tx1) {
            removed = true;
            break;
        }
    }
    cancel_token.cancel();
    assert!(
        removed,
        "Expired transaction should have been cleaned up within 500ms"
    );
}

#[tokio::test]
async fn trigger_expiry_cleanup_deletes_expired_documents() {
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use serde_json::json;
    use std::sync::atomic::AtomicU64;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let lsm_config = contextra_store::LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap());
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(),
    );
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = crate::Collection::new(
        "default".to_string(),
        storage,
        index,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    let vec = vec![1.0, 0.0, 0.0, 0.0];
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    col.insert(
        "doc1",
        &vec,
        Some(json!({"created_at_ms": now_ms - 100, "ttl_ms": 50})),
    )
    .await
    .unwrap();

    col.trigger_expiry_cleanup().await.unwrap();
    let result = col.get("doc1").await.unwrap();
    assert!(result.is_none(), "Expired document must be deleted");
}

#[tokio::test]
async fn test_worker_immediate_cancellation() {
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
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
    let col = Arc::new(crate::Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    ));

    let cancel_token = tokio_util::sync::CancellationToken::new();
    cancel_token.cancel();

    let handle = start_expiry_cleanup_worker(col, Duration::from_secs(60), cancel_token);
    let res = handle.await;
    assert!(res.is_ok(), "Task should exit cleanly upon cancellation");
}
