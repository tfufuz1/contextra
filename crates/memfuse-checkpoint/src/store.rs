use crate::guard::{CheckpointGuard, PinGuard};
use crate::manifest::CheckpointManifest;
use crate::meta::{validate_identifier, CheckpointMeta, StateCheckpoint};
use crate::orphan::{monotonic_timestamp_ms, InstanceOrphanRegistry, PinId, PinnedSeqNoOrphan};
use memfuse_core::{BoxFuture, MemFuseError, Result, TxId, WorkflowState};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Trait für die Checkpoint-Verwaltung.
pub trait CheckpointRegistry: memfuse_core::traits::Checkpoint + Send + Sync {
    fn save_checkpoint<'a>(&'a self, meta: CheckpointMeta) -> BoxFuture<'a, Result<()>>;
    fn load_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<Option<CheckpointMeta>>>;
    fn list_checkpoints<'a>(&'a self) -> BoxFuture<'a, Result<Vec<CheckpointMeta>>>;

    fn orphan_registry(&self) -> Option<&Arc<InstanceOrphanRegistry>> {
        None
    }

    fn recover_orphaned_pins<'a>(&'a self) -> BoxFuture<'a, Result<Vec<PinId>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn recover_orphaned_checkpoints<'a>(&'a self) -> BoxFuture<'a, Result<Vec<TxId>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn flush_orphan_registry<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
}

#[derive(Debug, Default)]
struct CheckpointIndex {
    by_seq: HashMap<u64, CheckpointMeta>,
    by_name: HashMap<String, u64>,
}

impl CheckpointIndex {
    fn insert(&mut self, meta: CheckpointMeta) {
        if let Some(old_seq) = self.by_name.insert(meta.name.clone(), meta.seq_no) {
            if old_seq != meta.seq_no {
                self.by_seq.remove(&old_seq);
            }
        }
        self.by_seq.insert(meta.seq_no, meta);
    }

    fn remove_by_name(&mut self, name: &str) -> Option<CheckpointMeta> {
        if let Some(seq_no) = self.by_name.remove(name) {
            self.by_seq.remove(&seq_no)
        } else {
            None
        }
    }

    fn remove_by_seq(&mut self, seq_no: u64) -> Option<CheckpointMeta> {
        if let Some(meta) = self.by_seq.remove(&seq_no) {
            if self.by_name.get(&meta.name) == Some(&seq_no) {
                self.by_name.remove(&meta.name);
            }
            Some(meta)
        } else {
            None
        }
    }

    fn get_by_name(&self, name: &str) -> Option<&CheckpointMeta> {
        self.by_name
            .get(name)
            .and_then(|seq_no| self.by_seq.get(seq_no))
    }

    fn get_by_seq(&self, seq_no: u64) -> Option<&CheckpointMeta> {
        self.by_seq.get(&seq_no)
    }

    fn clear(&mut self) {
        self.by_seq.clear();
        self.by_name.clear();
    }
}

/// Counter metadata persisted to guarantee TxId monotonicity across restarts.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TxCounterMeta {
    pub high_water_mark: u64,
}

const TX_BATCH_SIZE: u64 = 100;

async fn persist_hwm_internal<S: memfuse_core::StorageEngine>(
    storage: &Arc<S>,
    namespace: &str,
    hwm: u64,
) -> Result<()> {
    let meta = TxCounterMeta {
        high_water_mark: hwm,
    };
    let bytes =
        serde_json::to_vec(&meta).map_err(|e| MemFuseError::Serialization(e.to_string()))?;
    let key = format!("{namespace}:checkpoint:__sys_tx_counter__");
    let tx = TxId::new(TxId::INTERNAL_BASE + hwm);
    storage.put(tx, key.as_bytes(), &bytes).await?;
    storage.commit(tx).await?;
    storage.flush().await?;
    Ok(())
}

/// Registry für gespeicherte Checkpoints mit Thread-sicherem Zustand.
///
/// # Invarianten
/// - Alle Methoden sind durch `RwLock` thread-sicher
/// - `StorageEngine`-Zugriffe nutzen atomare Transaktionen via `TxId`
/// - Keine Panics (Zero-Panic Doctrine)
pub struct PersistentCheckpointStore<S: memfuse_core::StorageEngine> {
    storage: Arc<S>,
    /// Registrierter Checkpoint-Index im Arbeitsspeicher — geschützt durch ein konsolidiertes RwLock
    index: RwLock<CheckpointIndex>,
    /// Namespace-Präfix für Storage-Keys
    namespace: String,
    /// Lock für sequentielle Schreiboperationen auf den Storage (HIGH-002)
    write_lock: tokio::sync::Mutex<()>,
    /// Atomarer Zähler für interne TxIds (vermeidet Kollisionen)
    tx_counter: AtomicU64,
    /// Reservierter High-Water-Mark Wert in persistentem Storage
    allocated_hwm: AtomicU64,
    /// Lock für HWM-Reservierung und Persistierung
    hwm_lock: tokio::sync::Mutex<()>,
    /// Instanz-spezifischer Orphan Registry
    orphan_registry: Arc<InstanceOrphanRegistry>,
    /// Instanz-spezifischer Checkpoint Counter für monotone Zeitstempel
    checkpoint_counter: Arc<AtomicU64>,
    /// Instanz-spezifischer Zähler für verpasste Rollbacks
    skipped_rollbacks: Arc<AtomicU64>,
}

impl<S: memfuse_core::StorageEngine> PersistentCheckpointStore<S> {
    /// Öffnet einen PersistentCheckpointStore asynchron mit Rekonstruktion und Monotonie-Garantie.
    pub async fn open(storage: Arc<S>, namespace: impl Into<String>) -> Result<Self> {
        let ns = namespace.into();
        let orphan_path = std::path::PathBuf::from(format!("{ns}_orphaned_checkpoints.json"));
        let orphan_registry = Arc::new(InstanceOrphanRegistry::new(&orphan_path));
        Self::open_with_orphan_registry(storage, ns, orphan_registry).await
    }

