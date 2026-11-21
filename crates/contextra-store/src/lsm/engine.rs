//! # LSM-Storage Engine (Ring 1)
//!
//! Implementiert [`LsmStorage`] als zentrale Fassade fuer alle Speicheroperationen.


pub mod facade;
pub mod migration;
use super::config::LsmConfig;
use super::group_commit::PendingCommitQueue;
use super::guard::LsmState;
use super::observer::{ObserverRegistry, WalObserver};
use super::ops::read;
use crate::compaction::{CompactionEngine, MergeOperator};
use crate::sstable::{BlockCache, SstableReader};
use crate::wal::KeyManager;
use crate::wal::Wal;
use bytes::Bytes;
use crate::snapshot_pins::SnapshotPins;
use crate::staging::TxStaging;
use crate::budget::ResourceTracker;
use crate::error::StoreResult as Result;
use contextra_ids::TxId;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Health status of the storage engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StorageHealth {
    Healthy,
    FlushFailing,
}

/// LSM-Tree based storage engine.
/// Primaere Schnittstelle fuer den LSM-Baum mit get, put, delete und scan.
pub struct LsmStorage {
    pub(super) config: LsmConfig,
    pub(super) key_manager: Option<Arc<KeyManager>>,
    pub(super) state: RwLock<LsmState>,
    /// SSTables stored separately for shared access with compaction engine.
    pub(super) sstables: Arc<RwLock<Vec<Arc<SstableReader>>>>,
    pub(super) tx_staging: TxStaging,
    pub(super) budget: Arc<crate::budget::ResourceTracker>,
    pub(super) block_cache: Arc<BlockCache>,
    pub(super) wal: RwLock<Arc<Wal>>,
    pub(crate) snapshot_registry: Arc<SnapshotPins>,
    /// Persistent CompactionEngine instance — retains counter across maybe_compact() calls.
    /// Prevents SSTable name collisions from fresh-counter ad-hoc instantiation (audit H-3).
    pub(super) compaction_engine: Arc<CompactionEngine>,
    pub(super) manifest: Arc<crate::manifest::Manifest>,
    pub(super) next_seq_no: AtomicU64,
    pub(super) last_committed_tx: AtomicU64,
    pub(super) last_applied_seq: AtomicU64,
    pub(super) flush_notify: Arc<tokio::sync::Notify>,
    pub(super) health: Arc<parking_lot::RwLock<StorageHealth>>,
    /// Mutex to serialize commits and prevent snapshot inversion (parallel seq_no holes).
    pub(super) commit_mutex: tokio::sync::Mutex<()>,
    /// Mutex to serialize flush executions and prevent duplicate SST / double-release races (H4c).
    pub(super) flush_mutex: tokio::sync::Mutex<()>,
    /// Write-fencing coordinator between Flush and Purge (SPEC §3.5, WP-STO-5).
    pub(crate) flush_purge_fence: super::fence::FlushPurgeFence,
    /// Pending sealed WAL files awaiting SST flush and cleanup (H4a + H4b).
    pub(super) pending_sealed_wals: tokio::sync::Mutex<Vec<(std::path::PathBuf, [u8; 32])>>,
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
    pub(super) clock: parking_lot::RwLock<Arc<dyn contextra_env::Clock>>,
    pub(super) metrics_sink: Arc<parking_lot::RwLock<Arc<dyn contextra_env::MetricsSink>>>,
}

impl Drop for LsmStorage {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}

impl LsmStorage {
    /// Opens or recovers an LSM storage instance with the given configuration.
    ///
    /// # Errors
    /// Gibt `ContextraError::Storage` zurück, falls I/O-Fehler, Verzeichnis-Lock-Konflikte
    /// oder Korruptionen während des Manifest- oder WAL-Replays auftreten.
    ///
    /// # Panics
    /// Panics: keine.
    ///
    /// # Crash
    /// Führt Open-Heal am WAL-Tail durch; stellt den konsistenten Zustand nach Abstürzen wieder her.
    ///
    /// # Concurrency & Performance
    /// Initialisierung blockiert bis zur Freigabe des Verzeichnis-Locks (`DirLock`). Komplexität: $O(N)$
    /// proportional zur Anzahl unkompaktierter WAL-Frames und Manifest-Einträge.
    pub async fn open(config: LsmConfig) -> Result<Self> {
        Self::new_with_merge_operator(config, None).await
    }

    /// Opens or recovers an LSM storage instance with the given configuration and a `MergeOperator`.
    ///
    /// # Errors
    /// Gibt `ContextraError::Storage` zurück bei Initialisierungs-, Format- oder I/O-Fehlern.
    ///
    /// # Panics
    /// Panics: keine.
    ///
    /// # Crash
    /// Validiert bestehende SSTables und stellt WAL-Kettenintegrität über HMAC sicher.
    pub async fn open_with_merge_operator(
        config: LsmConfig,
        merge_operator: Arc<dyn MergeOperator>,
    ) -> Result<Self> {
        Self::new_with_merge_operator(config, Some(merge_operator)).await
    }

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
    ///
    /// # Errors
    /// Gibt `ContextraError::Storage` zurück, falls beim abschließenden Flush I/O- oder Fsync-Fehler auftreten.
    ///
    /// # Panics
    /// Panics: keine.
    ///
    /// # Crash
    /// Schreibt und synchronisiert verbleibende MemTable-Inhalte als SSTable auf Platte.
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
    
