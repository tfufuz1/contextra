use super::config::LsmConfig;
use super::guard::LsmState;
use super::group_commit::PendingCommitQueue;
use crate::compaction::CompactionEngine;
use crate::memtable::MemTable;
use crate::sstable::{BlockCache, SstableReader};
use crate::wal::Wal;
use contextra_core::{ContextraError, Result, ResourceTracker, SnapshotRegistry, StorageEngine, TxBuffer, TxId};
use contextra_crypto::crypto::KeyManager;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// LSM-Tree based storage engine.
pub struct LsmStorage {
    pub config: LsmConfig,
    pub(super) key_manager: Option<Arc<KeyManager>>,
    pub(super) state: RwLock<LsmState>,
    pub(super) sstables: Arc<RwLock<Vec<Arc<SstableReader>>>>,
    pub(super) tx_buffer: TxBuffer<(Vec<u8>, Vec<u8>)>,
    pub(super) budget: Arc<ResourceTracker>,
    pub(super) block_cache: Arc<BlockCache>,
    pub(super) wal: RwLock<Arc<Wal>>,
    pub snapshot_registry: Arc<SnapshotRegistry>,
    pub(super) compaction_engine: Arc<CompactionEngine>,
    pub(super) manifest: Arc<crate::manifest::Manifest>,
    pub(super) next_seq_no: AtomicU64,
    pub(super) last_committed_tx: AtomicU64,
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
}