    /// Öffnet einen PersistentCheckpointStore mit einer spezifischen instanzgebundenen Orphan Registry.
    pub async fn open_with_orphan_registry(
        storage: Arc<S>,
        namespace: impl Into<String>,
        orphan_registry: Arc<InstanceOrphanRegistry>,
    ) -> Result<Self> {
        let namespace = namespace.into();

        // 1. Scan store for highest existing TxId under namespace
        let prefix = format!("{namespace}:checkpoint:");
        let entries = storage.scan_prefix(prefix.as_bytes()).await?;
        let mut scanned_max_raw: Option<u64> = None;

        for (_key, value_bytes) in entries {
            let meta_tx =
                if let Ok(manifest) = serde_json::from_slice::<CheckpointManifest>(&value_bytes) {
                    Some(manifest.meta.tx_id)
                } else if let Ok(meta) = serde_json::from_slice::<CheckpointMeta>(&value_bytes) {
                    Some(meta.tx_id)
                } else {
                    None
                };

            if let Some(tx) = meta_tx {
                if tx.inner() >= TxId::INTERNAL_BASE {
                    let raw = tx.inner() - TxId::INTERNAL_BASE;
                    scanned_max_raw = Some(scanned_max_raw.map_or(raw, |m| m.max(raw)));
                }
            }
        }

        if let Ok(last_tx) = storage.last_tx_id().await {
            if last_tx.inner() >= TxId::INTERNAL_BASE {
                let raw = last_tx.inner() - TxId::INTERNAL_BASE;
                scanned_max_raw = Some(scanned_max_raw.map_or(raw, |m| m.max(raw)));
            }
        }

        // 2. Read persisted counter metadata
        let counter_key = format!("{namespace}:checkpoint:__sys_tx_counter__");
        let persisted_val: Option<u64> = match storage.get(counter_key.as_bytes()).await {
            Ok(Some(bytes)) => serde_json::from_slice::<TxCounterMeta>(&bytes)
                .map(|m| m.high_water_mark)
                .ok(),
            _ => None,
        };

        // 3. Consistency check: if persisted value exists and is LESS THAN scanned max raw -> Hard Error (Requirement 4)
        if let (Some(persisted), Some(scanned)) = (persisted_val, scanned_max_raw) {
            if persisted < scanned {
                return Err(MemFuseError::Internal(format!(
                    "TxId collision / regression detected in namespace '{namespace}': \
                     persisted tx_counter HWM ({persisted}) is strictly less than highest tx_id found in store ({scanned})"
                )));
            }
        }

        // 4. Determine initial start_raw
        let start_raw = match (persisted_val, scanned_max_raw) {
            (Some(p), _) => p + 1,
            (None, Some(s)) => s + 1,
            (None, None) => 0,
        };

        let initial_hwm = persisted_val.unwrap_or_else(|| scanned_max_raw.unwrap_or(0));

        // 5. Recover orphaned sequence pins on startup (ADR-052)
        let orphans = orphan_registry.get_orphan_pins();
        for orphan in orphans {
            if let Err(e) = storage.unpin_checkpoint(orphan.seq_no).await {
                tracing::warn!(pin_id = orphan.seq_no, error = %e, "Failed to unpin orphaned pin during store startup recovery");
            } else {
                orphan_registry.clear_orphan_pin(orphan.seq_no);
            }
        }

        let checkpoint_counter = Arc::new(AtomicU64::new(0));
        let skipped_rollbacks = Arc::new(AtomicU64::new(0));

        Ok(Self {
            storage,
            index: RwLock::new(CheckpointIndex::default()),
            namespace,
            write_lock: tokio::sync::Mutex::new(()),
            tx_counter: AtomicU64::new(start_raw),
            allocated_hwm: AtomicU64::new(initial_hwm),
            hwm_lock: tokio::sync::Mutex::new(()),
            orphan_registry,
            checkpoint_counter,
            skipped_rollbacks,
        })
    }

    pub fn new(storage: Arc<S>, namespace: impl Into<String>) -> Result<Self> {
        let ns = namespace.into();
        let orphan_path = std::path::PathBuf::from(format!("{ns}_orphaned_checkpoints.json"));
        let orphan_registry = Arc::new(InstanceOrphanRegistry::new(&orphan_path));
        Self::new_with_orphan_registry(storage, ns, orphan_registry)
    }

