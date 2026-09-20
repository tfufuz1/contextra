// FILE-CONTEXT
// ZWECK: Hintergrund-Worker-Tasks zur TTL-Löschung, Entropie-Pruning und Bereinigung verwaister Transaktionen (Orphan Cleanup).
// INVARIANTEN: Geordnete Abschaltung via CancellationToken; Beschränkung der pro Tick verarbeiteten Elemente.
// NICHT-OFFENSICHTLICH: Orphan Cleanup Worker triggert bei HNSW-Indextrennung automatischen Rebuild mit Timeout.
// STAND: TS:2026-08-29T17:22:29Z (SESSION: 0dcb9f3b)

use crate::collection::Collection;
use memfuse_core::traits::StorageEngine;
use memfuse_core::tx_buffer::TxBuffer;
#[cfg(feature = "background-maintenance")]
use memfuse_core::VectorIndex;
use memfuse_graph::hyperedge::HyperEdgeId;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Configuration parameters for HNSW index rebuild backoff and failure escalation.
#[derive(Debug, Clone, Copy)]
pub struct OrphanCleanupBackoffConfig {
    /// Initial cooldown duration after the first failed rebuild attempt.
    pub base_delay: Duration,
    /// Maximum cooldown cap between rebuild attempts.
    pub max_delay: Duration,
    /// Number of consecutive rebuild failures triggering a structural problem alert log.
    pub alert_threshold: u32,
}

impl Default for OrphanCleanupBackoffConfig {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_secs(5),
            max_delay: Duration::from_secs(300),
            alert_threshold: 3,
        }
    }
}

/// Helper calculating exponential backoff cooldown given consecutive failures.
pub fn calculate_rebuild_cooldown(
    consecutive_failures: u32,
    base_delay: Duration,
    max_delay: Duration,
) -> Duration {
    if consecutive_failures == 0 {
        Duration::ZERO
    } else {
        let shift = consecutive_failures.saturating_sub(1).min(30);
        let multiplier = 1u64.checked_shl(shift).unwrap_or(u64::MAX);
        let calculated = base_delay.saturating_mul(multiplier as u32);
        calculated.min(max_delay)
    }
}

/// Abstraction trait over vector indexes capable of connectivity check and rebuild.
pub trait OrphanCleanupIndex: Send + Sync {
    fn check_connectivity(&self) -> memfuse_core::Result<()>;
    fn rebuild(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = memfuse_core::Result<()>> + Send + '_>>;
}

impl OrphanCleanupIndex for memfuse_index::hnsw::HnswIndex {
    fn check_connectivity(&self) -> memfuse_core::Result<()> {
        self.check_connectivity()
    }

    fn rebuild(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = memfuse_core::Result<()>> + Send + '_>>
    {
        Box::pin(async move { self.rebuild().await })
    }
}

#[cfg(feature = "background-maintenance")]
use memfuse_adapt::{AdaptiveDecayController, DecayControllerConfig};

/// Maximum number of orphan transactions processed in a single worker tick
/// to avoid starving foreground operations.
pub const MAX_ORPHANS_PER_TICK: usize = 100;

/// Maximum number of expired documents processed in a single expiry cleanup tick.
pub const MAX_EXPIRED_PER_TICK: usize = 100;

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

/// Starts a background task to periodically clean up expired documents with TTL.
pub fn start_expiry_cleanup_worker<S: StorageEngine>(
    collection: Arc<Collection<S>>,
    interval: Duration,
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        tracing::info!(
            collection = %collection.name(),
            interval = ?interval,
            "Expiry cleanup worker task started"
        );
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    match collection.reap_expired_documents(MAX_EXPIRED_PER_TICK).await {
                        Ok(reaped) if reaped > 0 => {
                            tracing::info!(
                                collection = %collection.name(),
                                reaped = reaped,
                                "Expiry cleanup worker cleaned up expired documents"
                            );
                        }
                        Ok(_) => {}
                        Err(err) => {
                            tracing::error!(
                                collection = %collection.name(),
                                error = %err,
                                "Error during expiry cleanup worker execution"
                            );
                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    tracing::info!(
                        collection = %collection.name(),
                        "Expiry cleanup worker task shutting down via token"
                    );
                    break;
                }
            }
        }
    })
}

