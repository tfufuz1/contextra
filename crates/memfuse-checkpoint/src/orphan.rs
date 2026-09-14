use crate::meta::StateCheckpoint;
use memfuse_core::{Result, TxId};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Type alias for sequence numbers managed as pinned checkpoint identifiers.
pub type PinId = u64;

#[deprecated(
    since = "0.1.0",
    note = "Use InstanceOrphanRegistry instead. Global statics are deprecated per ADR-053."
)]
#[allow(deprecated)]
pub static ORPHAN_REGISTRY: std::sync::OnceLock<OrphanRegistry> = std::sync::OnceLock::new();

fn warn_deprecated_global_orphan_path() {
    static WARN_ONCE: std::sync::Once = std::sync::Once::new();
    WARN_ONCE.call_once(|| {
        tracing::warn!(
            "Deprecated global OrphanRegistry path used — migrate to InstanceOrphanRegistry (ADR-053)"
        );
    });
}

#[deprecated(
    since = "0.1.0",
    note = "Use InstanceOrphanRegistry instead. Global functions are not safe in multi-instance environments."
)]
#[allow(deprecated)]
pub fn global_orphan_registry() -> &'static OrphanRegistry {
    warn_deprecated_global_orphan_path();
    ORPHAN_REGISTRY.get_or_init(OrphanRegistry::default)
}

/// Returns the default directory for orphan state files when no environment variable is set.
///
/// # Security & Persistence Notice
/// Uses `dirs::data_local_dir()`. If unavailable, this falls back to [`std::env::temp_dir()`].
/// [`std::env::temp_dir()`] is NOT a permanent storage location and may be cleared by the OS.
/// For production deployments, `MEMFUSE_ORPHAN_PIN_PATH` and `MEMFUSE_ORPHAN_PATH` MUST be explicitly configured.
fn default_data_dir() -> std::path::PathBuf {
    dirs::data_local_dir().unwrap_or_else(std::env::temp_dir)
}

pub(crate) fn monotonic_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn orphan_pin_file_path() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("MEMFUSE_ORPHAN_PIN_PATH") {
        return std::path::PathBuf::from(p);
    }
    default_data_dir().join("memfuse_orphaned_pins.json")
}

/// Durable append-only registry for orphaned sequence pins (ADR-052).
#[deprecated(
    since = "0.1.0",
    note = "Use InstanceOrphanRegistry instead. Global OrphanRegistry is deprecated per ADR-053."
)]
pub struct OrphanRegistry {
    inner: InstanceOrphanRegistry,
}

#[allow(deprecated)]
impl Default for OrphanRegistry {
    fn default() -> Self {
        Self::new(orphan_pin_file_path())
    }
}

#[allow(deprecated)]
impl OrphanRegistry {
    pub fn new(file_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            inner: InstanceOrphanRegistry::new(file_path),
        }
    }

    /// Synchronously registers an orphaned sequence pin and persists to disk.
    pub fn register_orphan(&self, pin_id: PinId) -> std::io::Result<()> {
        let wall_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.inner.register_orphan_sync(PinnedSeqNoOrphan {
            seq_no: pin_id,
            timestamp_ms: wall_ms,
        });
        Ok(())
    }

    /// Retrieves all currently registered orphaned pin sequence numbers.
    pub fn get_orphans(&self) -> Vec<PinId> {
        self.inner
            .get_orphan_pins()
            .into_iter()
            .map(|o| o.seq_no)
            .collect()
    }

    /// Synchronously clears memory and disk records.
    pub fn clear_all(&self) {
        self.inner.clear_all();
    }

    /// Recovers all registered orphaned pins by unpinning them in storage and cleaning the registry.
    pub async fn recover_and_clean<S: memfuse_core::StorageEngine>(
        &self,
        storage: &S,
    ) -> Result<Vec<PinId>> {
        let orphans = self.get_orphans();
        let mut recovered = Vec::new();

        for pin_id in orphans {
            if let Err(e) = storage.unpin_checkpoint(pin_id).await {
                tracing::warn!(pin_id = pin_id, error = %e, "Failed to unpin orphaned pin during recovery");
            } else {
                recovered.push(pin_id);
                self.inner.clear_orphan_pin(pin_id);
            }
        }

        Ok(recovered)
    }
}

fn orphan_file_path() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("MEMFUSE_ORPHAN_PATH") {
        return std::path::PathBuf::from(p);
    }
    default_data_dir().join("memfuse_orphaned_checkpoints.json")
}

/// Orphaned gepinnte Sequenznummer — wird beim Recovery verarbeitet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PinnedSeqNoOrphan {
    pub seq_no: u64,
    pub timestamp_ms: u64,
}

