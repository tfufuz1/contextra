//! # Modul: internal.rs
//!
//! **Zweck:** Interne Hilfsfunktionen für Dokumenttransformation und Validierung.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, strikte Mandanten- und Indexisolation.

use crate::collection::Collection;
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::Result;

// ANCHOR[contextra-engine:COL-VAL-012]
/// Dokumentation für `validate_doc_id` im Rahmen der Collection- und Suchabstraktion.
pub(crate) fn validate_doc_id(id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(contextra_types::ContextraError::invalid_input(
            "Document ID cannot be empty",
        ));
    }
    if id.len() > 256 {
        return Err(contextra_types::ContextraError::invalid_input(
            "Document ID exceeds maximum length of 256 bytes",
        ));
    }
    if id.contains('\0') {
        return Err(contextra_types::ContextraError::invalid_input(
            "Document ID cannot contain null bytes",
        ));
    }
    Ok(())
}

// ANCHOR[contextra-engine:COL-VAL-031]
/// Dokumentation für `validate_embedding` im Rahmen der Collection- und Suchabstraktion.
pub(crate) fn validate_embedding(embedding: &[f32]) -> Result<()> {
    if embedding.is_empty() {
        return Err(contextra_types::ContextraError::invalid_input(
            "Embedding vector cannot be empty",
        ));
    }
    if embedding.iter().all(|&x| x == 0.0) {
        return Err(contextra_types::ContextraError::invalid_input(
            "Zero vector embeddings are not allowed in regular Collection insertion. Use put_kv for non-vector entries.",
        ));
    }
    if embedding.iter().any(|&x| !x.is_finite()) {
        return Err(contextra_types::ContextraError::invalid_input(
            "Embedding vector contains NaN or Infinite values",
        ));
    }
    Ok(())
}

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Applies configured backpressure delay if system pressure level is Critical (INV-PRESSURE-1).
    pub(crate) async fn apply_insert_backpressure(&self) {
        let delay_ms = self.config.read().backpressure_delay_ms;
        if let Some(delay_ms) = delay_ms {
            let rx_opt = self.pressure_rx.read().clone();
            if let Some(ref rx) = rx_opt {
                let level = rx.borrow().pressure_level;
                let level_val = match level {
                    contextra_store::PressureLevel::Normal => 0.0,
                    contextra_store::PressureLevel::Elevated => 1.0,
                    contextra_store::PressureLevel::Critical => 2.0,
                };
                crate::metrics_names::record_gauge_safe(
                    self.metrics_sink().as_ref(),
                    crate::metrics_names::BACKPRESSURE_LEVEL,
                    level_val,
                    &[],
                );
                if level == contextra_store::PressureLevel::Critical {
                    let backpressure_delay = std::time::Duration::from_millis(delay_ms);
                    tracing::warn!(
                        "Insert backpressure active (Critical pressure level); delaying {}ms",
                        backpressure_delay.as_millis()
                    );
                    tokio::time::sleep(backpressure_delay).await;
                }
            }
        }
    }

    /// Locks keys in deterministic sorted order to prevent deadlocks across batch operations.
    /// Deduplicates by lock shard index to prevent self-deadlock when multiple keys map to the same shard.
    ///
    /// INV-DAG-DEPS: Sperren-Hierarchie: collections (RwLock) → kv_locks (tokio Mutex) → embedder (RwLock).
    pub(crate) async fn lock_keys_sorted<'a>(
        &'a self,
        keys: impl IntoIterator<Item = &'a str>,
    ) -> Vec<crate::collection::kv_lock::KeyGuard<'a>> {
        self.kv_locks.lock_many_sorted(keys).await
    }
}
