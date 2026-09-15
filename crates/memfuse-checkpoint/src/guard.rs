use crate::meta::StateCheckpoint;
use crate::orphan::{InstanceOrphanRegistry, PinnedSeqNoOrphan};
use memfuse_core::{MemFuseError, Result, TxId};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// RAII-Guard für gepinnte Checkpoint-Sequenznummern (gemäß ADR-015).
/// Garantiert, dass eine gepinnte Sequenznummer bei einem Fehler oder Panic während des Schreibens
/// automatisch entpinnt wird, um dauerhafte GC-Blockaden zu verhindern.
#[must_use = "PinGuard must be defused upon successful checkpoint storage"]
pub struct PinGuard<S: memfuse_core::StorageEngine> {
    storage: Arc<S>,
    seq_no: Option<u64>,
    orphan_registry: Arc<InstanceOrphanRegistry>,
}

impl<S: memfuse_core::StorageEngine> PinGuard<S> {
    pub async fn pin(
        storage: Arc<S>,
        seq_no: u64,
        orphan_registry: Arc<InstanceOrphanRegistry>,
    ) -> Result<Self> {
        storage.pin_checkpoint(seq_no).await?;
        Ok(Self {
            storage,
            seq_no: Some(seq_no),
            orphan_registry,
        })
    }

    /// Entschärft den Guard nach erfolgreicher Persistierung, sodass die Sequenznummer gepinnt bleibt.
    pub fn defuse(mut self) {
        self.seq_no.take();
    }

    /// Entpinnt die Sequenznummer explizit und asynchron bei einem abgefangenen Fehler.
    pub async fn unpin(mut self) -> Result<()> {
        if let Some(seq_no) = self.seq_no.take() {
            self.storage.unpin_checkpoint(seq_no).await?;
        }
        if let Err(err) = self.orphan_registry.flush_orphan_registry().await {
            tracing::warn!(?err, "Failed piggyback flush on PinGuard unpin");
        }
        Ok(())
    }
}

impl<S: memfuse_core::StorageEngine> Drop for PinGuard<S> {
    fn drop(&mut self) {
        if let Some(seq_no) = self.seq_no.take() {
            let wall_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let orphan = PinnedSeqNoOrphan {
                seq_no,
                timestamp_ms: wall_ms,
            };
            self.orphan_registry.register_orphan_sync(orphan);
            tracing::warn!(
                seq = seq_no,
                "PinGuard dropped without explicit unpin — registered as orphan"
            );
        }
    }
}

/// RAII Guard, der explizit über [`commit`](Self::commit) oder [`rollback`](Self::rollback) finalisiert werden MUSS.
///
/// # RAII-Kontrakt und Betriebshinweise
/// Ein `CheckpointGuard` muss explizit über `commit()` oder `rollback().await` finalisiert werden.
/// Wird der Guard ohne expliziten Commit/Rollback gedroppt (z. B. bei Panik oder Programmierfehler),
/// wird KEIN asynchroner Hintergrund-Rollback gespawnt, um Kollisionen mit späteren Transaktionen zu verhindern.
/// Stattdessen wird der Checkpoint als "orphaned" registriert/persistiert und beim nächsten kontrollierten Recovery-Zyklus verarbeitet.
#[must_use = "CheckpointGuard must be explicitly finalized via .commit() or .rollback().await"]
pub struct CheckpointGuard<S: memfuse_core::StorageEngine> {
    pub(crate) checkpoint: Option<StateCheckpoint>,
    pub(crate) storage: Arc<S>,
    pub(crate) namespace: String,
    pub(crate) orphan_registry: Arc<InstanceOrphanRegistry>,
    pub(crate) skipped_rollbacks: Arc<AtomicU64>,
}

impl<S: memfuse_core::StorageEngine> CheckpointGuard<S> {
    pub fn new(checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>) -> Self {
        let ns = namespace.into();
        let orphan_path = std::path::PathBuf::from(format!("{ns}_orphaned_checkpoints.json"));
        let registry = Arc::new(InstanceOrphanRegistry::new(orphan_path));
        let skipped_rollbacks = Arc::new(AtomicU64::new(0));
        Self::with_registry_and_counter(checkpoint, storage, ns, registry, skipped_rollbacks)
    }

