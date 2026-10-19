// FILE-CONTEXT
// ZWECK: Worker-Task und FIFO-Queue zur verzögerten Kaskaden-Tombstoning-Verarbeitung von Hyperedges.
// INVARIANTEN: Geordnete Entnahme und Beschränkung der pro Tick verarbeiteten Hyperedges.

use crate::collection::Collection;
use contextra_graph::hyperedge::HyperEdgeId;
use contextra_ports::StorageEngine;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Maximum number of deferred hyperedges processed in a single worker tick.
pub const MAX_DEFERRED_HYPEREDGES_PER_TICK: usize = 1_000;

/// Thread-safe FIFO queue for storing hyperedges whose cascade invalidation was
/// deferred to the background worker due to fan-out limits (H5 / AK-6).
///
/// DISCONNECT NOTE: In `contextra-graph` (`crates/contextra-graph/src/cascade.rs:331`),
/// overflow hyperedges are enqueued directly into `CsrGraph::cascade_queue` via
/// `graph.enqueue_cascade_deferred(superseded_doc_id, &deferred)`.
/// `DeferredHyperedgeQueue` provides an in-memory staging queue for engine-level producers.
#[derive(Debug, Default)]
pub struct DeferredHyperedgeQueue {
    inner: Mutex<VecDeque<HyperEdgeId>>,
}

impl DeferredHyperedgeQueue {
    /// Creates a new empty `DeferredHyperedgeQueue`.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(VecDeque::new()),
        }
    }

    /// Enqueues hyperedge IDs for deferred background tombstoning.
    ///
    /// Note: Producer gap in `crates/contextra-graph/src/cascade.rs:331` enqueues overflow
    /// into `CsrGraph::cascade_queue`. This method allows engine-level producers or test
    /// harnesses to enqueue items directly into this queue.
    pub async fn enqueue(&self, ids: impl IntoIterator<Item = HyperEdgeId>) {
        let mut guard = self.inner.lock().await;
        guard.extend(ids);
    }

    /// Drains up to `max` hyperedge IDs from the front of the queue (FIFO).
    pub async fn drain_up_to(&self, max: usize) -> Vec<HyperEdgeId> {
        let mut guard = self.inner.lock().await;
        let drain_count = max.min(guard.len());
        guard.drain(..drain_count).collect()
    }

    /// Returns the number of items currently pending in the queue.
    pub async fn len(&self) -> usize {
        let guard = self.inner.lock().await;
        guard.len()
    }

    /// Returns true if the queue contains no items.
    pub async fn is_empty(&self) -> bool {
        let guard = self.inner.lock().await;
        guard.is_empty()
    }
}

/// Starts a background task to process deferred hyperedge tombstones.
///
/// Processes both items explicitly enqueued in `queue` (`DeferredHyperedgeQueue`) AND
/// pending items in `collection.graph_index()`'s internal cascade queue
/// (`CsrGraph::process_cascade_queue`).
pub fn start_hyperedge_cascade_deferred_worker<S: StorageEngine>(
    collection: Arc<Collection<S>>,
    queue: Arc<DeferredHyperedgeQueue>,
    interval: Duration,
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        tracing::info!(
            collection = %collection.name(),
            interval = ?interval,
            "Deferred hyperedge cascade worker task started"
        );

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    // 1. Drain items from DeferredHyperedgeQueue
                    let batch = queue.drain_up_to(MAX_DEFERRED_HYPEREDGES_PER_TICK).await;
                    if !batch.is_empty() {
                        let graph = collection.graph_index();
                        let tx = match collection.allocate_tx() {
                            Ok(t) => t,
                            Err(err) => {
                                tracing::error!(
                                    collection = %collection.name(),
                                    error = %err,
                                    "Deferred hyperedge worker failed to allocate TxId for queue drain"
                                );
                                continue;
                            }
                        };
                        let mut tombstoned_count = 0usize;
                        for id in batch {
                            if graph.tombstone_hyperedge(id, tx) {
                                tombstoned_count += 1;
                            }
                        }
                        tracing::info!(
                            collection = %collection.name(),
                            processed = tombstoned_count,
                            "Deferred hyperedge worker tombstoned enqueued hyperedges"
                        );
                    }

                    // 2. Process pending items in CsrGraph's internal cascade_queue
                    let graph = collection.graph_index();
                    if graph.pending_cascade_queue_len() > 0 {
                        let tx = match collection.allocate_tx() {
                            Ok(t) => t,
                            Err(err) => {
                                tracing::error!(
                                    collection = %collection.name(),
                                    error = %err,
                                    "Deferred hyperedge worker failed to allocate TxId for CsrGraph cascade_queue"
                                );
                                continue;
                            }
                        };
                        match graph.process_cascade_queue(MAX_DEFERRED_HYPEREDGES_PER_TICK, tx).await {
                            Ok(count) => {
                                if count > 0 {
                                    tracing::info!(
                                        collection = %collection.name(),
                                        processed = count,
                                        "Deferred hyperedge worker processed CsrGraph cascade_queue items"
                                    );
                                }
                            }
                            Err(err) => {
                                tracing::error!(
                                    collection = %collection.name(),
                                    error = %err,
                                    "Deferred hyperedge worker error processing CsrGraph cascade_queue"
                                );
                            }
                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    tracing::info!(
                        collection = %collection.name(),
                        "Deferred hyperedge worker task shutting down via token"
                    );
                    break;
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_deferred_hyperedge_queue_len_and_drain() {
        let queue = DeferredHyperedgeQueue::new();
        assert!(queue.is_empty().await);

        queue
            .enqueue(vec![
                HyperEdgeId::new(100),
                HyperEdgeId::new(200),
                HyperEdgeId::new(300),
            ])
            .await;

        assert_eq!(queue.len().await, 3);
        assert!(!queue.is_empty().await);

        let drained = queue.drain_up_to(2).await;
        assert_eq!(drained, vec![HyperEdgeId::new(100), HyperEdgeId::new(200)]);
        assert_eq!(queue.len().await, 1);

        let drained_remaining = queue.drain_up_to(10).await;
        assert_eq!(drained_remaining, vec![HyperEdgeId::new(300)]);
        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_worker_processes_queue_and_csrgraph_and_cancels() {
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
        let col = Arc::new(Collection::new(
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
            tokio::time::sleep(Duration::from_millis(10)).await;
            if queue.is_empty().await {
                processed = true;
                break;
            }
        }

        cancel_token.cancel();
        let join_res = handle.await;
        assert!(join_res.is_ok(), "Worker task should cancel cleanly");
        assert!(processed, "Worker task should process queued items");
    }

    #[tokio::test]
    async fn test_worker_cancellation_preserves_unprocessed_items() {
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
        let col = Arc::new(Collection::new(
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
            .enqueue(vec![HyperEdgeId::new(55), HyperEdgeId::new(66)])
            .await;

        let cancel_token = tokio_util::sync::CancellationToken::new();
        cancel_token.cancel();

        let handle = start_hyperedge_cascade_deferred_worker(
            col.clone(),
            queue.clone(),
            Duration::from_secs(60),
            cancel_token.clone(),
        );

        let join_res = handle.await;
        assert!(join_res.is_ok(), "Immediate cancellation should exit cleanly");
        assert_eq!(
            queue.len().await,
            2,
            "Immediate cancellation must leave queue items intact"
        );
    }
}
