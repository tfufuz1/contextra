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
use contextra_core::{ResourceTracker, Result, SnapshotRegistry, StorageEngine, TxBuffer, TxId};
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
    pub(super) last_applied_seq: AtomicU64,
    pub(super) flush_notify: Arc<tokio::sync::Notify>,
    pub(super) health: Arc<parking_lot::RwLock<StorageHealth>>,
    /// Mutex to serialize commits and prevent snapshot inversion (parallel seq_no holes).
    pub(super) commit_mutex: tokio::sync::Mutex<()>,
    /// Mutex to serialize flush executions and prevent duplicate SST / double-release races (H4c).
    pub(super) flush_mutex: tokio::sync::Mutex<()>,
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
    pub ssi_validator: Arc<contextra_mvcc::SequenceLogSsiValidator>,
    pub(super) metrics_sink: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>,
}

impl Drop for LsmStorage {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}

impl LsmStorage {
    /// Opens or recovers an LSM storage instance with the given configuration.
    pub async fn open(config: LsmConfig) -> Result<Self> {
        Self::new_with_merge_operator(config, None).await
    }

    /// Opens or recovers an LSM storage instance with the given configuration and a `MergeOperator`.
    // TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task
    pub async fn open_with_merge_operator(
        config: LsmConfig,
        merge_operator: Arc<dyn MergeOperator>,
    ) -> Result<Self> {
        Self::new_with_merge_operator(config, Some(merge_operator)).await
    }

