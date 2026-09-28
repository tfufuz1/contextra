use super::config::LsmConfig;
use super::group_commit::PendingCommitQueue;
use super::guard::LsmState;
use super::observer::{ObserverRegistry, WalObserver};
use crate::compaction::CompactionEngine;
use crate::sstable::{BlockCache, SstableReader};
use crate::wal::KeyManager;
use crate::wal::Wal;
use bytes::Bytes;
use contextra_core::{ResourceTracker, Result, SnapshotRegistry, StorageEngine, TxBuffer, TxId};
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::ops::read;

/// LSM-Tree based storage engine.
pub struct LsmStorage {
    pub(super) config: LsmConfig,
    pub(super) key_manager: Option<Arc<KeyManager>>,
    pub(super) state: RwLock<LsmState>,
    /// SSTables stored separately for shared access with compaction engine.
    pub(super) sstables: Arc<RwLock<Vec<Arc<SstableReader>>>>,
    pub(super) tx_buffer: TxBuffer<(Vec<u8>, Vec<u8>)>,
    pub(super) budget: Arc<ResourceTracker>,
    pub(super) block_cache: Arc<BlockCache>,
    pub(super) wal: RwLock<Arc<Wal>>,
    pub snapshot_registry: Arc<SnapshotRegistry>,
    /// Persistent CompactionEngine instance — retains counter across maybe_compact() calls.
    /// Prevents SSTable name collisions from fresh-counter ad-hoc instantiation (audit H-3).
    pub(super) compaction_engine: Arc<CompactionEngine>,
    pub(super) manifest: Arc<crate::manifest::Manifest>,
    pub(super) next_seq_no: AtomicU64,
    pub(super) last_committed_tx: AtomicU64,
    /// Mutex to serialize commits and prevent snapshot inversion (parallel seq_no holes).
    pub(super) commit_mutex: tokio::sync::Mutex<()>,
    pub(super) cancel_token: tokio_util::sync::CancellationToken,
    pub(super) task_tracker: tokio_util::task::TaskTracker,
    pub(super) flush_counter: AtomicU64,
    pub(super) segment_counter: AtomicU64,
    pub(super) budget_tracking_drift_bytes: std::sync::atomic::AtomicU64,
    pub(super) pending_commit_queue: tokio::sync::Mutex<Option<PendingCommitQueue>>,
    pub(super) wal_queue_depth: Arc<std::sync::atomic::AtomicUsize>,
    pub(super) pressure_rx: tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure>,
    pub(super) intent_locks: std::sync::Mutex<HashMap<Vec<u8>, TxId>>,
    pub(super) observer_registry: ObserverRegistry,
    pub ssi_validator: Arc<contextra_mvcc::SequenceLogSsiValidator>,
}

impl Drop for LsmStorage {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}

impl LsmStorage {
    /// Returns a watch receiver for monitoring system pressure levels.
    pub fn pressure_receiver(
        &self,
    ) -> tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure> {
        self.pressure_rx.clone()
    }

    /// Signals the background compaction engine to stop.
    pub fn shutdown(&self) {
        self.cancel_token.cancel();
    }

    /// Waits for all spawned tasks to shut down fully.
    pub async fn wait_shutdown(&self) {
        self.shutdown();
        self.task_tracker.wait().await;
    }

    /// Gracefully closes the storage engine, stopping background tasks and flushing active memtable to disk.
    pub async fn close(&self) -> Result<()> {
        self.wait_shutdown().await;
        self.flush().await?;
        Ok(())
    }

    /// Spawns a background task tracked by this storage instance.
    pub fn spawn_tracked<F>(&self, future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.task_tracker.spawn(future);
    }

    #[doc(hidden)]
    pub async fn simulate_wal_append_failure_for_test(&self) {
        #[cfg(feature = "fault-injection")]
        crate::wal::FAIL_APPEND_FOR_TX.store(u64::MAX, std::sync::atomic::Ordering::SeqCst);
    }

    #[doc(hidden)]
    pub async fn restore_wal_file_handle_for_test(&self) {
        #[cfg(feature = "fault-injection")]
        crate::wal::FAIL_APPEND_FOR_TX.store(0, std::sync::atomic::Ordering::SeqCst);
    }

    /// Returns the accumulated total memory budget tracking drift in bytes caused by
    /// unbudgeted memtable puts during commit when memory limit was exceeded.
    pub fn budget_tracking_drift_bytes(&self) -> u64 {
        self.budget_tracking_drift_bytes
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Maximum threshold for surviving entries during rollback below which entries are retained in memtable
    /// instead of creating a new SSTable (M-4 optimization).
    pub const ROLLBACK_INLINE_THRESHOLD_ENTRIES: usize = 1;

    /// Registers a new WAL observer.
    pub fn register_observer(&self, observer: Arc<dyn WalObserver>) {
        self.observer_registry.register_observer(observer);
    }

    /// Deregisters an observer matching the provided `Arc` reference via pointer equality.
    pub fn deregister_observer(&self, observer: &Arc<dyn WalObserver>) {
        self.observer_registry.deregister_observer(observer);
    }

    /// Sets the maximum latency threshold for observer callbacks.
    pub fn set_max_observer_latency(&self, latency: std::time::Duration) {
        self.observer_registry.set_max_observer_latency(latency);
    }

    /// Returns the current maximum latency threshold for observer callbacks.
    pub fn max_observer_latency(&self) -> std::time::Duration {
        self.observer_registry.max_observer_latency()
    }

    /// Sets the injected clock port for deterministic observer latency evaluation (P28).
    pub fn set_clock(&self, clock: Arc<dyn contextra_ports::Clock>) {
        self.observer_registry.set_clock(clock);
    }

    /// Performs a tracked read for transaction `tx_id`.
    ///
    /// Evaluates `snapshot_seq = next_seq_no - 1` ONCE (K2), registers the read key in `TxBuffer`,
    /// and reads the key at `snapshot_seq`.
    pub async fn get_tracked(&self, tx_id: TxId, key: &[u8]) -> Result<Option<Bytes>> {
        read::get_tracked(self, tx_id, key).await
    }

    /// Performs a tracked read for transaction `tx_id` at an explicit `snapshot_seq`.
    ///
    /// Registers the read key in `TxBuffer` at `snapshot_seq` and reads the key at `snapshot_seq`.
    pub async fn get_at_seq_tracked(
        &self,
        tx_id: TxId,
        key: &[u8],
        snapshot_seq: u64,
    ) -> Result<Option<Bytes>> {
        read::get_at_seq_tracked(self, tx_id, key, snapshot_seq).await
    }

    /// Returns the current `next_seq_no` value for testing verification.
    pub fn next_seq_no_for_test(&self) -> u64 {
        self.next_seq_no.load(std::sync::atomic::Ordering::Acquire)
    }
}
