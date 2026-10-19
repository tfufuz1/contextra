// FILE-CONTEXT
// ZWECK: Central Lifecycle Manager (EngineLifecycle) and ErasureSupervisor for background workers and purge tasks (§7.7, F-21, F-22, M3.7).
// INVARIANTEN: Shared CancellationToken; ordered shutdown across all background reapers and background erasure tasks.

use crate::background_workers::{start_expiry_cleanup_worker, start_orphan_cleanup_worker};
use crate::collection::Collection;
use contextra_mvcc::tx_buffer::TxBuffer;
use contextra_ports::StorageEngine;
use contextra_vector::hnsw::HnswIndex;
use parking_lot::Mutex;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

/// Supervisor managing async background erasure and purge operations (§5.3).
///
/// Ensures post-commit erasure tasks execute to completion even if caller
/// futures are dropped, and drains all active erasure tasks during lifecycle shutdown.
#[derive(Debug, Clone)]
pub struct ErasureSupervisor {
    cancel_token: CancellationToken,
    tracker: TaskTracker,
}

impl ErasureSupervisor {
    /// Creates a new `ErasureSupervisor` bound to a cancellation token.
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self {
            cancel_token,
            tracker: TaskTracker::new(),
        }
    }

    /// Spawns a background erasure/purge task managed by the supervisor.
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tracker.spawn(future)
    }

    /// Returns `true` if the supervisor cancellation token has been triggered.
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// Initiates supervisor shutdown and waits for all active erasure tasks to finish.
    pub async fn shutdown(&self) {
        self.tracker.close();
        self.tracker.wait().await;
    }
}

/// Central engine lifecycle manager bundling background reapers and the `ErasureSupervisor` (§7.7).
///
/// Manages background maintenance workers:
/// - Expiry reaper (`start_expiry_reaper`)
/// - Orphan reaper (`start_orphan_reaper`)
/// - `ErasureSupervisor`
pub struct EngineLifecycle {
    cancel_token: CancellationToken,
    erasure_supervisor: ErasureSupervisor,
    handles: Mutex<Vec<JoinHandle<()>>>,
}

impl fmt::Debug for EngineLifecycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EngineLifecycle")
            .field("cancel_token", &self.cancel_token)
            .field("erasure_supervisor", &self.erasure_supervisor)
            .field("handles_count", &self.handles.lock().len())
            .finish()
    }
}

impl EngineLifecycle {
    /// Creates a new `EngineLifecycle` manager with a shared cancellation token.
    pub fn new() -> Self {
        let cancel_token = CancellationToken::new();
        let erasure_supervisor = ErasureSupervisor::new(cancel_token.clone());
        Self {
            cancel_token,
            erasure_supervisor,
            handles: Mutex::new(Vec::new()),
        }
    }

    /// Returns the shared cancellation token.
    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// Returns a reference to the `ErasureSupervisor`.
    pub fn erasure_supervisor(&self) -> &ErasureSupervisor {
        &self.erasure_supervisor
    }

    /// Registers an arbitrary background worker handle under lifecycle management.
    pub fn register_worker(&self, handle: JoinHandle<()>) {
        self.handles.lock().push(handle);
    }

    /// Starts and registers the expiry cleanup reaper task.
    pub fn start_expiry_reaper<S: StorageEngine + 'static>(
        &self,
        collection: Arc<Collection<S>>,
        interval: Duration,
    ) {
        let handle = start_expiry_cleanup_worker(collection, interval, self.cancel_token.clone());
        self.handles.lock().push(handle);
    }

    /// Starts and registers the orphan transaction cleanup reaper task.
    pub fn start_orphan_reaper<T: Clone + Send + Sync + 'static>(
        &self,
        buffer: Arc<TxBuffer<T>>,
        hnsw_index: Arc<HnswIndex>,
        interval: Duration,
    ) {
        let handle =
            start_orphan_cleanup_worker(buffer, hnsw_index, interval, self.cancel_token.clone());
        self.handles.lock().push(handle);
    }

    /// Triggers graceful cancellation across all background tasks and supervisor tasks.
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// Returns `true` if lifecycle shutdown has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// Performs an ordered, graceful shutdown of all background reapers and the `ErasureSupervisor`.
    pub async fn close(&self) {
        self.cancel();

        // 1. Drain and await background worker handles
        let handles: Vec<JoinHandle<()>> = std::mem::take(&mut *self.handles.lock());
        for handle in handles {
            let _ = handle.await;
        }

        // 2. Shut down ErasureSupervisor (waiting for active purge tasks)
        self.erasure_supervisor.shutdown().await;
    }
}

impl Default for EngineLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EngineLifecycle {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ContextraConfig;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_engine_lifecycle_worker_registration_and_close() {
        let lifecycle = EngineLifecycle::new();

        let ran = Arc::new(AtomicBool::new(false));
        let ran_clone = Arc::clone(&ran);
        let token = lifecycle.cancel_token();

        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(10)) => {}
                _ = token.cancelled() => {
                    ran_clone.store(true, Ordering::SeqCst);
                }
            }
        });

        lifecycle.register_worker(handle);
        assert!(!lifecycle.is_cancelled());

        lifecycle.close().await;

        assert!(lifecycle.is_cancelled());
        assert!(ran.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_erasure_supervisor_task_completion_on_close() {
        let lifecycle = EngineLifecycle::new();
        let supervisor = lifecycle.erasure_supervisor().clone();

        let task_completed = Arc::new(AtomicBool::new(false));
        let task_completed_clone = Arc::clone(&task_completed);

        supervisor.spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            task_completed_clone.store(true, Ordering::SeqCst);
        });

        lifecycle.close().await;

        assert!(task_completed.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_reaper_lifecycle_start_and_shutdown() {
        let tmp = tempdir().unwrap();
        let config = ContextraConfig {
            consolidation_enabled: false,
            ..Default::default()
        };
        let db = crate::Contextra::open_with_config(tmp.path(), config)
            .await
            .unwrap();
        let col = db.collection("default").await.unwrap();

        let buffer = Arc::new(TxBuffer::<String>::new());
        let hnsw_config = contextra_vector::HnswConfig {
            dimension: 768,
            distance_metric: contextra_types::DistanceMetric::Cosine,
            max_elements: 1000,
            ..Default::default()
        };
        let hnsw_index = Arc::new(HnswIndex::try_new(hnsw_config).unwrap());

        let lifecycle = EngineLifecycle::new();

        lifecycle.start_expiry_reaper(col, Duration::from_millis(20));
        lifecycle.start_orphan_reaper(buffer, hnsw_index, Duration::from_millis(20));

        tokio::time::sleep(Duration::from_millis(50)).await;

        lifecycle.close().await;

        assert!(lifecycle.is_cancelled());
        db.close().await.unwrap();
    }
}
