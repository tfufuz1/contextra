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
mod tests;
