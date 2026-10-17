use crate::meta::StateCheckpoint;
use crate::orphan::{clock_timestamp_ms, InstanceOrphanRegistry, PinnedSeqNoOrphan};
use contextra_ports::SystemClock;
use contextra_types::{ContextraError, Result, TxId};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// RAII-Guard für gepinnte Checkpoint-Sequenznummern (gemäß ADR-015).
/// Garantiert, dass eine gepinnte Sequenznummer bei einem Fehler oder Panic während des Schreibens
/// automatisch entpinnt wird, um dauerhafte GC-Blockaden zu verhindern.
#[must_use = "PinGuard must be defused upon successful checkpoint storage"]
pub struct PinGuard<S: contextra_ports::StorageEngine> {
    storage: Arc<S>,
    seq_no: Option<u64>,
    orphan_registry: Arc<InstanceOrphanRegistry>,
    clock: Arc<dyn contextra_ports::Clock>,
}

impl<S: contextra_ports::StorageEngine> PinGuard<S> {
    pub async fn pin(
        storage: Arc<S>,
        seq_no: u64,
        orphan_registry: Arc<InstanceOrphanRegistry>,
    ) -> Result<Self> {
        let clock = orphan_registry.clock().clone();
        Self::pin_with_clock(storage, seq_no, orphan_registry, clock).await
    }

    pub async fn pin_with_clock(
        storage: Arc<S>,
        seq_no: u64,
        orphan_registry: Arc<InstanceOrphanRegistry>,
        clock: Arc<dyn contextra_ports::Clock>,
    ) -> Result<Self> {
        storage.pin_checkpoint(seq_no).await?;
        Ok(Self {
            storage,
            seq_no: Some(seq_no),
            orphan_registry,
            clock,
        })
    }

    pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock> {
        &self.clock
    }

    /// Entschärft den Guard nach erfolgreicher Persistierung, sodass die Sequenznummer gepinnt bleibt.
    pub fn defuse(mut self) {
        self.seq_no.take();
    }

    /// Entpinnt die Sequenznummer explizit und asynchron bei einem abgefangenen Fehler.
    ///
    /// # Fehlerpropazierung & Pin-Verlustschutz
    /// Wenn `unpin_checkpoint` fehlschlägt oder das anschließende `flush_orphan_registry()`
    /// einen E/A-Fehler zurückgibt, wird die Sequenznummer synchron in die `InstanceOrphanRegistry`
    /// eingetragen (sofern noch nicht entpinnt), damit beim Recovery-Zyklus kein Pin verloren geht,
    /// und der Fehler wird als `Result::Err` propagiert.
    pub async fn unpin(mut self) -> Result<()> {
        let mut unpin_err = None;
        if let Some(seq_no) = self.seq_no.take() {
            if let Err(err) = self.storage.unpin_checkpoint(seq_no).await {
                // Bei Unpin-Fehler: Registriere als verwaist, um GC-Lecks zu verhindern
                let timestamp_ms = clock_timestamp_ms(self.clock.as_ref());
                self.orphan_registry
                    .register_orphan_sync(PinnedSeqNoOrphan {
                        seq_no,
                        timestamp_ms,
                    });
                unpin_err = Some(err);
            }
        }

        if let Err(flush_err) = self.orphan_registry.flush_orphan_registry().await {
            let context_err = ContextraError::Internal(format!(
                "Failed to flush orphan registry during PinGuard::unpin: {flush_err}"
            ));
            return Err(unpin_err.unwrap_or(context_err));
        }

        if let Some(err) = unpin_err {
            return Err(err);
        }

        Ok(())
    }
}