    pub fn with_registry(
        checkpoint: StateCheckpoint,
        storage: Arc<S>,
        namespace: impl Into<String>,
        orphan_registry: Arc<InstanceOrphanRegistry>,
    ) -> Self {
        let skipped_rollbacks = Arc::new(AtomicU64::new(0));
        Self::with_registry_and_counter(
            checkpoint,
            storage,
            namespace,
            orphan_registry,
            skipped_rollbacks,
        )
    }

    pub fn with_registry_and_counter(
        checkpoint: StateCheckpoint,
        storage: Arc<S>,
        namespace: impl Into<String>,
        orphan_registry: Arc<InstanceOrphanRegistry>,
        skipped_rollbacks: Arc<AtomicU64>,
    ) -> Self {
        Self {
            checkpoint: Some(checkpoint),
            storage,
            namespace: namespace.into(),
            orphan_registry,
            skipped_rollbacks,
        }
    }

    /// Erstellt einen neuen CheckpointGuard für einen Agenten-Schritt.
    pub async fn for_agent_step(storage: Arc<S>, tx: TxId) -> Result<Self> {
        let wall_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let cp = StateCheckpoint {
            tx_id: tx,
            timestamp_ms: wall_ms,
            namespace: Some("agent_step".to_string()),
        };
        Ok(Self::new(cp, storage, "agent_step"))
    }

    pub async fn for_agent_step_with_registry(
        storage: Arc<S>,
        tx: TxId,
        orphan_registry: Arc<InstanceOrphanRegistry>,
    ) -> Result<Self> {
        let wall_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let cp = StateCheckpoint {
            tx_id: tx,
            timestamp_ms: wall_ms,
            namespace: Some("agent_step".to_string()),
        };
        Ok(Self::with_registry(
            cp,
            storage,
            "agent_step",
            orphan_registry,
        ))
    }

    pub fn checkpoint(&self) -> Result<&StateCheckpoint> {
        self.checkpoint
            .as_ref()
            .ok_or_else(|| MemFuseError::Internal("Checkpoint already consumed".into()))
    }

    pub fn commit(mut self) -> Result<StateCheckpoint> {
        let cp = self
            .checkpoint
            .take()
            .ok_or_else(|| MemFuseError::Internal("Checkpoint already consumed".into()))?;
        let reg = Arc::clone(&self.orphan_registry);
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                if let Err(err) = reg.flush_orphan_registry().await {
                    tracing::warn!(?err, "Failed piggyback flush on CheckpointGuard commit");
                }
            });
        }
        Ok(cp)
    }

    /// Führt ein manuelles, asynchrones Rollback des Checkpoints aus.
    /// Nach Aufruf von `rollback()` ist der Guard konsumiert, sodass beim Drop kein erneutes Rollback ausgelöst wird.
    ///
    /// # Serialisierungsbarriere
    /// Wenn neuere committete Transaktionen mit `last_tx > target_tx` existieren, schlägt das Rollback mit
    /// einem Fehler fehl, um Datenverlust neuerer Transaktionen zu verhindern.
    pub async fn rollback(mut self) -> Result<()> {
        if let Some(cp) = self.checkpoint.take() {
            let last_tx = self.storage.last_tx_id().await?;
            if last_tx > cp.tx_id {
                return Err(MemFuseError::Transaction(format!(
                    "Serialization barrier violation: Cannot rollback to TxId {} because newer committed transaction TxId {} exists in storage",
                    cp.tx_id.inner(),
                    last_tx.inner()
                )));
            }
            let res = self.storage.rollback_to_tx(cp.tx_id).await;
            if let Err(err) = self.orphan_registry.flush_orphan_registry().await {
                tracing::warn!(?err, "Failed piggyback flush on CheckpointGuard rollback");
            }
            res
        } else {
            Err(MemFuseError::Internal("Checkpoint already consumed".into()))
        }
    }

    /// Führt ein synchrones, blockierendes Rollback des Checkpoints aus.
    ///
    /// # Einschränkung
    /// Diese Methode darf **nicht** aus einem laufenden asynchronen Tokio-Kontext heraus aufgerufen werden.
    /// In asynchronem Kontext führt ein Aufruf von `rollback_blocking` zu einem Fehler
    /// [`MemFuseError::Internal`], um Deadlocks und Tokio-Panics zu verhindern. Verwende in async-Kontexten
    /// stattdessen [`rollback`](Self::rollback).
    pub fn rollback_blocking(mut self) -> Result<()> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(MemFuseError::Internal(
                "Cannot call rollback_blocking from within an active async Tokio runtime context; use rollback().await instead"
                    .to_string(),
            ));
        }

        let cp = self
            .checkpoint
            .take()
            .ok_or_else(|| MemFuseError::Internal("Checkpoint already consumed".into()))?;

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                MemFuseError::Internal(format!(
                    "Failed to create Tokio runtime for rollback_blocking: {e}"
                ))
            })?;

        rt.block_on(async {
            let last_tx = self.storage.last_tx_id().await?;
            if last_tx > cp.tx_id {
                return Err(MemFuseError::Transaction(format!(
                    "Serialization barrier violation: Cannot rollback to TxId {} because newer committed transaction TxId {} exists in storage",
                    cp.tx_id.inner(),
                    last_tx.inner()
                )));
            }
            self.storage.rollback_to_tx(cp.tx_id).await
        })
    }
}

