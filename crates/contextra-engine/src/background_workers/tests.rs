// FILE-CONTEXT
// ZWECK: Unit-Tests für background_workers Submodule.

use super::*;
use contextra_graph::hyperedge::HyperEdgeId;
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
async fn test_deferred_hyperedge_queue_fifo_and_limits() {
    let queue = DeferredHyperedgeQueue::new();

    let empty = queue.drain_up_to(100).await;
    assert!(empty.is_empty());

    queue
        .enqueue(vec![
            HyperEdgeId::new(10),
            HyperEdgeId::new(20),
            HyperEdgeId::new(30),
        ])
        .await;

    let drained_part = queue.drain_up_to(2).await;
    assert_eq!(
        drained_part,
        vec![HyperEdgeId::new(10), HyperEdgeId::new(20)]
    );

    let drained_all = queue.drain_up_to(100).await;
    assert_eq!(drained_all, vec![HyperEdgeId::new(30)]);

    let empty_again = queue.drain_up_to(10).await;
    assert!(empty_again.is_empty());
}

#[tokio::test]
async fn test_deferred_hyperedge_worker_single_tick_processing() {
    use contextra_graph::CsrGraph;
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
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    ));

    let queue = Arc::new(DeferredHyperedgeQueue::new());
    queue
        .enqueue(vec![HyperEdgeId::new(1), HyperEdgeId::new(2)])
        .await;

    let cancel_token = tokio_util::sync::CancellationToken::new();
    let handle = start_hyperedge_cascade_deferred_worker(
        col.clone(),
        queue.clone(),
        Duration::from_millis(10),
        cancel_token.clone(),
    );

    let mut processed = false;
    for _ in 0..50 {
        sleep(Duration::from_millis(10)).await;
        if queue.drain_up_to(1).await.is_empty() {
            processed = true;
            break;
        }
    }

    cancel_token.cancel();
    let _ = handle.await;

    assert!(processed, "Worker should process queued items in tick");
}

#[tokio::test]
async fn test_deferred_hyperedge_worker_fanout_multitick_processing() {
    use contextra_graph::CsrGraph;
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
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    ));

    let queue = Arc::new(DeferredHyperedgeQueue::new());

    let total_items = 2_500;
    let items: Vec<HyperEdgeId> = (1..=total_items).map(HyperEdgeId::new).collect();
    queue.enqueue(items).await;

    let cancel_token = tokio_util::sync::CancellationToken::new();
    let handle = start_hyperedge_cascade_deferred_worker(
        col.clone(),
        queue.clone(),
        Duration::from_millis(10),
        cancel_token.clone(),
    );

    let mut fully_drained = false;
    for _ in 0..100 {
        sleep(Duration::from_millis(15)).await;
        if queue.drain_up_to(1).await.is_empty() {
            fully_drained = true;
            break;
        }
    }

    cancel_token.cancel();
    let _ = handle.await;

    assert!(
        fully_drained,
        "Worker should drain all items across multiple ticks without item loss"
    );
}