/// Deprecated legacy alias for `start_expiry_cleanup_worker`.
#[deprecated(note = "use start_expiry_cleanup_worker instead")]
pub fn start_expiry_reaper<S: StorageEngine>(
    collection: Arc<Collection<S>>,
    interval: Duration,
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    start_expiry_cleanup_worker(collection, interval, cancel_token)
}

/// Starts a background task for decay-controller-driven importance-score eviction.
/// Nur aktiv wenn `background-maintenance` Feature-Flag gesetzt.
#[cfg(feature = "background-maintenance")]
pub fn start_decay_cleanup_worker<S: StorageEngine, V: VectorIndex>(
    collection: Arc<Collection<S, V>>,
    decay_config: DecayControllerConfig,
    interval: Duration,
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    let decay_controller = AdaptiveDecayController::new(decay_config);
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    match collection.evict_decayed_chunks(&decay_controller, 100).await {
                        Ok(n) if n > 0 => tracing::info!(evicted = n, "Decay controller worker evicted chunks"),
                        Ok(_) => {},
                        Err(e) => tracing::error!(error = %e, "Decay controller worker error"),
                    }
                }
                _ = cancel_token.cancelled() => break,
            }
        }
    })
}

/// Deprecated legacy alias for `start_decay_cleanup_worker`.
#[cfg(feature = "background-maintenance")]
#[deprecated(note = "use start_decay_cleanup_worker instead")]
pub fn start_thermostat_reaper<S: StorageEngine, V: VectorIndex>(
    collection: Arc<Collection<S, V>>,
    decay_config: DecayControllerConfig,
    interval: Duration,
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    start_decay_cleanup_worker(collection, decay_config, interval, cancel_token)
}

/// Starts a background task to periodically clean up orphan transactions and manage HNSW rebuilds with exponential backoff.
pub fn start_orphan_cleanup_worker_with_config<
    T: Clone + Send + Sync + 'static,
    I: OrphanCleanupIndex + 'static,
>(
    buffer: Arc<TxBuffer<T>>,
    hnsw_index: Arc<I>,
    interval: Duration,
    backoff_config: OrphanCleanupBackoffConfig,
    cancel_token: tokio_util::sync::CancellationToken,
) -> (tokio::task::JoinHandle<()>, Arc<AtomicU32>) {
    let consecutive_failed_rebuilds = Arc::new(AtomicU32::new(0));
    let failure_counter = consecutive_failed_rebuilds.clone();

    let handle = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut last_rebuild_attempt: Option<std::time::Instant> = None;

        tracing::info!(
            "Orphan cleanup worker started (timeout: {:?}, interval: {:?}, base_backoff: {:?})",
            buffer.tx_timeout(),
            interval,
            backoff_config.base_delay
        );
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let buf = buffer.clone();
                    let expired = tokio::task::spawn_blocking(move || {
                        buf.reap_orphans_bounded(MAX_ORPHANS_PER_TICK)
                    })
                    .await
                    .unwrap_or_default();

                    if !expired.is_empty() {
                        tracing::warn!(
                            "Orphan cleanup worker cleaned up {} expired transactions",
                            expired.len()
                        );
                    }

                    if let Err(err) = hnsw_index.check_connectivity() {
                        let failures = consecutive_failed_rebuilds.load(Ordering::Relaxed);
                        let cooldown = calculate_rebuild_cooldown(
                            failures,
                            backoff_config.base_delay,
                            backoff_config.max_delay,
                        );

                        let is_on_cooldown = if let Some(last_attempt) = last_rebuild_attempt {
                            last_attempt.elapsed() < cooldown
                        } else {
                            false
                        };

                        if is_on_cooldown {
                            if let Some(last_attempt) = last_rebuild_attempt {
                                let remaining = cooldown.saturating_sub(last_attempt.elapsed());
                                tracing::debug!(
                                    failures = failures,
                                    cooldown_remaining_ms = remaining.as_millis(),
                                    "HNSW index degraded but rebuild is on cooldown, skipping this tick"
                                );
                            }
                        } else {
                            tracing::warn!(
                                error = %err,
                                failures = failures,
                                "HNSW index degraded — triggering automatic rebuild"
                            );

                            last_rebuild_attempt = Some(std::time::Instant::now());

                            let rebuild_res = tokio::time::timeout(
                                Duration::from_secs(120),
                                hnsw_index.rebuild(),
                            )
                            .await;

                            let rebuild_ok = match rebuild_res {
                                Ok(Ok(())) => true,
                                Ok(Err(rebuild_err)) => {
                                    tracing::error!(error = %rebuild_err, "HNSW rebuild failed");
                                    false
                                }
                                Err(_) => {
                                    tracing::warn!("HNSW rebuild timed out after 120s; skipping this tick");
                                    false
                                }
                            };

                            let connectivity_restored = rebuild_ok && hnsw_index.check_connectivity().is_ok();

                            if connectivity_restored {
                                consecutive_failed_rebuilds.store(0, Ordering::Relaxed);
                                last_rebuild_attempt = None;
                                tracing::info!("HNSW rebuild succeeded and index connectivity restored");
                            } else {
                                let new_failures = consecutive_failed_rebuilds.fetch_add(1, Ordering::Relaxed) + 1;
                                tracing::error!(
                                    consecutive_failures = new_failures,
                                    "HNSW rebuild did not restore index connectivity"
                                );

                                if new_failures >= backoff_config.alert_threshold {
                                    tracing::error!(
                                        consecutive_failures = new_failures,
                                        "ALERT[STRUCTURAL_PROBLEM]: HNSW index rebuild failed {} consecutive times to restore connectivity. Deletion rate may exceed rebuild capacity or index is corrupted.",
                                        new_failures
                                    );
                                }
                            }
                        }
                    } else if consecutive_failed_rebuilds.load(Ordering::Relaxed) > 0 {
                        consecutive_failed_rebuilds.store(0, Ordering::Relaxed);
                        last_rebuild_attempt = None;
                        tracing::info!("HNSW index connectivity healthy, reset failed rebuild counter");
                    }
                }
                _ = cancel_token.cancelled() => {
                    tracing::info!("Orphan cleanup worker shutting down via token");
                    break;
                }
            }
        }
    });

    (handle, failure_counter)
}