impl<S: memfuse_core::StorageEngine> Drop for CheckpointGuard<S> {
    fn drop(&mut self) {
        if let Some(mut cp) = self.checkpoint.take() {
            cp.namespace = Some(self.namespace.clone());
            self.skipped_rollbacks.fetch_add(1, Ordering::SeqCst);
            tracing::error!(
                tx_id = ?cp.tx_id,
                "CheckpointGuard dropped without explicit commit or rollback — \
                 checkpoint registered in instance-scoped orphan registry for controlled recovery (ADR-053)."
            );
            self.orphan_registry.register_checkpoint_sync(cp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memfuse_core::{BoxFuture, StorageEngine, StorageStats};
    use parking_lot::Mutex;
    use std::collections::{HashMap, HashSet};

    struct MockStorage {
        data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
        pinned: Mutex<HashSet<u64>>,
        rolled_back_tx: Mutex<Vec<TxId>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                data: Mutex::new(HashMap::new()),
                pinned: Mutex::new(HashSet::new()),
                rolled_back_tx: Mutex::new(Vec::new()),
            }
        }
    }

    impl StorageEngine for MockStorage {
        fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
            Box::pin(async move { Ok(self.data.lock().get(key).cloned()) })
        }
        fn get_at_seq<'a>(
            &'a self,
            key: &'a [u8],
            _seq: u64,
        ) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
            Box::pin(async move { self.get(key).await })
        }
        fn put<'a>(
            &'a self,
            _tx_id: TxId,
            key: &'a [u8],
            value: &'a [u8],
        ) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
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

    #[test]
    fn test_orphan_registry_persists_across_drop() {
        let registry = Arc::new(InstanceOrphanRegistry::new(""));
        let storage = Arc::new(MockStorage::new());
        let seq_no = 12345;

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime for test");

        let guard = rt.block_on(async {
            PinGuard::pin(storage.clone(), seq_no, registry.clone())
                .await
                .unwrap()
        });

        // Drop guard without runtime or explicit unpin/defuse
        drop(guard);

        // Verify orphan ID appears in registry
        let orphans = registry.get_orphan_pins();
        assert_eq!(
            orphans.len(),
            1,
            "Orphan sequence number 12345 must appear in registry upon PinGuard drop"
        );
        assert_eq!(orphans[0].seq_no, seq_no);
    }

    #[test]
    fn test_rollback_blocking_in_sync_context() {
        let storage = Arc::new(MockStorage::new());
        let cp = StateCheckpoint {
            tx_id: TxId::new(909),
            timestamp_ms: 1000,
            namespace: Some("test".to_string()),
        };
        let guard = CheckpointGuard::new(cp, storage.clone(), "test");
        let res = guard.rollback_blocking();
        assert!(
            res.is_ok(),
            "rollback_blocking must succeed in sync context"
        );

        let rolled_back = storage.rolled_back_tx.lock().clone();
        assert_eq!(rolled_back, vec![TxId::new(909)]);
    }

    #[tokio::test]
    async fn test_rollback_blocking_in_async_context_returns_error() {
        let storage = Arc::new(MockStorage::new());
        let cp = StateCheckpoint {
            tx_id: TxId::new(1010),
            timestamp_ms: 1000,
            namespace: Some("test".to_string()),
        };
        let guard = CheckpointGuard::new(cp, storage, "test");

        // Timeout safety net to guarantee no hanging/deadlock
        let res = tokio::time::timeout(std::time::Duration::from_secs(2), async move {
            guard.rollback_blocking()
        })
        .await
        .expect("rollback_blocking in async context timed out - possible deadlock!");

        assert!(
            res.is_err(),
            "rollback_blocking called from async context must return error immediately to prevent deadlock"
        );
        if let Err(MemFuseError::Internal(msg)) = res {
            assert!(msg.contains("active async Tokio runtime context"));
            assert!(msg.contains("rollback().await"));
        } else {
            panic!(
                "Expected MemFuseError::Internal error message instructing to use rollback().await"
            );
        }
    }

    #[tokio::test]
    async fn test_checkpoint_guard_for_agent_step() {
        let storage = Arc::new(MockStorage::new());
        let guard = CheckpointGuard::for_agent_step(storage.clone(), TxId::new(55))
            .await
            .unwrap();

        let cp = guard.checkpoint().unwrap();
        assert_eq!(cp.tx_id, TxId::new(55));
        assert!(cp.timestamp_ms > 0);

        let committed = guard.commit().unwrap();
        assert_eq!(committed.tx_id, TxId::new(55));
    }

    #[allow(non_snake_case)]
    #[tokio::test]
    async fn checkpoint_guard_CASE_uncommitted_guard_holds_state() {
        let storage = Arc::new(MockStorage::new());
        let cp = StateCheckpoint {
            tx_id: TxId::new(500),
            timestamp_ms: 1000,
            namespace: Some("test".to_string()),
        };
        let guard = CheckpointGuard::new(cp, storage, "test");
        let cp_ref = guard.checkpoint().expect("// expect #[cfg(test)]").clone();
        assert_eq!(cp_ref.tx_id, TxId::new(500));

        // Commit takes ownership of self and consumes the state checkpoint
        let committed_cp = guard.commit().expect("// expect #[cfg(test)]");
        assert_eq!(committed_cp.tx_id, TxId::new(500));
    }

    #[allow(non_snake_case)]
    #[tokio::test]
    async fn checkpoint_guard_CASE_commit_moves_ownership() {
        let storage = Arc::new(MockStorage::new());
        let guard = CheckpointGuard::new(
            StateCheckpoint {
                tx_id: TxId::new(777),
                timestamp_ms: 12345,
                namespace: Some("test".to_string()),
            },
            storage,
            "test",
        );

        assert!(guard.checkpoint().is_ok());

        let cp = guard.commit().expect("// expect #[cfg(test)]");
        assert_eq!(cp.tx_id, TxId::new(777));
        assert_eq!(cp.timestamp_ms, 12345);
    }

    #[allow(non_snake_case)]
    #[tokio::test]
    async fn checkpoint_guard_CASE_rollback_consumed_returns_err() {
        let dummy_storage = Arc::new(MockStorage::new());
        let consumed_guard = CheckpointGuard::<MockStorage> {
            checkpoint: None,
            storage: dummy_storage,
            namespace: "test".to_string(),
            orphan_registry: Arc::new(InstanceOrphanRegistry::new("")),
            skipped_rollbacks: Arc::new(AtomicU64::new(0)),
        };
        assert!(matches!(
            consumed_guard.checkpoint(),
            Err(MemFuseError::Internal(_))
        ));

        let consumed_guard2 = CheckpointGuard::<MockStorage> {
            checkpoint: None,
            storage: Arc::new(MockStorage::new()),
            namespace: "test".to_string(),
            orphan_registry: Arc::new(InstanceOrphanRegistry::new("")),
            skipped_rollbacks: Arc::new(AtomicU64::new(0)),
        };
        assert!(matches!(
            consumed_guard2.commit(),
            Err(MemFuseError::Internal(_))
        ));

        let consumed_guard3 = CheckpointGuard::<MockStorage> {
            checkpoint: None,
            storage: Arc::new(MockStorage::new()),
            namespace: "test".to_string(),
            orphan_registry: Arc::new(InstanceOrphanRegistry::new("")),
            skipped_rollbacks: Arc::new(AtomicU64::new(0)),
        };
        let res = consumed_guard3.rollback().await;
        assert!(matches!(res, Err(MemFuseError::Internal(_))));
    }
}
