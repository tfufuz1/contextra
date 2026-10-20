#![deny(dead_code)]
//! Checkpoint-Registry für Time-Travel und MVCC-basiertes Snapshotting (gemäß ADR-011).
//!
//! # Öffentliche Checkpoint-Subsystem Architecture (ADR-011)
//! `contextra-checkpoint` ist der **einzige öffentlich sichtbare Einstiegspunkt** für das Checkpoint-Konzept.
//! Es stellt den Trait [`contextra_ports::CheckpointCoordinator`], die Registrie [`PersistentCheckpointStore`]
//! sowie den RAII-Guard [`CheckpointGuard`] für automatisches Rollback bei Fehlern bereit.
//!
//! **Hinweis zur Abgrenzung:**
//! Das Crate `contextra-store` besitzt ein lokales, crate-internes Checkpoint-Modul (`pub(crate)`). Dieses ist ein reines
//! Implementierungsdetail für MVCC-Snapshot-Pinning (gekoppelt an `SnapshotRegistry`) und darf NIEMALS direkt von außerhalb
//! des Store-Crates verwendet werden.
//!
//! # Architektur & Crash-Safety
//! `PersistentCheckpointStore` delegiert Persistenz an ein [`contextra_ports::StorageEngine`]-Objekt
//! und cacht aktive Checkpoints in einem thread-sicheren In-Memory-Store (`parking_lot::RwLock`).
//!
//! **Drop-Semantik & Non-Blocking-I/O:**
//! `CheckpointGuard::drop()` und `PinGuard::drop()` führen ausschließlich In-Memory-Mutationen im
//! `InstanceOrphanRegistry` aus ohne blockierendes Disk-I/O. Persistenz auf Disk erfolgt entkoppelt
//! über Piggyback-Flushes (z.B. bei commit/rollback/unpin) sowie explizite Lifecycle-Flushes (`shutdown()`, `close()`).

#![forbid(unsafe_code)]

// FILE-CONTEXT
// STAND:       2026-09-15T15:09:43Z (SESSION: 2e382e86)
// ZWECK:       RAII CheckpointGuard + persistente Snapshot-Verwaltung (Re-Exports)
// INVARIANTEN: All public items must be re-exported from lib.rs for zero-breakage external imports.
// SIEHE AUCH:  ADR-011
// REVIEW-PASS[1/2] (TS: 2026-09-15T15:09:43Z) (SESSION: 2e382e86) PRÜFER-KONTEXT: FRESH
// REVIEW-PASS[2/2] (TS: 2026-09-16T16:23:23Z) (SESSION: 8d62c439) PRÜFER-KONTEXT: FRESH

pub mod copy_registry;
mod guard;
mod hardlink_cloner;
mod manifest;
mod meta;
mod orphan;
pub mod restore_filter;
mod store;
pub mod wiring;

pub use copy_registry::{file_id_for_name, CopyKind, CopyRegEntry, CopyStatus};
pub use guard::{CheckpointGuard, PinGuard};
pub use hardlink_cloner::{CheckpointHardlinkCloner, DefaultHardlinkCloner, HardlinkCloneResult};
pub use manifest::CheckpointManifest;
pub use meta::{CheckpointMeta, StateCheckpoint};
pub use orphan::{
    monotonic_timestamp_ms, monotonic_timestamp_ms_with, InstanceOrphanRegistry, OrphanState,
    PinId, PinnedSeqNoOrphan,
};
#[allow(deprecated)]
pub use orphan::{
    await_pending_rollbacks, clear_all_orphaned_checkpoints, clear_orphaned_checkpoint,
    get_orphaned_checkpoints, get_orphaned_checkpoints_for_namespace, global_orphan_registry,
    orphaned_checkpoint_count, pending_rollback_count, register_orphaned_checkpoint,
    register_pinned_seq_no_orphan, OrphanRegistry,
};
pub use restore_filter::{filter_restore_records, CpyRestoreFilter};
#[cfg(any(test, feature = "test-utils", feature = "testing"))]
pub use restore_filter::NoOpRestoreFilter;
pub use store::{purge_registered_checkpoint, CheckpointRegistry, PersistentCheckpointStore};

pub(crate) fn block_on_sync<F, T>(future: F) -> contextra_types::Result<T>
where
    F: std::future::Future<Output = contextra_types::Result<T>> + Send + 'static,
    T: Send + 'static,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
            tokio::task::block_in_place(|| handle.block_on(future))
        } else {
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;
                rt.block_on(future)
            })
            .join()
            .map_err(|_| {
                contextra_types::ContextraError::Internal(
                    "Thread panic during sync execution in async context".into(),
                )
            })?
        }
    } else {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                contextra_types::ContextraError::Internal(format!(
                    "Failed to create Tokio runtime: {e}"
                ))
            })?;
        rt.block_on(future)
    }
}