/// Starts a background task to periodically clean up orphan transactions.
///
/// This worker handles the cleanup of transactions that have exceeded their
/// configured timeout without being committed or rolled back.
pub fn start_orphan_cleanup_worker<T: Clone + Send + Sync + 'static>(
    buffer: Arc<TxBuffer<T>>,
    hnsw_index: Arc<memfuse_index::hnsw::HnswIndex>,
    interval: Duration,
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    let (handle, _) = start_orphan_cleanup_worker_with_config(
        buffer,
        hnsw_index,
        interval,
        OrphanCleanupBackoffConfig::default(),
        cancel_token,
    );
    handle
}

/// Deprecated legacy alias for `start_orphan_cleanup_worker`.
#[deprecated(note = "use start_orphan_cleanup_worker instead")]
pub fn start_orphan_reaper<T: Clone + Send + Sync + 'static>(
    buffer: Arc<TxBuffer<T>>,
    hnsw_index: Arc<memfuse_index::hnsw::HnswIndex>,
    interval: Duration,
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    start_orphan_cleanup_worker(buffer, hnsw_index, interval, cancel_token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use memfuse_core::tx_buffer::IndexOp;
    use memfuse_core::types::{DocId, TxId};
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
        fn check_connectivity(&self) -> memfuse_core::Result<()> {
            if self.connectivity_ok.load(Ordering::SeqCst) {
                Ok(())
            } else {
                Err(memfuse_core::MemFuseError::HnswConnectivityDegraded {
                    deleted_ratio: 50.0,
                })
            }
        }

        fn rebuild(
            &self,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = memfuse_core::Result<()>> + Send + '_>,
        > {
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
        use memfuse_graph::CsrGraph;
        use memfuse_index::HnswIndex;
        use memfuse_store::LsmStorage;
        use std::sync::atomic::AtomicU64;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let storage = Arc::new(
            LsmStorage::new(memfuse_store::LsmConfig {
                path: dir.path().to_path_buf(),
                ..Default::default()
            })
            .await
            .unwrap(),
        );
        let index = Arc::new(
            HnswIndex::try_new(memfuse_index::HnswConfig {
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
            memfuse_text::Language::English,
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
        use memfuse_graph::CsrGraph;
        use memfuse_index::HnswIndex;
        use memfuse_store::LsmStorage;
        use std::sync::atomic::AtomicU64;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let storage = Arc::new(
            LsmStorage::new(memfuse_store::LsmConfig {
                path: dir.path().to_path_buf(),
                ..Default::default()
            })
            .await
            .unwrap(),
        );
        let index = Arc::new(
            HnswIndex::try_new(memfuse_index::HnswConfig {
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
            memfuse_text::Language::English,
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
        use memfuse_graph::CsrGraph;
        use memfuse_index::HnswIndex;
        use memfuse_store::LsmStorage;
        use std::sync::atomic::AtomicU64;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let storage = Arc::new(
            LsmStorage::new(memfuse_store::LsmConfig {
                path: dir.path().to_path_buf(),
                ..Default::default()
            })
            .await
            .unwrap(),
        );
        let index = Arc::new(
            HnswIndex::try_new(memfuse_index::HnswConfig {
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
            memfuse_text::Language::English,
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
        use memfuse_graph::CsrGraph;
        use memfuse_index::HnswIndex;
        use memfuse_store::LsmStorage;
        use std::sync::atomic::AtomicU64;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let storage = Arc::new(
            LsmStorage::new(memfuse_store::LsmConfig {
                path: dir.path().to_path_buf(),
                ..Default::default()
            })
            .await
            .unwrap(),
        );
        let index = Arc::new(
            HnswIndex::try_new(memfuse_index::HnswConfig {
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
            memfuse_text::Language::English,
        ));

        let vec = vec![1.0, 0.0, 0.0, 0.0];
        col.insert_with_ttl("doc_task_ttl", &vec, None, 2)
            .await
            .unwrap();

        col.insert("d1", &vec, None).await.unwrap();
        col.insert("d2", &vec, None).await.unwrap();

        let cancel_token = tokio_util::sync::CancellationToken::new();
        let handle = start_expiry_cleanup_worker(
            col.clone(),
            Duration::from_millis(10),
            cancel_token.clone(),
        );

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
        let config = memfuse_index::hnsw::HnswConfig::default();
        let hnsw_index = Arc::new(memfuse_index::hnsw::HnswIndex::try_new(config).unwrap());
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
        use memfuse_graph::CsrGraph;
        use memfuse_index::HnswIndex;
        use memfuse_store::LsmStorage;
        use serde_json::json;
        use std::sync::atomic::AtomicU64;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let lsm_config = memfuse_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap());
        let index = Arc::new(
            HnswIndex::try_new(memfuse_index::HnswConfig {
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
            memfuse_text::Language::English,
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
        use memfuse_graph::CsrGraph;
        use memfuse_index::HnswIndex;
        use memfuse_store::LsmStorage;
        use std::sync::atomic::AtomicU64;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let storage = Arc::new(
            LsmStorage::new(memfuse_store::LsmConfig {
                path: dir.path().to_path_buf(),
                ..Default::default()
            })
            .await
            .unwrap(),
        );
        let index = Arc::new(
            HnswIndex::try_new(memfuse_index::HnswConfig {
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
            memfuse_text::Language::English,
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
        use memfuse_core::{DecayFunction, ImportanceScore, MemoryImportance, TxId};
        use memfuse_graph::CsrGraph;
        use memfuse_index::HnswIndex;
        use memfuse_store::LsmStorage;
        use serde_json::json;
        use std::sync::atomic::Ordering;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let storage = Arc::new(
            LsmStorage::new(memfuse_store::LsmConfig {
                path: dir.path().to_path_buf(),
                ..Default::default()
            })
            .await
            .unwrap(),
        );
        let index = Arc::new(
            HnswIndex::try_new(memfuse_index::HnswConfig {
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
            memfuse_text::Language::English,
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
        use memfuse_adapt::DecayControllerConfig;
        use memfuse_core::{DecayFunction, ImportanceScore, MemoryImportance, TxId};
        use memfuse_graph::CsrGraph;
        use memfuse_index::HnswIndex;
        use memfuse_store::LsmStorage;
        use serde_json::json;
        use std::sync::atomic::Ordering;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let storage = Arc::new(
            LsmStorage::new(memfuse_store::LsmConfig {
                path: dir.path().to_path_buf(),
                ..Default::default()
            })
            .await
            .unwrap(),
        );
        let index = Arc::new(
            HnswIndex::try_new(memfuse_index::HnswConfig {
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
            memfuse_text::Language::English,
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
}