impl<S: contextra_ports::StorageEngine> Drop for PinGuard<S> {
    fn drop(&mut self) {
        if let Some(seq_no) = self.seq_no.take() {
            let timestamp_ms = clock_timestamp_ms(self.clock.as_ref());
            let orphan = PinnedSeqNoOrphan {
                seq_no,
                timestamp_ms,
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
pub struct CheckpointGuard<S: contextra_ports::StorageEngine> {
    pub(crate) checkpoint: Option<StateCheckpoint>,
    pub(crate) storage: Arc<S>,
    pub(crate) namespace: String,
    pub(crate) orphan_registry: Arc<InstanceOrphanRegistry>,
    pub(crate) skipped_rollbacks: Arc<AtomicU64>,
    pub(crate) clock: Arc<dyn contextra_ports::Clock>,
    pub(crate) metrics_sink: Option<Arc<dyn contextra_ports::MetricsSink>>,
    pub(crate) created_at: std::time::Instant,
}

impl<S: contextra_ports::StorageEngine> CheckpointGuard<S> {
    /// Erstellt einen neuen CheckpointGuard mit einer expliziten Clock (P28 / INV-CHECKPOINT-DETERMINISM-1).
    pub fn new_with_clock(
        checkpoint: StateCheckpoint,
        storage: Arc<S>,
        namespace: impl Into<String>,
        clock: Arc<dyn contextra_ports::Clock>,
    ) -> Self {
        Self::with_clock(checkpoint, storage, namespace, clock)
    }

    #[deprecated(
        since = "0.1.0",
        note = "Use new_with_clock to inject a Clock port explicitly (P28 / INV-CHECKPOINT-DETERMINISM-1)"
    )]
    pub fn new(checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>) -> Self {
        let clock: Arc<dyn contextra_ports::Clock> = Arc::new(SystemClock::new());
        Self::with_clock(checkpoint, storage, namespace, clock)
    }

    pub fn with_clock(
        checkpoint: StateCheckpoint,
        storage: Arc<S>,
        namespace: impl Into<String>,
        clock: Arc<dyn contextra_ports::Clock>,
    ) -> Self {
        let ns = namespace.into();
        let orphan_path = std::path::PathBuf::from(format!("{ns}_orphaned_checkpoints.json"));
        let registry = Arc::new(InstanceOrphanRegistry::new(orphan_path).with_clock(clock.clone()));
        let skipped_rollbacks = Arc::new(AtomicU64::new(0));
        Self::with_registry_counter_and_clock(
            checkpoint,
            storage,
            ns,
            registry,
            skipped_rollbacks,
            clock,
        )
    }

    pub fn with_registry(
        checkpoint: StateCheckpoint,
        storage: Arc<S>,
        namespace: impl Into<String>,
        orphan_registry: Arc<InstanceOrphanRegistry>,
    ) -> Self {
        let clock = orphan_registry.clock().clone();
        let skipped_rollbacks = Arc::new(AtomicU64::new(0));
        Self::with_registry_counter_and_clock(
            checkpoint,
            storage,
            namespace,
            orphan_registry,
            skipped_rollbacks,
            clock,
        )
    }

    pub fn with_registry_and_counter(
        checkpoint: StateCheckpoint,
        storage: Arc<S>,
        namespace: impl Into<String>,
        orphan_registry: Arc<InstanceOrphanRegistry>,
        skipped_rollbacks: Arc<AtomicU64>,
    ) -> Self {
        let clock = orphan_registry.clock().clone();
        Self::with_registry_counter_and_clock(
            checkpoint,
            storage,
            namespace,
            orphan_registry,
            skipped_rollbacks,
            clock,
        )
    }

    pub fn with_registry_counter_and_clock(
        checkpoint: StateCheckpoint,
        storage: Arc<S>,
        namespace: impl Into<String>,
        orphan_registry: Arc<InstanceOrphanRegistry>,
        skipped_rollbacks: Arc<AtomicU64>,
        clock: Arc<dyn contextra_ports::Clock>,
    ) -> Self {
        Self {
            checkpoint: Some(checkpoint),
            storage,
            namespace: namespace.into(),
            orphan_registry,
            skipped_rollbacks,
            clock,
            metrics_sink: None,
            created_at: std::time::Instant::now(),
        }
    }

    pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self {
        self.metrics_sink = Some(sink);
        self
    }

    pub fn with_metrics_sink_opt(
        mut self,
        sink: Option<Arc<dyn contextra_ports::MetricsSink>>,
    ) -> Self {
        self.metrics_sink = sink;
        self
    }

    pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock> {
        &self.clock
    }

    /// Erstellt einen neuen CheckpointGuard für einen Agenten-Schritt.
    pub async fn for_agent_step(storage: Arc<S>, tx: TxId) -> Result<Self> {
        let clock = Arc::new(SystemClock::new());
        Self::for_agent_step_with_clock(storage, tx, clock).await
    }

    pub async fn for_agent_step_with_registry(
        storage: Arc<S>,
        tx: TxId,
        orphan_registry: Arc<InstanceOrphanRegistry>,
    ) -> Result<Self> {
        let wall_ms = clock_timestamp_ms(orphan_registry.clock().as_ref());
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

    pub async fn for_agent_step_with_clock(
        storage: Arc<S>,
        tx: TxId,
        clock: Arc<dyn contextra_ports::Clock>,
    ) -> Result<Self> {
        let ns = "agent_step";
        let orphan_path = std::path::PathBuf::from(format!("{ns}_orphaned_checkpoints.json"));
        let registry = Arc::new(InstanceOrphanRegistry::new(orphan_path).with_clock(clock.clone()));
        let wall_ms = clock_timestamp_ms(clock.as_ref());
        let cp = StateCheckpoint {
            tx_id: tx,
            timestamp_ms: wall_ms,
            namespace: Some(ns.to_string()),
        };
        Ok(Self::with_registry(cp, storage, ns, registry))
    }

    pub fn checkpoint(&self) -> Result<&StateCheckpoint> {
        self.checkpoint
            .as_ref()
            .ok_or_else(|| ContextraError::Internal("Checkpoint already consumed".into()))
    }

    pub fn commit(mut self) -> Result<StateCheckpoint> {
        let cp = self
            .checkpoint
            .take()
            .ok_or_else(|| ContextraError::Internal("Checkpoint already consumed".into()))?;
        if let Some(ref sink) = self.metrics_sink {
            let elapsed = self.created_at.elapsed().as_secs_f64();
            sink.record_histogram(
                "checkpoint_duration_seconds",
                elapsed,
                &[("status", "commit")],
            );
        }
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

    /// Asynchroner Commit-Pfad, der Fehler beim Entleeren des Orphan-Registers direkt als `Result` propagiert.
    pub async fn commit_async(mut self) -> Result<StateCheckpoint> {
        let cp = self
            .checkpoint
            .take()
            .ok_or_else(|| ContextraError::Internal("Checkpoint already consumed".into()))?;
        if let Some(ref sink) = self.metrics_sink {
            let elapsed = self.created_at.elapsed().as_secs_f64();
            sink.record_histogram(
                "checkpoint_duration_seconds",
                elapsed,
                &[("status", "commit")],
            );
        }
        self.orphan_registry
            .flush_orphan_registry()
            .await
            .map_err(|e| {
                ContextraError::Internal(format!(
                    "Failed to flush orphan registry during commit: {e}"
                ))
            })?;
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
            if let Some(ref sink) = self.metrics_sink {
                let elapsed = self.created_at.elapsed().as_secs_f64();
                sink.record_histogram(
                    "checkpoint_duration_seconds",
                    elapsed,
                    &[("status", "rollback")],
                );
            }
            let last_tx = self.storage.last_tx_id().await?;
            if last_tx > cp.tx_id {
                return Err(ContextraError::Transaction(format!(
                    "Serialization barrier violation: Cannot rollback to TxId {} because newer committed transaction TxId {} exists in storage",
                    cp.tx_id.inner(),
                    last_tx.inner()
                )));
            }
            let res = self.storage.rollback_to_tx(cp.tx_id).await;
            if let Err(err) = self.orphan_registry.flush_orphan_registry().await {
                let flush_err = ContextraError::Internal(format!(
                    "Failed to flush orphan registry during CheckpointGuard::rollback: {err}"
                ));
                if res.is_ok() {
                    return Err(flush_err);
                }
            }
            res
        } else {
            Err(ContextraError::Internal(
                "Checkpoint already consumed".into(),
            ))
        }
    }

    /// Führt ein synchrones, blockierendes Rollback des Checkpoints aus.
    ///
    /// # Einschränkung
    /// Diese Methode darf **nicht** aus einem laufenden asynchronen Tokio-Kontext heraus aufgerufen werden.
    /// In asynchronem Kontext führt ein Aufruf von `rollback_blocking` zu einem Fehler
    /// [`ContextraError::Internal`], um Deadlocks und Tokio-Panics zu verhindern. Verwende in async-Kontexten
    /// stattdessen [`rollback`](Self::rollback).
    pub fn rollback_blocking(mut self) -> Result<()> {
        let cp = self
            .checkpoint
            .take()
            .ok_or_else(|| ContextraError::Internal("Checkpoint already consumed".into()))?;

        if let Some(ref sink) = self.metrics_sink {
            let elapsed = self.created_at.elapsed().as_secs_f64();
            sink.record_histogram(
                "checkpoint_duration_seconds",
                elapsed,
                &[("status", "rollback")],
            );
        }

        let storage = Arc::clone(&self.storage);

        crate::block_on_sync(async move {
            let last_tx = storage.last_tx_id().await?;
            if last_tx > cp.tx_id {
                return Err(ContextraError::Transaction(format!(
                    "Serialization barrier violation: Cannot rollback to TxId {} because newer committed transaction TxId {} exists in storage",
                    cp.tx_id.inner(),
                    last_tx.inner()
                )));
            }
            storage.rollback_to_tx(cp.tx_id).await
        })
    }
}

impl<S: contextra_ports::StorageEngine> Drop for CheckpointGuard<S> {
    fn drop(&mut self) {
        if let Some(mut cp) = self.checkpoint.take() {
            if let Some(ref sink) = self.metrics_sink {
                let elapsed = self.created_at.elapsed().as_secs_f64();
                sink.record_histogram(
                    "checkpoint_duration_seconds",
                    elapsed,
                    &[("status", "rollback")],
                );
            }
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
mod tests;