    pub fn new_with_orphan_registry(
        storage: Arc<S>,
        namespace: impl Into<String>,
        orphan_registry: Arc<InstanceOrphanRegistry>,
    ) -> Result<Self> {
        let ns = namespace.into();
        let storage_clone = storage.clone();
        let ns_clone = ns.clone();
        let orphan_reg_clone = orphan_registry.clone();

        let res = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
                tokio::task::block_in_place(|| {
                    handle.block_on(Self::open_with_orphan_registry(
                        storage_clone,
                        ns_clone,
                        orphan_reg_clone,
                    ))
                })
            } else {
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .map_err(|e| MemFuseError::Internal(e.to_string()))?;
                    rt.block_on(Self::open_with_orphan_registry(
                        storage_clone,
                        ns_clone,
                        orphan_reg_clone,
                    ))
                })
                .join()
                .map_err(|_| {
                    MemFuseError::Internal(
                        "Thread panic during PersistentCheckpointStore initialization".into(),
                    )
                })
                .and_then(|r| r)
            }
        } else {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| {
                    MemFuseError::Internal(format!("Failed to create Tokio runtime: {e}"))
                })?;
            rt.block_on(Self::open_with_orphan_registry(
                storage_clone,
                ns_clone,
                orphan_reg_clone,
            ))
        };

        res.map_err(|e| {
            MemFuseError::Internal(format!(
                "Failed to initialize PersistentCheckpointStore for namespace '{ns}': {e}"
            ))
        })
    }

    // INVARIANT: Checkpoint TxIds use INTERNAL_BASE+n range to avoid
    // collision with Collection-sequenced TxIds [1, ~10^12].
    // See: DECISIONS.md AGT-GRAPH-001, TxId::INTERNAL_BASE
    pub async fn allocate_tx(&self) -> Result<TxId> {
        let raw = self.tx_counter.fetch_add(1, Ordering::SeqCst);
        if raw >= 1_000_000 {
            return Err(MemFuseError::Internal(
                "Checkpoint TxId counter overflow".to_string(),
            ));
        }

        let current_hwm = self.allocated_hwm.load(Ordering::SeqCst);
        if raw >= current_hwm {
            let _guard = self.hwm_lock.lock().await;
            let active_hwm = self.allocated_hwm.load(Ordering::SeqCst);
            if raw >= active_hwm {
                let new_hwm = raw + TX_BATCH_SIZE - 1;
                persist_hwm_internal(&self.storage, &self.namespace, new_hwm).await?;
                self.allocated_hwm.store(new_hwm, Ordering::SeqCst);
            }
        }

        Ok(TxId::new(TxId::INTERNAL_BASE + raw))
    }

    #[deprecated(
        since = "0.1.0",
        note = "Use `allocate_tx()` instead — both methods are functionally identical, `allocate_tx()` is the canonical public API."
    )]
    #[allow(dead_code)]
    async fn next_tx(&self) -> Result<TxId> {
        self.allocate_tx().await
    }

    pub fn skipped_rollback_count(&self) -> u64 {
        self.skipped_rollbacks.load(Ordering::Relaxed)
    }

    pub fn checkpoint_guard_skipped_rollback_count(&self) -> u64 {
        self.skipped_rollback_count()
    }

    pub fn checkpoint_counter(&self) -> u64 {
        self.checkpoint_counter.load(Ordering::Relaxed)
    }

    pub fn monotonic_timestamp_ms(&self) -> u64 {
        let wall_ms = monotonic_timestamp_ms();
        self.checkpoint_counter
            .fetch_max(wall_ms, Ordering::SeqCst)
            .max(wall_ms)
    }

    /// Creates an ephemeral transactional checkpoint RAII guard.
    /// If the returned guard is dropped without calling `.commit()`, the underlying storage
    /// is automatically rolled back to `tx_id`.
    pub fn create_guard(&self, tx_id: TxId) -> Result<CheckpointGuard<S>> {
        let timestamp_ms = self.monotonic_timestamp_ms();
        let cp = StateCheckpoint {
            tx_id,
            timestamp_ms,
            namespace: Some(self.namespace.clone()),
        };
        Ok(CheckpointGuard::with_registry_and_counter(
            cp,
            Arc::clone(&self.storage),
            &self.namespace,
            Arc::clone(&self.orphan_registry),
            Arc::clone(&self.skipped_rollbacks),
        ))
    }

    /// Creates a new persistent checkpoint.
    pub async fn create_checkpoint(
        &self,
        name: &str,
        collection_id: &str,
        seq_no: u64,
        tx_id: TxId,
        metadata: serde_json::Value,
    ) -> Result<CheckpointMeta> {
        validate_identifier("Checkpoint name", name)?;
        validate_identifier("Collection ID", collection_id)?;

        let meta = CheckpointMeta {
            name: name.to_string(),
            collection_id: collection_id.to_string(),
            seq_no,
            tx_id,
            metadata,
            created_at: self.monotonic_timestamp_ms(),
        };

        let _guard = self.write_lock.lock().await;

        // Lade alten Checkpoint (für späteres Unpin)
        let old_checkpoint = self.get_checkpoint_internal(name).await?;

        // 1. Pin new checkpoint's seq_no (MUST happen BEFORE storage.save())
        let pin_guard = PinGuard::pin(
            Arc::clone(&self.storage),
            seq_no,
            Arc::clone(&self.orphan_registry),
        )
        .await?;

        // 2. storage.save() the new checkpoint (RAII PinGuard will unpin on drop/panic or explicit unpin on error)
        if let Err(e) = self.save_checkpoint_internal(meta.clone()).await {
            if let Err(unpin_err) = pin_guard.unpin().await {
                tracing::warn!(
                    seq = seq_no,
                    "Failed to unpin new checkpoint after save failure: {unpin_err}"
                );
            }
            return Err(e);
        }

        // 3. Save succeeded: defuse pin_guard so seq_no remains pinned
        pin_guard.defuse();

        // 4. If save() succeeds: unpin the old checkpoint's seq_no
        if let Some(old) = old_checkpoint {
            if old.seq_no != seq_no {
                if let Err(e) = self.storage.unpin_checkpoint(old.seq_no).await {
                    // INTENTIONAL: Unpin of the old checkpoint failed. This is non-fatal —
                    // the old seq_no remains pinned, delaying SSTable GC but not causing
                    // data loss. The orphaned pin will clear when the collection is reopened
                    // or when the old checkpoint is explicitly dropped.
                    tracing::warn!(
                        old_seq = old.seq_no,
                        "Konnte alten Checkpoint nicht entpinnen: {e}"
                    );
                }
                // Alten Checkpoint aus Cache entfernen
                self.index.write().remove_by_seq(old.seq_no);
            }
        }

        // 5. Update the stored checkpoint reference
        self.index.write().insert(meta.clone());

        Ok(meta)
    }

    /// Deletes a persistent checkpoint by name.
    pub async fn drop_checkpoint(&self, name: &str) -> Result<()> {
        validate_identifier("Checkpoint name", name)?;
        let _guard = self.write_lock.lock().await;

        if let Some(checkpoint) = self.get_checkpoint_internal(name).await? {
            // 1. Zuerst aus Storage löschen (mit eindeutiger TxId)
            let key = format!("{}:checkpoint:{}", self.namespace, name);

            // FIX CHK-002: Generiere eine eindeutige TxId statt INTERNAL_BASE
            let unique_tx = self.allocate_tx().await?;

            if let Err(e) = self.storage.delete(unique_tx, key.as_bytes()).await {
                if let Err(rb_err) = self.storage.rollback(unique_tx).await {
                    tracing::warn!(tx = ?unique_tx, error = %rb_err, "Storage rollback failed during drop_checkpoint delete");
                }
                return Err(e);
            }
            if let Err(e) = self.storage.commit(unique_tx).await {
                if let Err(rb_err) = self.storage.rollback(unique_tx).await {
                    tracing::warn!(tx = ?unique_tx, error = %rb_err, "Storage rollback failed during drop_checkpoint commit");
                }
                return Err(e);
            }

            // 2. Erst nach erfolgreichem Storage-Delete entpinnen
            if let Err(e) = self.storage.unpin_checkpoint(checkpoint.seq_no).await {
                tracing::warn!(
                    seq = checkpoint.seq_no,
                    "Unpin nach drop fehlgeschlagen: {e}"
                );
            }

            // 3. Cache bereinigen
            self.index.write().remove_by_name(&checkpoint.name);
        }
        Ok(())
    }

    /// Helper for internal saving logic. Uses name as key for uniqueness.
    async fn save_checkpoint_internal(&self, meta: CheckpointMeta) -> Result<()> {
        let key = format!("{}:checkpoint:{}", self.namespace, meta.name);
        let manifest = CheckpointManifest::new(meta.clone(), vec!["storage".to_string()])?;
        let value = serde_json::to_vec(&manifest)
            .map_err(|e| MemFuseError::Serialization(e.to_string()))?;

        let tx = self.allocate_tx().await?;
        if let Err(e) = self.storage.put(tx, key.as_bytes(), &value).await {
            if let Err(rb_err) = self.storage.rollback(tx).await {
                tracing::warn!(tx = ?tx, error = %rb_err, "Storage rollback failed during save_checkpoint_internal put");
            }
            return Err(e);
        }
        if let Err(e) = self.storage.commit(tx).await {
            if let Err(rb_err) = self.storage.rollback(tx).await {
                tracing::warn!(tx = ?tx, error = %rb_err, "Storage rollback failed during save_checkpoint_internal commit");
            }
            return Err(e);
        }

        // In-Memory Cache aktualisieren
        self.index.write().insert(meta);
        Ok(())
    }

    /// Internal helper to get checkpoint by name without extra locking.
    async fn get_checkpoint_internal(&self, name: &str) -> Result<Option<CheckpointMeta>> {
        // Erst O(1) In-Memory Name-Index prüfen (unter EINEM Read-Lock)
        if let Some(cp) = self.index.read().get_by_name(name) {
            return Ok(Some(cp.clone()));
        }

        // Storage direkt fragen
        let key = format!("{}:checkpoint:{}", self.namespace, name);
        match self.storage.get(key.as_bytes()).await? {
            Some(bytes) => {
                let meta =
                    if let Ok(manifest) = serde_json::from_slice::<CheckpointManifest>(&bytes) {
                        manifest.verify()?;
                        manifest.meta
                    } else {
                        serde_json::from_slice::<CheckpointMeta>(&bytes)
                            .map_err(|e| MemFuseError::Serialization(e.to_string()))?
                    };
                self.index.write().insert(meta.clone());
                Ok(Some(meta))
            }
            None => Ok(None),
        }
    }

    /// Public inherent methods for compatibility
    pub async fn list_checkpoints(&self) -> Result<Vec<CheckpointMeta>> {
        let prefix = format!("{}:checkpoint:", self.namespace);
        let entries: Vec<(Vec<u8>, Vec<u8>)> = self.storage.scan_prefix(prefix.as_bytes()).await?;

        let mut result = Vec::with_capacity(entries.len());
        for (key_bytes, value_bytes) in entries {
            if key_bytes.ends_with(b":__sys_tx_counter__") {
                continue;
            }
            let meta =
                if let Ok(manifest) = serde_json::from_slice::<CheckpointManifest>(&value_bytes) {
                    manifest.verify()?;
                    manifest.meta
                } else {
                    serde_json::from_slice::<CheckpointMeta>(&value_bytes)
                        .map_err(|e| MemFuseError::Serialization(e.to_string()))?
                };
            result.push(meta);
        }

        // Cache synchronisieren
        {
            let mut idx = self.index.write();
            idx.clear();
            for meta in &result {
                idx.insert(meta.clone());
            }
        }

        result.sort_by_key(|m| m.seq_no);
        Ok(result)
    }

    pub async fn get_checkpoint(&self, name: &str) -> Result<Option<CheckpointMeta>> {
        validate_identifier("Checkpoint name", name)?;
        self.get_checkpoint_internal(name).await
    }

    /// Restores the system to a specific checkpoint by name.
    /// This will rollback the underlying storage to the transaction ID of the checkpoint.
    ///
    /// # Serialisierungsbarriere
    /// Wenn neuere committete Transaktionen (`last_tx > meta.tx_id`) existieren, schlägt die Wiederherstellung
    /// mit einem Fehler fehl.
    pub async fn restore_checkpoint(&self, name: &str) -> Result<CheckpointMeta> {
        validate_identifier("Checkpoint name", name)?;
        let _guard = self.write_lock.lock().await;

        let meta = self
            .get_checkpoint_internal(name)
            .await?
            .ok_or(MemFuseError::CheckpointNotFound)?;

        let last_tx = self.storage.last_tx_id().await?;
        if last_tx > meta.tx_id {
            return Err(MemFuseError::Transaction(format!(
                "Serialization barrier violation: Cannot restore checkpoint '{}' at TxId {} because newer committed transaction TxId {} exists in storage",
                name,
                meta.tx_id.inner(),
                last_tx.inner()
            )));
        }

        // 1. Rollback storage state
        self.storage.rollback_to_tx(meta.tx_id).await?;

        // 2. Synchronize cache
        self.list_checkpoints().await?;

        Ok(meta)
    }

    pub fn orphan_registry(&self) -> &Arc<InstanceOrphanRegistry> {
        &self.orphan_registry
    }

    pub async fn flush_orphan_registry(&self) -> Result<()> {
        self.orphan_registry
            .flush_orphan_registry()
            .await
            .map_err(|e| MemFuseError::Internal(format!("Failed to flush orphan registry: {e}")))
    }

    /// Performs an explicit shutdown flush of the orphan registry to disk.
    pub async fn shutdown(&self) -> Result<()> {
        self.flush_orphan_registry().await
    }

    /// Alias for `shutdown()`. Performs an explicit graceful close flush.
    pub async fn close(&self) -> Result<()> {
        self.shutdown().await
    }

    pub fn register_pinned_seq_no_orphan(&self, orphan: PinnedSeqNoOrphan) {
        self.orphan_registry.register_orphan_sync(orphan);
    }

    pub fn register_orphaned_checkpoint(&self, cp: StateCheckpoint) {
        self.orphan_registry.register_checkpoint_sync(cp);
    }

    pub fn get_orphaned_checkpoints(&self) -> Vec<StateCheckpoint> {
        self.orphan_registry.get_orphaned_checkpoints()
    }

    pub fn clear_orphaned_checkpoint(&self, tx_id: TxId) {
        self.orphan_registry.clear_orphaned_checkpoint(tx_id);
    }

    pub fn clear_all_orphaned_checkpoints(&self) {
        self.orphan_registry.clear_all();
    }

    /// Recovers all registered/persisted orphaned sequence pins (ADR-052).
    pub async fn recover_orphaned_pins(&self) -> Result<Vec<PinId>> {
        let orphans = self.orphan_registry.get_orphan_pins();
        let mut recovered = Vec::new();
        for orphan in orphans {
            if let Err(e) = self.storage.unpin_checkpoint(orphan.seq_no).await {
                tracing::warn!(pin_id = orphan.seq_no, error = %e, "Failed to unpin orphaned pin during recovery");
            } else {
                recovered.push(orphan.seq_no);
                self.orphan_registry.clear_orphan_pin(orphan.seq_no);
            }
        }
        Ok(recovered)
    }

    /// Recovers all registered/persisted orphaned checkpoints during controlled startup or recovery.
    /// Checks the serialization barrier (`last_tx <= cp.tx_id`) before executing rollback.
    pub async fn recover_orphaned_checkpoints(&self) -> Result<Vec<TxId>> {
        let _guard = self.write_lock.lock().await;
        let orphans = self.get_orphaned_checkpoints();

        let mut recovered = Vec::new();
        let last_tx = self.storage.last_tx_id().await?;

        for cp in orphans {
            if last_tx <= cp.tx_id {
                if let Err(e) = self.storage.rollback_to_tx(cp.tx_id).await {
                    tracing::error!(tx_id = ?cp.tx_id, "Failed to recover orphaned checkpoint: {e}");
                } else {
                    recovered.push(cp.tx_id);
                    self.clear_orphaned_checkpoint(cp.tx_id);
                }
            } else {
                tracing::warn!(
                    tx_id = ?cp.tx_id,
                    last_tx = ?last_tx,
                    "Orphaned checkpoint skipped during recovery due to serialization barrier (newer transaction committed)"
                );
                self.clear_orphaned_checkpoint(cp.tx_id);
            }
        }

        Ok(recovered)
    }
}