    /// Returns a watch receiver for monitoring system pressure levels.
    // TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task
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
        let active_wal = {
            let wal_guard = self.wal.read().await;
            Arc::clone(&*wal_guard)
        };
        active_wal.close().await?;
        Ok(())
    }

    /// Spawns a background task tracked by this storage instance.
    // TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task
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

    /// Returns the currently configured [`MetricsSink`].
    pub fn metrics_sink(&self) -> Arc<dyn contextra_ports::MetricsSink> {
        self.metrics_sink.read().clone()
    }

    /// Sets the [`MetricsSink`] for recording operational metrics.
    pub fn set_metrics_sink(&self, sink: Arc<dyn contextra_ports::MetricsSink>) {
        *self.metrics_sink.write() = sink;
    }

    /// Configures the [`MetricsSink`] fluently on `self` and returns `self`.
    pub fn with_metrics_sink(self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self {
        *self.metrics_sink.write() = sink;
        self
    }

    /// Sets the injected clock port for deterministic observer latency evaluation (P28).
    pub fn set_clock(&self, clock: Arc<dyn contextra_ports::Clock>) {
        self.observer_registry.set_clock(clock);
    }

    /// Performs a tracked read for transaction `tx_id`.
    ///
    /// Evaluates `snapshot_seq = next_seq_no - 1` ONCE (K2), registers the read key in `TxBuffer`,
    /// and reads the key at `snapshot_seq`.
    ///
    /// # SSI Protection Scope
    /// ONLY keys read via `get_tracked` or `get_at_seq_tracked` within transaction `tx_id` are registered
    /// in the transaction's `ReadSet` and receive Serializable Snapshot Isolation (SSI) conflict validation
    /// during commit. Untracked reads via `StorageEngine::get()` or `LsmStorage::get()` perform point-in-time
    /// snapshot reads without transaction tracking and do NOT receive SSI write-skew protection.
    pub async fn get_tracked(&self, tx_id: TxId, key: &[u8]) -> Result<Option<Bytes>> {
        read::get_tracked(self, tx_id, key).await
    }

    /// Performs a tracked read for transaction `tx_id` at an explicit `snapshot_seq`.
    ///
    /// Registers the read key in `TxBuffer` at `snapshot_seq` and reads the key at `snapshot_seq`.
    ///
    /// # SSI Protection Scope
    /// ONLY keys read via `get_tracked` or `get_at_seq_tracked` within transaction `tx_id` are registered
    /// in the transaction's `ReadSet` and receive Serializable Snapshot Isolation (SSI) conflict validation
    /// during commit. Untracked reads via `StorageEngine::get()` or `LsmStorage::get()` perform point-in-time
    /// snapshot reads without transaction tracking and do NOT receive SSI write-skew protection.
    pub async fn get_at_seq_tracked(
        &self,
        tx_id: TxId,
        key: &[u8],
        snapshot_seq: u64,
    ) -> Result<Option<Bytes>> {
        read::get_at_seq_tracked(self, tx_id, key, snapshot_seq).await
    }

    /// Performs a tracked prefix scan for transaction `tx_id`.
    ///
    /// Evaluates `snapshot_seq = last_applied_seq` ONCE, scans all matching entries, and registers
    /// each scanned key in transaction `tx_id`'s `ReadSet`.
    pub async fn scan_prefix_tracked(
        &self,
        tx_id: TxId,
        prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        read::scan_prefix_tracked(self, tx_id, prefix).await
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
    pub fn next_seq_no_for_test(&self) -> u64 {
        self.next_seq_no.load(std::sync::atomic::Ordering::Acquire)
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
    pub fn budget_for_test(&self) -> Arc<ResourceTracker> {
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

    /// Checks if a database directory contains any legacy WAL files requiring explicit migration.
    ///
    /// Dies ist der Migrationsmechanismus vor der endgültigen Entfernung des Legacy-Fallbacks
    /// (v17 Teil 4.1, Ziel P1). Die Entfernung selbst erfolgt in einem separaten PR, erst
    /// nachdem bestätigt ist, dass keine produktiven Alt-WAL-Dateien mehr existieren.
    pub async fn has_pending_legacy_wal_migration_path(path: &std::path::Path) -> Result<bool> {
        let mut entries = match tokio::fs::read_dir(path).await {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(e) => {
                return Err(contextra_core::ContextraError::Storage(format!(
                    "Failed to read directory for legacy WAL check: {e}"
                )))
            }
        };

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            contextra_core::ContextraError::Storage(format!(
                "Failed to read directory entry for legacy WAL check: {e}"
            ))
        })? {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            let is_wal_file = (name_str.starts_with("wal-") && name_str.ends_with(".log"))
                || name_str == "wal.log"
                || name_str.ends_with(".wal");

            if is_wal_file {
                let file_path = entry.path();
                if !Wal::has_migration_marker(&file_path).await {
                    let test_wal = Wal::open_read_only_with_config(
                        &file_path,
                        crate::wal::WalConfig {
                            allow_legacy_integrity_key_fallback: true,
                            min_wal_version: crate::wal::WalVersion::V1,
                            ..Default::default()
                        },
                    )
                    .await?;

                    let _ = test_wal.replay().await?;
                    if test_wal.legacy_key_used_for_test() {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Checks if the current storage instance's data directory contains any pending legacy WAL files.
    ///
    /// Dies ist der Migrationsmechanismus vor der endgültigen Entfernung des Legacy-Fallbacks
    /// (v17 Teil 4.1, Ziel P1). Die Entfernung selbst erfolgt in einem separaten PR, erst
    /// nachdem bestätigt ist, dass keine produktiven Alt-WAL-Dateien mehr existieren.
    // TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task
    pub async fn has_pending_legacy_wal_migration(&self) -> Result<bool> {
        Self::has_pending_legacy_wal_migration_path(&self.config.path).await
    }

    /// Explicitly migrates all legacy WAL files in the specified directory using the provided clock port.
    ///
    /// Returns the number of migrated WAL files.
    ///
    /// Dies ist der Migrationsmechanismus vor der endgültigen Entfernung des Legacy-Fallbacks
    /// (v17 Teil 4.1, Ziel P1). Die Entfernung selbst erfolgt in einem separaten PR, erst
    /// nachdem bestätigt ist, dass keine produktiven Alt-WAL-Dateien mehr existieren.
    // TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task
    pub async fn migrate_legacy_wal_keys(
        config: &LsmConfig,
        clock: Arc<dyn contextra_ports::Clock>,
    ) -> Result<usize> {
        let mut entries = match tokio::fs::read_dir(&config.path).await {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(e) => {
                return Err(contextra_core::ContextraError::Storage(format!(
                    "Failed to read directory for legacy WAL migration: {e}"
                )))
            }
        };

        let salt_path = config.path.join("SALT");
        if !salt_path.exists() {
            let buf = crate::lsm::recovery::generate_crypto_salt();
            crate::lsm::recovery::write_salt_atomically(&salt_path, &buf).await?;
        }

        let salt = tokio::fs::read(&salt_path).await.map_err(|e| {
            contextra_core::ContextraError::Storage(format!(
                "SALT file read failed during migration: {e}"
            ))
        })?;

        let key_manager = if let Some(passphrase) = &config.encryption_passphrase {
            Some(Arc::new(crate::wal::KeyManager::try_new(
                passphrase, &salt,
            )?))
        } else {
            None
        };

        let mut migrated_count = 0usize;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            contextra_core::ContextraError::Storage(format!(
                "Failed to read directory entry for legacy WAL migration: {e}"
            ))
        })? {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            let is_wal_file = (name_str.starts_with("wal-") && name_str.ends_with(".log"))
                || name_str == "wal.log"
                || name_str.ends_with(".wal");

            if is_wal_file {
                let file_path = entry.path();
                if !Wal::has_migration_marker(&file_path).await {
                    let wal =
                        Wal::open_for_legacy_migration(&file_path, key_manager.clone()).await?;

                    if wal.was_legacy_rekeyed() {
                        let timestamp_nanos = clock.now_unix_nanos();
                        tracing::info!(
                            wal_path = %file_path.display(),
                            timestamp_nanos = timestamp_nanos,
                            "Legacy WAL key migration executed for segment"
                        );
                        migrated_count += 1;
                    }
                }
            }
        }

        Ok(migrated_count)
    }
}