#[deprecated(
    since = "0.1.0",
    note = "Use PersistentCheckpointStore::register_pinned_seq_no_orphan instead. Global functions are not safe in multi-instance environments."
)]
#[allow(deprecated)]
pub fn register_pinned_seq_no_orphan(orphan: PinnedSeqNoOrphan) {
    if let Err(e) = global_orphan_registry().register_orphan(orphan.seq_no) {
        tracing::error!(
            ?e,
            seq_no = orphan.seq_no,
            "Failed to register pinned seq_no in global orphan registry (ADR-058)"
        );
    }
}

/// Instance-scoped orphan state for checkpoints and pinned sequence numbers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct OrphanState {
    pub checkpoints: Vec<StateCheckpoint>,
    pub pinned_seq_nos: Vec<PinnedSeqNoOrphan>,
    pub persist_path: std::path::PathBuf,
}

impl OrphanState {
    pub fn persist_sync(&self) -> std::io::Result<()> {
        if self.persist_path.as_os_str().is_empty() {
            return Ok(());
        }
        let data = serde_json::to_vec_pretty(self)?;
        std::fs::write(&self.persist_path, data)
    }

    pub fn load_sync(path: &std::path::Path) -> Self {
        if let Ok(data) = std::fs::read(path) {
            if let Ok(state) = serde_json::from_slice::<OrphanState>(&data) {
                return state;
            }
        }
        Self {
            checkpoints: Vec::new(),
            pinned_seq_nos: Vec::new(),
            persist_path: path.to_path_buf(),
        }
    }
}

/// Instance-scoped orphan registry for checkpoints and pinned sequence numbers (ADR-053).
#[derive(Debug)]
pub struct InstanceOrphanRegistry {
    pins: Mutex<Vec<PinnedSeqNoOrphan>>,
    checkpoints: Mutex<Vec<StateCheckpoint>>,
    persist_path: std::path::PathBuf,
    is_dirty: AtomicBool,
}

impl Default for InstanceOrphanRegistry {
    fn default() -> Self {
        Self::new(orphan_file_path())
    }
}

impl InstanceOrphanRegistry {
    pub fn new(persist_path: impl Into<std::path::PathBuf>) -> Self {
        let path = persist_path.into();
        Self::load_sync(&path)
    }

    pub fn load_sync(path: &std::path::Path) -> Self {
        if !path.as_os_str().is_empty() {
            if let Ok(data) = std::fs::read(path) {
                if let Ok(state) = serde_json::from_slice::<OrphanState>(&data) {
                    return Self {
                        pins: Mutex::new(state.pinned_seq_nos),
                        checkpoints: Mutex::new(state.checkpoints),
                        persist_path: path.to_path_buf(),
                        is_dirty: AtomicBool::new(false),
                    };
                }
            }
        }
        Self {
            pins: Mutex::new(Vec::new()),
            checkpoints: Mutex::new(Vec::new()),
            persist_path: path.to_path_buf(),
            is_dirty: AtomicBool::new(false),
        }
    }

    pub fn persist_sync(&self) -> std::io::Result<()> {
        if self.persist_path.as_os_str().is_empty() {
            return Ok(());
        }
        let state = OrphanState {
            checkpoints: self.checkpoints.lock().clone(),
            pinned_seq_nos: self.pins.lock().clone(),
            persist_path: self.persist_path.clone(),
        };
        let data = serde_json::to_vec_pretty(&state)?;
        std::fs::write(&self.persist_path, data)
    }

    /// Registers a pinned sequence number orphan in-memory without blocking disk I/O.
    ///
    /// # Crash-Safety & RAII Drop Semantics
    /// This method is called synchronously from [`PinGuard::drop`]. It strictly performs
    /// in-memory mutations (`Mutex<Vec<...>>`) and flags the registry as dirty without triggering
    /// synchronous disk I/O on Tokio worker threads. Unpersisted orphans remain in memory and are
    /// persisted during piggyback flushes (e.g., explicit unpin, commit, rollback) or graceful shutdown.
    /// An orphan registered between drops and the next flush is retained in memory and would only be lost
    /// across an ungraceful process panic/crash before the next flush, matching pre-existing crash window semantics.
    pub fn register_orphan_sync(&self, orphan: PinnedSeqNoOrphan) {
        let mut lock = self.pins.lock();
        if !lock.iter().any(|o| o.seq_no == orphan.seq_no) {
            let seq_no = orphan.seq_no;
            lock.push(orphan);
            drop(lock);
            self.is_dirty.store(true, Ordering::Release);
            tracing::debug!(
                seq = seq_no,
                "Registered orphan pin in memory (dirty); awaiting next flush"
            );
        }
    }