impl<S: memfuse_core::StorageEngine> CheckpointRegistry for PersistentCheckpointStore<S> {
    fn save_checkpoint<'a>(&'a self, meta: CheckpointMeta) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let _guard = self.write_lock.lock().await;
            self.save_checkpoint_internal(meta).await
        })
    }

    fn load_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<Option<CheckpointMeta>>> {
        Box::pin(async move {
            if let Some(meta) = self.index.read().get_by_seq(seq_no) {
                return Ok(Some(meta.clone()));
            }

            let all = self.list_checkpoints().await?;
            Ok(all.into_iter().find(|c| c.seq_no == seq_no))
        })
    }

    fn list_checkpoints<'a>(&'a self) -> BoxFuture<'a, Result<Vec<CheckpointMeta>>> {
        Box::pin(async move { self.list_checkpoints().await })
    }

    fn orphan_registry(&self) -> Option<&Arc<InstanceOrphanRegistry>> {
        Some(&self.orphan_registry)
    }

    fn recover_orphaned_pins<'a>(&'a self) -> BoxFuture<'a, Result<Vec<PinId>>> {
        Box::pin(async move { self.recover_orphaned_pins().await })
    }

    fn recover_orphaned_checkpoints<'a>(&'a self) -> BoxFuture<'a, Result<Vec<TxId>>> {
        Box::pin(async move { self.recover_orphaned_checkpoints().await })
    }

    fn flush_orphan_registry<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { self.flush_orphan_registry().await })
    }
}

