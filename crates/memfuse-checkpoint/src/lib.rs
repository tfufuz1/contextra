//! Checkpoint-Registry für Time-Travel und MVCC-basiertes Snapshotting (gemäß ADR-011).
//!
//! # Öffentliche Checkpoint-Subsystem Architecture (ADR-011)
//! `memfuse-checkpoint` ist der **einzige öffentlich sichtbare Einstiegspunkt** für das Checkpoint-Konzept.
//! Es stellt den Trait [`memfuse_core::traits::CheckpointCoordinator`], die Registrie [`PersistentCheckpointStore`]
//! sowie den RAII-Guard [`CheckpointGuard`] für automatisches Rollback bei Fehlern bereit.
//!
//! **Hinweis zur Abgrenzung:**
//! Das Crate `memfuse-store` besitzt ein lokales, crate-internes Checkpoint-Modul (`pub(crate)`). Dieses ist ein reines
//! Implementierungsdetail für MVCC-Snapshot-Pinning (gekoppelt an `SnapshotRegistry`) und darf NIEMALS direkt von außerhalb
//! des Store-Crates verwendet werden.
//!
//! # Architektur & Crash-Safety
//! `PersistentCheckpointStore` delegiert Persistenz an ein [`memfuse_core::StorageEngine`]-Objekt
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

mod guard;
mod manifest;
mod meta;
mod orphan;
mod store;

pub use guard::{CheckpointGuard, PinGuard};
pub use manifest::CheckpointManifest;
pub use meta::{CheckpointMeta, StateCheckpoint};
#[allow(deprecated)]
pub use orphan::{
    await_pending_rollbacks, clear_all_orphaned_checkpoints, clear_orphaned_checkpoint,
    get_orphaned_checkpoints, get_orphaned_checkpoints_for_namespace, global_orphan_registry,
    orphaned_checkpoint_count, pending_rollback_count, register_orphaned_checkpoint,
    register_pinned_seq_no_orphan, InstanceOrphanRegistry, OrphanRegistry, OrphanState, PinId,
    PinnedSeqNoOrphan,
};
pub use store::{CheckpointRegistry, PersistentCheckpointStore};
