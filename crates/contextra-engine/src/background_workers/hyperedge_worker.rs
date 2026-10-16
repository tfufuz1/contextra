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
}

/// Starts a background task to process deferred hyperedge tombstones.
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
                    let batch = queue.drain_up_to(MAX_DEFERRED_HYPEREDGES_PER_TICK).await;
                    if !batch.is_empty() {
                        let graph = collection.graph_index();
                        let tx = match collection.allocate_tx() {
                            Ok(t) => t,
                            Err(err) => {
                                tracing::error!(
                                    collection = %collection.name(),
                                    error = %err,
                                    "Deferred hyperedge worker failed to allocate TxId"
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
                            "Deferred hyperedge worker tombstoned hyperedges"
                        );
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