impl<S: memfuse_core::StorageEngine> memfuse_core::traits::CheckpointCoordinator
    for PersistentCheckpointStore<S>
{
    type Meta = CheckpointMeta;

    async fn create_named_checkpoint(
        &self,
        name: &str,
        collection_id: &str,
        seq_no: u64,
        tx_id: TxId,
        metadata: serde_json::Value,
    ) -> Result<Self::Meta> {
        self.create_checkpoint(name, collection_id, seq_no, tx_id, metadata)
            .await
    }

    async fn restore_named_checkpoint(&self, name: &str) -> Result<Self::Meta> {
        self.restore_checkpoint(name).await
    }

    async fn drop_named_checkpoint(&self, name: &str) -> Result<()> {
        self.drop_checkpoint(name).await
    }

    async fn list_named_checkpoints(&self) -> Result<Vec<Self::Meta>> {
        self.list_checkpoints().await
    }
}

impl<S: memfuse_core::StorageEngine> memfuse_core::traits::Checkpoint
    for PersistentCheckpointStore<S>
{
    fn take_snapshot<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<WorkflowState>> {
        Box::pin(async move {
            let seq_no = self.storage.last_seq_no().await?;
            Ok(WorkflowState {
                tx,
                graph_hash: *blake3::hash(format!("seq-{}", seq_no).as_bytes()).as_bytes(),
            })
        })
    }

    fn restore<'a>(&'a self, state: &'a WorkflowState) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.storage.rollback_to_tx(state.tx).await?;
            self.list_checkpoints().await?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orphan::pending_rollback_count;
    use memfuse_core::{BoxFuture, StorageEngine, StorageStats};
    use parking_lot::Mutex;
    use std::collections::HashSet;

    struct MockStorage {
        data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
        pinned: Mutex<HashSet<u64>>,
        fail_on_put: Mutex<Option<Vec<u8>>>,
        rolled_back_tx: Mutex<Vec<TxId>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                data: Mutex::new(HashMap::new()),
                pinned: Mutex::new(HashSet::new()),
                fail_on_put: Mutex::new(None),
                rolled_back_tx: Mutex::new(Vec::new()),
            }
        }
    }

    impl StorageEngine for MockStorage {
        fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
            Box::pin(async move { Ok(self.data.lock().get(key).cloned().map(bytes::Bytes::from)) })
        }
        fn get_at_seq<'a>(
            &'a self,
            key: &'a [u8],
            _seq: u64,
        ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
            Box::pin(async move { self.get(key).await })
        }
        fn put<'a>(
            &'a self,
            _tx_id: TxId,
            key: &'a [u8],
            value: &'a [u8],
        ) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                if let Some(fail_key) = self.fail_on_put.lock().as_ref() {
                    if key == fail_key {
                        return Err(MemFuseError::Internal("Mock Storage Error".to_string()));
                    }
                }
                self.data.lock().insert(key.to_vec(), value.to_vec());
                Ok(())
            })
        }
        fn delete<'a>(&'a self, _tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                self.data.lock().remove(key);
                Ok(())
            })
        }
        fn commit<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn rollback<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                self.rolled_back_tx.lock().push(tx_id);
                Ok(())
            })
        }
        fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
            Box::pin(async move { Ok(0) })
        }
        fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
            Box::pin(async move { Ok(TxId::new(0)) })
        }
        fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
            Box::pin(async move {
                Ok(StorageStats {
                    num_segments: 0,
                    total_size_bytes: 0,
                    memtable_size_bytes: 0,
                })
            })
        }
        fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                self.pinned.lock().insert(seq_no);
                Ok(())
            })
        }
        fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                self.pinned.lock().remove(&seq_no);
                Ok(())
            })
        }
        fn scan_prefix<'a>(
            &'a self,
            prefix: &'a [u8],
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move {
                let data = self.data.lock();
                Ok(data
                    .iter()
                    .filter(|(k, _)| k.starts_with(prefix))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect())
            })
        }
        fn scan<'a>(
            &'a self,
            _s: std::ops::Bound<&'a [u8]>,
            _e: std::ops::Bound<&'a [u8]>,
            _: Option<usize>,
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(Vec::new()) })
        }
    }

    #[tokio::test]
    async fn test_create_and_load() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage, "test").unwrap();
        let meta = store
            .create_checkpoint("cp1", "c1", 1, TxId::new(1), serde_json::json!({}))
            .await
            .unwrap(); // unwrap
        let loaded = CheckpointRegistry::load_checkpoint(&store, 1)
            .await
            .unwrap()
            .unwrap(); // unwrap
        assert_eq!(loaded, meta);
    }

    #[tokio::test]
    async fn test_name_uniqueness() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();
        store
            .create_checkpoint("same", "c1", 1, TxId::new(1), serde_json::json!({}))
            .await
            .unwrap(); // unwrap
        store
            .create_checkpoint("same", "c1", 2, TxId::new(2), serde_json::json!({}))
            .await
            .unwrap(); // unwrap
        let all = store.list_checkpoints().await.unwrap(); // unwrap
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].seq_no, 2);
        assert!(!storage.pinned.lock().contains(&1));
        assert!(storage.pinned.lock().contains(&2));
    }

    #[tokio::test]
    async fn test_checkpoint_creation_rollback_on_failure() {
        let storage = Arc::new(MockStorage::new());
        let cp_key = b"test:checkpoint:fail_cp";
        *storage.fail_on_put.lock() = Some(cp_key.to_vec());

        let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();
        let seq_no = 123;

        let res = store
            .create_checkpoint("fail_cp", "c1", seq_no, TxId::new(1), serde_json::json!({}))
            .await;

        assert!(res.is_err());
        assert!(!storage.pinned.lock().contains(&seq_no));
    }

    #[tokio::test]
    async fn test_orphan_recovery_on_startup() {
        let storage = Arc::new(MockStorage::new());
        let store =
            PersistentCheckpointStore::new(storage.clone(), "test_orphan_recovery").unwrap();
        let seq_no = 67890;

        // Pin checkpoint and register orphan directly on store
        storage.pin_checkpoint(seq_no).await.unwrap();
        store.register_pinned_seq_no_orphan(PinnedSeqNoOrphan {
            seq_no,
            timestamp_ms: monotonic_timestamp_ms(),
        });

        assert!(storage.pinned.lock().contains(&seq_no));
        assert!(!store.orphan_registry().get_orphan_pins().is_empty());

        // Recover orphaned pins via store
        let recovered = store.recover_orphaned_pins().await.unwrap();

        assert_eq!(recovered, vec![seq_no]);
        assert!(
            !storage.pinned.lock().contains(&seq_no),
            "Storage sequence number 67890 must be unpinned after recovery"
        );
        assert!(
            store.orphan_registry().get_orphan_pins().is_empty(),
            "Orphan registry must be empty after recovery"
        );
    }

    #[tokio::test]
    async fn test_multi_instance_orphan_isolation() {
        let storage1 = Arc::new(MockStorage::new());
        let store_a = PersistentCheckpointStore::new(storage1, "ns_inst_a").unwrap();

        let storage2 = Arc::new(MockStorage::new());
        let store_b = PersistentCheckpointStore::new(storage2, "ns_inst_b").unwrap();

        // Drop an uncommitted guard in Store A
        {
            let _guard_a = store_a.create_guard(TxId::new(5555)).unwrap();
            // _guard_a drops here without commit or rollback
        }

        // Verify Store A captured orphan checkpoint
        let orphans_a = store_a.get_orphaned_checkpoints();
        assert_eq!(orphans_a.len(), 1);
        assert_eq!(orphans_a[0].tx_id, TxId::new(5555));

        // Verify Store B remained untouched (zero orphans)
        let orphans_b = store_b.get_orphaned_checkpoints();
        assert!(
            orphans_b.is_empty(),
            "Instance B orphan state must not be polluted by Instance A drops"
        );
    }

    #[tokio::test]
    async fn test_pin_guard_unpins_checkpoint_on_storage_write_failure() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage.clone(), "test_pinguard").unwrap();

        let seq_no = 999;
        let cp_key = b"test_pinguard:checkpoint:fail_write_cp";
        *storage.fail_on_put.lock() = Some(cp_key.to_vec());

        let res = store
            .create_checkpoint(
                "fail_write_cp",
                "col1",
                seq_no,
                TxId::new(10),
                serde_json::json!({}),
            )
            .await;

        assert!(
            res.is_err(),
            "Checkpoint creation must fail when storage put fails"
        );

        // Yield execution briefly to allow drop task on Handle::spawn to complete if async
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Verify PinGuard drop unpinned seq_no 999
        assert!(
            !storage.pinned.lock().contains(&seq_no),
            "Sequence number 999 must be unpinned after storage write failure via PinGuard RAII drop"
        );
    }

    #[tokio::test]
    async fn test_pin_before_unpin_invariant_on_failure() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();

        // 1. Create first checkpoint successfully
        store
            .create_checkpoint("my_cp", "c1", 1, TxId::new(1), serde_json::json!({}))
            .await
            .unwrap(); // unwrap

        assert!(storage.pinned.lock().contains(&1));

        // 2. Make next save fail
        let cp_key = b"test:checkpoint:my_cp";
        *storage.fail_on_put.lock() = Some(cp_key.to_vec());

        // 3. Try to overwrite with a new checkpoint, which will fail
        let res = store
            .create_checkpoint("my_cp", "c1", 2, TxId::new(2), serde_json::json!({}))
            .await;

        assert!(res.is_err());

        // 4. Verify invariant: old checkpoint (1) must still be pinned!
        assert!(
            storage.pinned.lock().contains(&1),
            "Old checkpoint should still be pinned because save failed"
        );

        // 5. Verify invariant: new checkpoint (2) should be unpinned (rolled back)!
        assert!(
            !storage.pinned.lock().contains(&2),
            "New checkpoint should be unpinned after failure"
        );
    }

    #[test]
    fn test_panic_unwind_triggers_orphan_registration_and_recovery() {
        let storage = Arc::new(MockStorage::new());

        let (store, panic_result) = std::thread::spawn({
            let storage = Arc::clone(&storage);
            move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to build Tokio runtime for panic test");

                let store =
                    Arc::new(PersistentCheckpointStore::new(storage, "test_panic").unwrap());
                let store_clone = Arc::clone(&store);

                let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    rt.block_on(async move {
                        let _guard = store_clone.create_guard(TxId::new(7070)).unwrap();
                        // Intentionally trigger panic inside guard scope
                        panic!("Simulated intentional panic between guard creation and commit");
                    });
                }));

                (store, panic_res)
            }
        })
        .join()
        .expect("Thread failed to join");

        assert!(panic_result.is_err(), "catch_unwind must capture the panic");

        // Verify orphaned checkpoint is registered in instance store after panic unwind
        assert_eq!(store.get_orphaned_checkpoints().len(), 1);

        // Perform recovery in a fresh runtime
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime for recovery test");

        rt.block_on(async move {
            let recovered = store.recover_orphaned_checkpoints().await.unwrap();
            assert_eq!(recovered, vec![TxId::new(7070)]);

            // Verify transaction was rolled back in storage
            let rolled_back = storage.rolled_back_tx.lock().clone();
            assert_eq!(rolled_back, vec![TxId::new(7070)]);
        });
    }

    #[tokio::test]
    async fn checkpoint_guard_rollback_on_drop() {
        let storage = Arc::new(MockStorage::new());
        let store =
            PersistentCheckpointStore::new(storage.clone(), "test_guard_rollback_on_drop").unwrap();
        store.clear_all_orphaned_checkpoints();

        {
            let _guard = store.create_guard(TxId::new(42)).unwrap(); // unwrap
                                                                     // guard drops here without commit
        }

        let recovered = store.recover_orphaned_checkpoints().await.unwrap();
        assert_eq!(recovered, vec![TxId::new(42)]);

        let rolled_back = storage.rolled_back_tx.lock().clone();
        assert_eq!(rolled_back, vec![TxId::new(42)]);
    }

    #[tokio::test]
    async fn checkpoint_guard_commit_prevents_rollback() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();

        let guard = store.create_guard(TxId::new(100)).unwrap(); // unwrap
        let cp = guard.commit().unwrap(); // unwrap
        assert_eq!(cp.tx_id, TxId::new(100));

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert!(
            storage.rolled_back_tx.lock().is_empty(),
            "Committed guard should not perform rollback"
        );
    }

    #[tokio::test]
    async fn list_checkpoints_empty_initially() {
        use memfuse_core::traits::CheckpointCoordinator;
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage, "test").unwrap();

        let list = store.list_named_checkpoints().await.unwrap(); // unwrap
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn checkpoint_not_found_returns_err() {
        use memfuse_core::traits::CheckpointCoordinator;
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage, "test").unwrap();

        let res = store.restore_named_checkpoint("nonexistent").await;
        assert!(matches!(res, Err(MemFuseError::CheckpointNotFound)));
    }

    #[tokio::test]
    async fn test_list_named_checkpoints_after_reopen() {
        use memfuse_core::traits::CheckpointCoordinator;
        let storage = Arc::new(MockStorage::new());
        {
            let store1 = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();
            store1
                .create_named_checkpoint(
                    "cp1",
                    "col1",
                    1,
                    TxId::new(TxId::INTERNAL_BASE + 1),
                    serde_json::json!({}),
                )
                .await
                .unwrap(); // unwrap
            store1
                .create_named_checkpoint(
                    "cp2",
                    "col1",
                    2,
                    TxId::new(TxId::INTERNAL_BASE + 2),
                    serde_json::json!({}),
                )
                .await
                .unwrap(); // unwrap
        }

        let store2 = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();
        let list = store2.list_named_checkpoints().await.unwrap(); // unwrap

        assert_eq!(list.len(), 2);
        let names: Vec<_> = list.into_iter().map(|m| m.name).collect();
        assert_eq!(names, vec!["cp1", "cp2"]);
    }

    #[tokio::test]
    async fn list_checkpoints_cache_matches_storage() {
        use memfuse_core::traits::CheckpointCoordinator;
        let storage = Arc::new(MockStorage::new());
        let store1 = Arc::new(PersistentCheckpointStore::new(storage.clone(), "test").unwrap());

        // Create 3 checkpoints
        for i in 1..=3 {
            store1
                .create_named_checkpoint(
                    &format!("cp-{i}"),
                    "col1",
                    i,
                    TxId::new(TxId::INTERNAL_BASE + i),
                    serde_json::json!({}),
                )
                .await
                .unwrap(); // unwrap
        }

        // Drop and reload the store from same storage
        let store2 = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();
        let list = store2.list_named_checkpoints().await.unwrap(); // unwrap

        assert_eq!(list.len(), 3);
        let names: Vec<_> = list.into_iter().map(|m| m.name).collect();
        assert_eq!(names, vec!["cp-1", "cp-2", "cp-3"]);
    }

    #[tokio::test]
    async fn concurrent_checkpoint_creation_is_safe() {
        use memfuse_core::traits::CheckpointCoordinator;
        use tokio::task::JoinSet;

        let storage = Arc::new(MockStorage::new());
        let store = Arc::new(PersistentCheckpointStore::new(storage, "test").unwrap());

        let mut tasks = JoinSet::new();
        for i in 0..8u64 {
            let store = Arc::clone(&store);
            tasks.spawn(async move {
                store
                    .create_named_checkpoint(
                        &format!("cp-{i}"),
                        "col1",
                        i,
                        TxId::new(TxId::INTERNAL_BASE + i),
                        serde_json::json!({}),
                    )
                    .await
            });
        }
        // All must succeed or fail without panicking
        while let Some(res) = tasks.join_next().await {
            let res = res.unwrap(); // unwrap
            if let Err(e) = res {
                println!("Checkpoint creation failed (acceptable): {e}");
            }
        }

        let all = store.list_named_checkpoints().await.unwrap(); // unwrap
        assert_eq!(all.len(), 8);
    }

    #[test]
    fn test_checkpoint_guard_dropped_outside_tokio_runtime() {
        std::thread::spawn(|| {
            let storage = Arc::new(MockStorage::new());
            let store = PersistentCheckpointStore::new(storage, "test").unwrap();

            let initial_skipped = store.skipped_rollback_count();

            {
                let _guard = store.create_guard(TxId::new(999)).unwrap(); // unwrap
                // _guard drops here at end of inner scope
            }

            assert_eq!(
                store.skipped_rollback_count(),
                initial_skipped + 1,
                "Skipped rollback counter must increment when guard is dropped outside Tokio runtime"
            );
        })
        .join()
        .expect("// expect Thread panic in test_checkpoint_guard_dropped_outside_tokio_runtime");
    }

    #[tokio::test]
    async fn test_auto_rollback_tracking_and_await() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage.clone(), "test_auto_rollback").unwrap();
        store.clear_all_orphaned_checkpoints();

        {
            let _guard = store.create_guard(TxId::new(808)).unwrap(); // unwrap
                                                                      // Drop without commit inside tokio runtime
        }

        let recovered = store.recover_orphaned_checkpoints().await.unwrap();
        assert_eq!(recovered, vec![TxId::new(808)]);

        let rolled_back = storage.rolled_back_tx.lock().clone();
        assert_eq!(rolled_back, vec![TxId::new(808)]);
        assert_eq!(pending_rollback_count(), 0);
    }

    #[tokio::test]
    async fn test_drop_checkpoint_uses_unique_tx_and_unpins() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();

        store
            .create_checkpoint("drop_me", "col1", 42, TxId::new(1), serde_json::json!({}))
            .await
            .unwrap(); // unwrap

        assert!(storage.pinned.lock().contains(&42));
        assert!(store.get_checkpoint("drop_me").await.unwrap().is_some()); // unwrap

        store.drop_checkpoint("drop_me").await.unwrap(); // unwrap

        assert!(
            !storage.pinned.lock().contains(&42),
            "Checkpoint seq_no 42 should be unpinned after drop"
        );
        assert!(store.get_checkpoint("drop_me").await.unwrap().is_none()); // unwrap
    }

    #[tokio::test]
    async fn test_next_tx_overflow_returns_err() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage, "test").unwrap();

        store.tx_counter.store(1_000_000, Ordering::SeqCst);

        let res = store.allocate_tx().await;
        assert!(res.is_err());
        if let Err(MemFuseError::Internal(msg)) = res {
            assert!(msg.contains("overflow"));
        } else {
            panic!("Expected Internal error on overflow");
        }
    }

    #[tokio::test]
    async fn test_input_validation_empty_and_oversized_names() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage, "test").unwrap();

        // Empty name
        let res = store
            .create_checkpoint("", "col1", 1, TxId::new(1), serde_json::json!({}))
            .await;
        assert!(matches!(res, Err(MemFuseError::InvalidInput(_))));

        // Whitespace name
        let res = store
            .create_checkpoint("   ", "col1", 1, TxId::new(1), serde_json::json!({}))
            .await;
        assert!(matches!(res, Err(MemFuseError::InvalidInput(_))));

        // Empty collection ID
        let res = store
            .create_checkpoint("cp1", "", 1, TxId::new(1), serde_json::json!({}))
            .await;
        assert!(matches!(res, Err(MemFuseError::InvalidInput(_))));

        // Oversized name (> 256 chars)
        let long_name = "a".repeat(257);
        let res = store
            .create_checkpoint(&long_name, "col1", 1, TxId::new(1), serde_json::json!({}))
            .await;
        assert!(matches!(res, Err(MemFuseError::InvalidInput(_))));

        // Multibyte unicode name: 256 chars (512+ bytes) accepted, 257 chars rejected
        let unicode_256 = "ä".repeat(256);
        let res_256 = store
            .create_checkpoint(&unicode_256, "col1", 1, TxId::new(1), serde_json::json!({}))
            .await;
        assert!(res_256.is_ok());

        let unicode_257 = "ä".repeat(257);
        let res_257 = store
            .create_checkpoint(&unicode_257, "col1", 1, TxId::new(1), serde_json::json!({}))
            .await;
        assert!(matches!(res_257, Err(MemFuseError::InvalidInput(_))));

        // Drop with empty name
        let res = store.drop_checkpoint("").await;
        assert!(matches!(res, Err(MemFuseError::InvalidInput(_))));

        // Get with empty name
        let res = store.get_checkpoint("   ").await;
        assert!(matches!(res, Err(MemFuseError::InvalidInput(_))));
    }

    #[allow(non_snake_case)]
    #[tokio::test]
    async fn create_checkpoint_CASE_unicode_and_multibyte_name() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();

        let unicode_name = "Prüfpunkt_1_🚀_日本語";
        let collection_id = "Sammlung_äöü_123";

        store
            .create_checkpoint(
                unicode_name,
                collection_id,
                10,
                TxId::new(101),
                serde_json::json!({"tag": "überprüfen"}),
            )
            .await
            .expect("// expect #[cfg(test)]");

        let fetched = store
            .get_checkpoint(unicode_name)
            .await
            .expect("// expect #[cfg(test)]")
            .expect("// expect #[cfg(test)]");

        assert_eq!(fetched.name, "Prüfpunkt_1_🚀_日本語");
        assert_eq!(fetched.collection_id, "Sammlung_äöü_123");
        assert_eq!(fetched.seq_no, 10);
        assert_eq!(fetched.tx_id, TxId::new(101));
        assert_eq!(fetched.metadata["tag"], "überprüfen");
    }

    #[allow(non_snake_case)]
    #[tokio::test]
    async fn create_checkpoint_CASE_exact_max_len_256() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();

        let max_name = "a".repeat(256);
        let res = store
            .create_checkpoint(&max_name, "col1", 1, TxId::new(1), serde_json::json!({}))
            .await;
        assert!(res.is_ok(), "256 characters name must be allowed");

        let fetched = store
            .get_checkpoint(&max_name)
            .await
            .expect("// expect #[cfg(test)]");
        assert!(fetched.is_some());
    }

    #[allow(non_snake_case)]
    #[tokio::test]
    async fn drop_checkpoint_CASE_nonexistent_returns_ok() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();

        let res = store.drop_checkpoint("nonexistent_checkpoint").await;
        assert!(
            res.is_ok(),
            "Dropping a non-existent checkpoint should be idempotent and return Ok(())"
        );
    }

    #[allow(non_snake_case)]
    #[tokio::test]
    async fn restore_checkpoint_CASE_not_found_returns_err() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage, "test").unwrap();

        let res = store.restore_checkpoint("missing_cp").await;
        assert!(matches!(res, Err(MemFuseError::CheckpointNotFound)));
    }

    #[allow(non_snake_case)]
    #[tokio::test]
    async fn list_checkpoints_CASE_corrupted_storage_data_propagates_err() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();

        // Put invalid JSON payload into storage under checkpoint namespace format (namespace:checkpoint:name)
        let corrupt_key = b"test:checkpoint:corrupt_cp";
        storage
            .put(TxId::new(1), corrupt_key, b"invalid json bytes{{{")
            .await
            .expect("// expect #[cfg(test)]");

        let res = store.list_checkpoints().await;
        assert!(matches!(res, Err(MemFuseError::Serialization(_))));
    }

    #[tokio::test]
    async fn test_get_orphaned_checkpoints_for_namespace() {
        let storage = Arc::new(MockStorage::new());
        let store_a = PersistentCheckpointStore::new(storage.clone(), "ns_a").unwrap();
        let store_b = PersistentCheckpointStore::new(storage.clone(), "ns_b").unwrap();

        {
            let _guard_a1 = store_a.create_guard(TxId::new(1001)).unwrap();
            let _guard_a2 = store_a.create_guard(TxId::new(1002)).unwrap();
            let _guard_b1 = store_b.create_guard(TxId::new(2001)).unwrap();
            // All 3 guards drop here uncommitted
        }

        let orphans_a = store_a.get_orphaned_checkpoints();
        let orphans_b = store_b.get_orphaned_checkpoints();

        assert_eq!(orphans_a.len(), 2);
        assert!(orphans_a
            .iter()
            .all(|cp| cp.namespace.as_deref() == Some("ns_a")));
        let tx_a: Vec<TxId> = orphans_a.iter().map(|cp| cp.tx_id).collect();
        assert!(tx_a.contains(&TxId::new(1001)));
        assert!(tx_a.contains(&TxId::new(1002)));

        assert_eq!(orphans_b.len(), 1);
        assert_eq!(orphans_b[0].tx_id, TxId::new(2001));
        assert_eq!(orphans_b[0].namespace.as_deref(), Some("ns_b"));
    }

    #[allow(non_snake_case)]
    #[tokio::test]
    async fn allocate_tx_CASE_parity_with_deprecated_next_tx() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage, "test").unwrap();

        let tx1 = store.allocate_tx().await.expect("// expect #[cfg(test)]");
        #[allow(deprecated)]
        let tx2 = store.next_tx().await.expect("// expect #[cfg(test)]");
        let tx3 = store.allocate_tx().await.expect("// expect #[cfg(test)]");

        assert_eq!(tx1, TxId::new(TxId::INTERNAL_BASE));
        assert_eq!(tx2, TxId::new(TxId::INTERNAL_BASE + 1));
        assert_eq!(tx3, TxId::new(TxId::INTERNAL_BASE + 2));
    }

    #[tokio::test]
    async fn test_store_skipped_rollback_count_and_checkpoint_counter() {
        let storage = Arc::new(MockStorage::new());
        let store = PersistentCheckpointStore::new(storage, "test_skipped").unwrap();

        assert_eq!(store.skipped_rollback_count(), 0);
        assert_eq!(store.checkpoint_guard_skipped_rollback_count(), 0);

        let ts = store.monotonic_timestamp_ms();
        assert!(ts > 0);

        {
            let _guard = store.create_guard(TxId::new(300)).unwrap();
            // Drop without commit or rollback
        }

        assert_eq!(store.skipped_rollback_count(), 1);
        assert_eq!(store.checkpoint_guard_skipped_rollback_count(), 1);
    }

    #[tokio::test]
    async fn test_concurrent_checkpoint_index_atomicity() {
        use std::sync::atomic::AtomicBool;

        let storage = Arc::new(MockStorage::new());
        let store = Arc::new(PersistentCheckpointStore::new(storage, "test_atomicity").unwrap());
        let stop_flag = Arc::new(AtomicBool::new(false));

        let cp_name = "atomic_cp";

        // Spawn writer task continuously creating and dropping checkpoints under the same name
        let writer_store = Arc::clone(&store);
        let writer_stop = Arc::clone(&stop_flag);
        let writer_handle = tokio::spawn(async move {
            let mut seq = 1u64;
            while !writer_stop.load(Ordering::Relaxed) {
                let _create_res = writer_store
                    .create_checkpoint(
                        cp_name,
                        "col_atomic",
                        seq,
                        TxId::new(seq),
                        serde_json::json!({"seq": seq}),
                    )
                    .await;
                tokio::task::yield_now().await;
                let _drop_res = writer_store.drop_checkpoint(cp_name).await;
                tokio::task::yield_now().await;
                seq += 1;
            }
        });

        // Spawn 4 reader tasks continuously reading the checkpoint by name
        let mut reader_handles = Vec::new();
        for _ in 0..4 {
            let reader_store = Arc::clone(&store);
            let reader_stop = Arc::clone(&stop_flag);
            reader_handles.push(tokio::spawn(async move {
                while !reader_stop.load(Ordering::Relaxed) {
                    if let Ok(Some(cp)) = reader_store.get_checkpoint(cp_name).await {
                        // Verify that if a checkpoint is returned, its internal name match and seq_no are consistent
                        assert_eq!(cp.name, cp_name);
                        assert!(cp.seq_no > 0);
                        let meta_seq = cp.metadata.get("seq").and_then(|v| v.as_u64());
                        assert_eq!(meta_seq, Some(cp.seq_no));
                    }
                    tokio::task::yield_now().await;
                }
            }));
        }

        // Run concurrent readers & writers for 200 ms
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        stop_flag.store(true, Ordering::Relaxed);

        let _ = writer_handle.await;
        for handle in reader_handles {
            let _ = handle.await;
        }
    }
}