    /// Registers an orphaned checkpoint in-memory without blocking disk I/O.
    ///
    /// # Crash-Safety & RAII Drop Semantics
    /// This method is called synchronously from [`CheckpointGuard::drop`]. It strictly performs
    /// in-memory mutations (`Mutex<Vec<...>>`) and flags the registry as dirty without triggering
    /// synchronous disk I/O on Tokio worker threads. Unpersisted orphans remain in memory and are
    /// persisted during piggyback flushes (e.g., explicit unpin, commit, rollback) or graceful shutdown.
    /// An orphan registered between drops and the next flush is retained in memory and would only be lost
    /// across an ungraceful process panic/crash before the next flush, matching pre-existing crash window semantics.
    pub fn register_checkpoint_sync(&self, cp: StateCheckpoint) {
        let mut lock = self.checkpoints.lock();
        if !lock.iter().any(|o| o.tx_id == cp.tx_id) {
            let tx_id = cp.tx_id;
            lock.push(cp);
            drop(lock);
            self.is_dirty.store(true, Ordering::Release);
            tracing::debug!(
                ?tx_id,
                "Registered orphaned checkpoint in memory (dirty); awaiting next flush"
            );
        }
    }

    /// Asynchronously flushes the in-memory orphan registry to disk if dirty.
    pub async fn flush_orphan_registry(&self) -> std::io::Result<()> {
        if self.persist_path.as_os_str().is_empty() {
            return Ok(());
        }
        if !self.is_dirty.swap(false, Ordering::AcqRel) {
            return Ok(());
        }
        let state = OrphanState {
            checkpoints: self.checkpoints.lock().clone(),
            pinned_seq_nos: self.pins.lock().clone(),
            persist_path: self.persist_path.clone(),
        };
        let res = if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::spawn_blocking(move || state.persist_sync())
                .await
                .map_err(|e| std::io::Error::other(e.to_string()))?
        } else {
            state.persist_sync()
        };
        if let Err(err) = &res {
            self.is_dirty.store(true, Ordering::Release);
            tracing::error!(?err, "Failed to persist orphan registry during flush");
        }
        res
    }

    pub fn drain_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan> {
        let mut lock = self.pins.lock();
        let drained = std::mem::take(&mut *lock);
        drop(lock);
        if let Err(err) = self.persist_sync() {
            self.is_dirty.store(true, Ordering::Release);
            tracing::error!(
                ?err,
                "Failed to persist orphan registry after draining pins"
            );
        } else {
            self.is_dirty.store(false, Ordering::Release);
        }
        drained
    }

    pub fn get_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan> {
        self.pins.lock().clone()
    }

    pub fn clear_orphan_pin(&self, seq_no: u64) {
        let mut lock = self.pins.lock();
        lock.retain(|o| o.seq_no != seq_no);
        drop(lock);
        if let Err(err) = self.persist_sync() {
            self.is_dirty.store(true, Ordering::Release);
            tracing::error!(
                ?err,
                seq_no = seq_no,
                "Failed to persist orphan registry after clearing pin"
            );
        } else {
            self.is_dirty.store(false, Ordering::Release);
        }
    }

    pub fn drain_orphaned_checkpoints(&self) -> Vec<StateCheckpoint> {
        let mut lock = self.checkpoints.lock();
        let drained = std::mem::take(&mut *lock);
        drop(lock);
        if let Err(err) = self.persist_sync() {
            self.is_dirty.store(true, Ordering::Release);
            tracing::error!(
                ?err,
                "Failed to persist orphan registry after draining checkpoints"
            );
        } else {
            self.is_dirty.store(false, Ordering::Release);
        }
        drained
    }

    pub fn get_orphaned_checkpoints(&self) -> Vec<StateCheckpoint> {
        self.checkpoints.lock().clone()
    }

    pub fn clear_orphaned_checkpoint(&self, tx_id: TxId) {
        let mut lock = self.checkpoints.lock();
        lock.retain(|o| o.tx_id != tx_id);
        drop(lock);
        if let Err(err) = self.persist_sync() {
            self.is_dirty.store(true, Ordering::Release);
            tracing::error!(?err, tx_id = ?tx_id, "Failed to persist orphan registry after clearing checkpoint");
        } else {
            self.is_dirty.store(false, Ordering::Release);
        }
    }

    pub fn clear_all(&self) {
        self.pins.lock().clear();
        self.checkpoints.lock().clear();
        if let Err(err) = self.persist_sync() {
            self.is_dirty.store(true, Ordering::Release);
            tracing::error!(?err, "Failed to persist orphan registry after clearing all");
        } else {
            self.is_dirty.store(false, Ordering::Release);
        }
    }
}