    pub async fn simulate_wal_append_failure_for_tx_for_test(&self, _tx_id: u64) {
        #[cfg(feature = "fault-injection")]
        crate::wal::FAIL_APPEND_FOR_TX.store(_tx_id, std::sync::atomic::Ordering::SeqCst);
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

    /// Returns the currently configured [`contextra_env::MetricsSink`].
    pub fn metrics_sink(&self) -> Arc<dyn contextra_env::MetricsSink> {
        self.metrics_sink.read().clone()
    }

    /// Sets the [`contextra_env::MetricsSink`] for recording operational metrics.
    pub fn set_metrics_sink(&self, sink: Arc<dyn contextra_env::MetricsSink>) {
        *self.metrics_sink.write() = sink;
    }

    /// Configures the [`contextra_env::MetricsSink`] fluently on `self` and returns `self`.
    pub fn with_metrics_sink(self, sink: Arc<dyn contextra_env::MetricsSink>) -> Self {
        *self.metrics_sink.write() = sink;
        self
    }

    /// Returns the currently configured clock port.
    pub fn clock(&self) -> Arc<dyn contextra_env::Clock> {
        self.clock.read().clone()
    }

    /// Sets the injected clock port for deterministic execution timing (P28).
    pub fn set_clock(&self, clock: Arc<dyn contextra_env::Clock>) {
        *self.clock.write() = Arc::clone(&clock);
        self.observer_registry.set_clock(clock);
    }

    /// Configures the clock port fluently on `self` and returns `self`.
    pub fn with_clock(self, clock: Arc<dyn contextra_env::Clock>) -> Self {
        self.set_clock(clock);
        self
    }

    /// Returns the highest sequence number applied to the MemTable and published.
    pub fn last_applied_seq(&self) -> u64 {
        self.last_applied_seq
            .load(std::sync::atomic::Ordering::Acquire)
    }

    /// Returns the current storage health status.
    pub fn health(&self) -> StorageHealth {
        *self.health.read()
    }

    /// Requests an asynchronous background flush without blocking the caller or holding commit locks.
    pub fn request_flush(&self) {
        self.flush_notify.notify_one();
    }

    /// Returns true if at least one observer is registered.
    pub fn has_observers(&self) -> bool {
        !self.observer_registry.is_empty()
    }

    /// Returns the current `next_seq_no` value for testing verification.
    #[doc(hidden)]
    
    pub fn next_seq_no_for_test(&self) -> u64 {
        self.next_seq_no.load(std::sync::atomic::Ordering::Acquire)
    }

    #[allow(dead_code)]
    pub(crate) fn sstables_arc(&self) -> Arc<RwLock<Vec<Arc<SstableReader>>>> {
        Arc::clone(&self.sstables)
    }

    #[allow(dead_code)]
    pub(crate) fn block_cache(&self) -> Arc<BlockCache> {
        Arc::clone(&self.block_cache)
    }

    #[doc(hidden)]
    
    pub fn sstables_for_test(&self) -> Arc<RwLock<Vec<Arc<SstableReader>>>> {
        Arc::clone(&self.sstables)
    }

    #[doc(hidden)]
    
    pub fn block_cache_for_test(&self) -> Arc<BlockCache> {
        Arc::clone(&self.block_cache)
    }

    #[doc(hidden)]
    
    pub fn budget_for_test(&self) -> Arc<crate::budget::ResourceTracker> {
        Arc::clone(&self.budget)
    }

    #[doc(hidden)]
    
    pub fn manifest_for_test(&self) -> Arc<crate::manifest::Manifest> {
        Arc::clone(&self.manifest)
    }

    #[doc(hidden)]
    
    pub async fn maybe_compact_for_test(&self) -> Result<bool> {
        self.compaction_engine
            .maybe_compact(&self.sstables, &self.config.path)
            .await
    }


    /// Returns a snapshot of the active memtable and immutable memtables for scanner/purger introspection.
    #[allow(dead_code)]
    pub(crate) async fn memtables_for_scanner(
        &self,
    ) -> (
        Arc<crate::memtable::MemTable>,
        Vec<Arc<crate::memtable::MemTable>>,
    ) {
        let state = self.state.read().await;
        (
            Arc::clone(&state.memtable),
            state.immutable_memtables.clone(),
        )
    }

    /// Returns a snapshot of currently open SSTable readers.
    #[allow(dead_code)]
    pub(crate) async fn sstables_for_scanner(&self) -> Vec<Arc<crate::sstable::SstableReader>> {
        self.sstables.read().await.clone()
    }

    /// Liefert den Key-Manager (Purge muss Neuschreibungen mit demselben Schluessel ablegen, T-DEL-1).
    pub(crate) fn key_manager(&self) -> Option<Arc<KeyManager>> {
        self.key_manager.clone()
    }

    /// Returns the data directory path.
    pub fn data_path(&self) -> &std::path::Path {
        &self.config.path
    }

    /// Returns the VFS instance.
    pub fn vfs(&self) -> &std::sync::Arc<dyn crate::io_seam::Vfs> {
        &self.compaction_engine.vfs
    }
}