#[tokio::test]
async fn test_deferred_hyperedge_worker_graceful_shutdown_preserves_queue() {
    use contextra_graph::CsrGraph;
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
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    ));

    let queue = Arc::new(DeferredHyperedgeQueue::new());
    queue
        .enqueue(vec![
            HyperEdgeId::new(100),
            HyperEdgeId::new(200),
            HyperEdgeId::new(300),
        ])
        .await;

    let cancel_token = tokio_util::sync::CancellationToken::new();
    cancel_token.cancel();

    let handle = start_hyperedge_cascade_deferred_worker(
        col.clone(),
        queue.clone(),
        Duration::from_secs(60),
        cancel_token.clone(),
    );

    let res = handle.await;
    assert!(res.is_ok(), "Task should exit cleanly upon cancellation");

    let remaining = queue.drain_up_to(100).await;
    assert_eq!(
        remaining,
        vec![
            HyperEdgeId::new(100),
            HyperEdgeId::new(200),
            HyperEdgeId::new(300)
        ],
        "Cancelled worker must not lose unhandled elements in queue"
    );
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

    let (handle, failures_counter) = start_orphan_cleanup_worker_with_config(
        buffer,
        mock_index.clone(),
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
    use contextra_graph::CsrGraph;
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
        Arc::new(CsrGraph::new()),
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
    use contextra_graph::CsrGraph;
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
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = crate::Collection::new(
        "default".to_string(),
        storage,
        index,
        graph,
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
    use contextra_graph::CsrGraph;
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
        Arc::new(CsrGraph::new()),
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

#[tokio::test]
async fn test_decay_eviction_thresholds() {
    use crate::decay_controller::{AdaptiveDecayController, DecayControllerConfig};
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_types::{DecayFunction, ImportanceScore, MemoryImportance, TxId};
    use contextra_vector::HnswIndex;
    use serde_json::json;
    use std::sync::atomic::Ordering;
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
    let next_tx = Arc::new(std::sync::atomic::AtomicU64::new(1));

    let col = Arc::new(crate::Collection::new(
        "default".to_string(),
        storage,
        index.clone(),
        Arc::new(CsrGraph::new()),
        next_tx.clone(),
        4,
        contextra_text::Language::English,
    ));

    let vec = vec![1.0, 0.0, 0.0, 0.0];

    for i in 0..10 {
        let id = format!("old_low_{i}");
        let imp = MemoryImportance::new(
            ImportanceScore::new(0.02),
            DecayFunction::Exponential { half_life_tx: 100 },
            TxId::new(1),
        );
        col.insert(&id, &vec, Some(json!({ "importance": imp })))
            .await
            .unwrap();
    }

    for i in 0..5 {
        let id = format!("fresh_high_{i}");
        let imp = MemoryImportance::new(
            ImportanceScore::new(0.95),
            DecayFunction::Exponential {
                half_life_tx: 100_000,
            },
            TxId::new(100_000),
        );
        col.insert(&id, &vec, Some(json!({ "importance": imp })))
            .await
            .unwrap();
    }

    next_tx.store(50_000, Ordering::SeqCst);

    let decay_controller = AdaptiveDecayController::new(DecayControllerConfig {
        kappa: 2.0,
        base_half_life_tx: 1_000,
        eviction_threshold: 0.01,
    });

    let evicted = col
        .evict_decayed_chunks(&decay_controller, 100)
        .await
        .unwrap();
    assert_eq!(evicted, 10, "All 10 old low-score chunks should be evicted");

    for i in 0..10 {
        let res = col.get(&format!("old_low_{i}")).await.unwrap();
        assert!(res.is_none(), "old_low_{i} must be deleted");
    }

    for i in 0..5 {
        let res = col.get(&format!("fresh_high_{i}")).await.unwrap();
        assert!(res.is_some(), "fresh_high_{i} must remain");
    }
}

#[cfg(feature = "background-maintenance")]
#[tokio::test]
async fn test_start_decay_cleanup_worker_background_task() {
    use contextra_adapt::DecayControllerConfig;
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_types::{DecayFunction, ImportanceScore, MemoryImportance, TxId};
    use contextra_vector::HnswIndex;
    use serde_json::json;
    use std::sync::atomic::Ordering;
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
    let next_tx = Arc::new(std::sync::atomic::AtomicU64::new(1));

    let col = Arc::new(crate::Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        next_tx.clone(),
        4,
        contextra_text::Language::English,
    ));

    let vec = vec![1.0, 0.0, 0.0, 0.0];
    let imp = MemoryImportance::new(
        ImportanceScore::new(0.01),
        DecayFunction::Exponential { half_life_tx: 10 },
        TxId::new(1),
    );
    col.insert("decay_target", &vec, Some(json!({ "importance": imp })))
        .await
        .unwrap();

    next_tx.store(100_000, Ordering::SeqCst);

    let cancel_token = tokio_util::sync::CancellationToken::new();
    let handle = start_decay_cleanup_worker(
        col.clone(),
        DecayControllerConfig::default(),
        Duration::from_millis(10),
        cancel_token.clone(),
    );

    let mut evicted = false;
    for _ in 0..50 {
        sleep(Duration::from_millis(10)).await;
        if col.get("decay_target").await.unwrap().is_none() {
            evicted = true;
            break;
        }
    }

    cancel_token.cancel();
    let _ = handle.await;

    assert!(
        evicted,
        "Decay controller worker task should evict low score document"
    );
}