/// Registers an uncommitted checkpoint as orphaned and persists it to disk.
#[deprecated(
    since = "0.1.0",
    note = "Use PersistentCheckpointStore::register_orphaned_checkpoint instead. Global functions are not safe in multi-instance environments."
)]
#[allow(deprecated)]
pub fn register_orphaned_checkpoint(cp: StateCheckpoint) {
    global_orphan_registry().inner.register_checkpoint_sync(cp);
}

/// Retrieves all active registered orphaned checkpoints.
#[deprecated(
    since = "0.1.0",
    note = "Use PersistentCheckpointStore::get_orphaned_checkpoints instead. Global functions are not safe in multi-instance environments."
)]
#[allow(deprecated)]
pub fn get_orphaned_checkpoints() -> Vec<StateCheckpoint> {
    global_orphan_registry().inner.get_orphaned_checkpoints()
}

/// Retrieves orphaned checkpoints registered for a specific namespace.
#[allow(deprecated)]
pub fn get_orphaned_checkpoints_for_namespace(ns: &str) -> Vec<StateCheckpoint> {
    get_orphaned_checkpoints()
        .into_iter()
        .filter(|cp| cp.namespace.as_deref() == Some(ns))
        .collect()
}

/// Removes a specific orphaned checkpoint after recovery.
#[deprecated(
    since = "0.1.0",
    note = "Use PersistentCheckpointStore::clear_orphaned_checkpoint instead. Global functions are not safe in multi-instance environments."
)]
#[allow(deprecated)]
pub fn clear_orphaned_checkpoint(tx_id: TxId) {
    global_orphan_registry()
        .inner
        .clear_orphaned_checkpoint(tx_id);
}

/// Clears all registered orphaned checkpoints.
#[deprecated(
    since = "0.1.0",
    note = "Use PersistentCheckpointStore::clear_all_orphaned_checkpoints instead. Global functions are not safe in multi-instance environments."
)]
#[allow(deprecated)]
pub fn clear_all_orphaned_checkpoints() {
    global_orphan_registry().inner.clear_all();
}

/// Retained for backward compatibility. No background tasks are spawned during drop.
pub async fn await_pending_rollbacks() {}

/// Retained for backward compatibility. Always returns 0 as background task spawning is removed.
pub fn pending_rollback_count() -> usize {
    0
}

/// Liefert die Anzahl der aktuell registrierten verwaisten ("orphaned") Checkpoints.
#[allow(deprecated)]
pub fn orphaned_checkpoint_count() -> usize {
    #[allow(deprecated)]
    get_orphaned_checkpoints().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_GLOBAL_ORPHAN_MUTEX: parking_lot::Mutex<()> = parking_lot::Mutex::new(());

    #[test]
    fn test_deprecated_global_orphan_path_warns() {
        let _guard = TEST_GLOBAL_ORPHAN_MUTEX.lock();
        #[allow(deprecated)]
        {
            clear_all_orphaned_checkpoints();
            register_pinned_seq_no_orphan(PinnedSeqNoOrphan {
                seq_no: 99999,
                timestamp_ms: 1000,
            });
            let _orphans = get_orphaned_checkpoints();
            clear_all_orphaned_checkpoints();
        }
    }

    #[test]
    fn timestamp_ms_is_monotonic() {
        let t1 = monotonic_timestamp_ms();
        let t2 = monotonic_timestamp_ms();
        assert!(t2 >= t1, "Timestamp must be monotonic");
    }

    #[test]
    fn test_instance_orphan_registry_drain_pins_and_checkpoints() {
        let registry = InstanceOrphanRegistry::new("");
        registry.register_orphan_sync(PinnedSeqNoOrphan {
            seq_no: 100,
            timestamp_ms: 1000,
        });
        registry.register_checkpoint_sync(StateCheckpoint {
            tx_id: TxId::new(200),
            timestamp_ms: 2000,
            namespace: Some("test_drain".to_string()),
        });

        assert_eq!(registry.get_orphan_pins().len(), 1);
        assert_eq!(registry.get_orphaned_checkpoints().len(), 1);

        let drained_pins = registry.drain_orphan_pins();
        assert_eq!(drained_pins.len(), 1);
        assert_eq!(drained_pins[0].seq_no, 100);
        assert!(registry.get_orphan_pins().is_empty());

        let drained_cps = registry.drain_orphaned_checkpoints();
        assert_eq!(drained_cps.len(), 1);
        assert_eq!(drained_cps[0].tx_id, TxId::new(200));
        assert!(registry.get_orphaned_checkpoints().is_empty());
    }

    proptest::proptest! {
        #[test]
        fn prop_monotonic_timestamp_ms_increases_or_equals(_n: u8) {
            let ts1 = monotonic_timestamp_ms();
            let ts2 = monotonic_timestamp_ms();
            proptest::prop_assert!(ts2 >= ts1);
        }
    }
}
