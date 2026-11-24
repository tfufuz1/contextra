#![deny(dead_code)]
//! # Checkpoint-Registry für Time-Travel und MVCC-basiertes Snapshotting
//!
//! Dieses Modul bildet die Wurzel der Crate `contextra-checkpoint`.
//!
//! ## Zweck & Rolle im System
//! Die Crate `contextra-checkpoint` gehört zu **Ring 1 (Persistenz)** der Contextra-Architektur.
//! Sie koordiniert konsistente Snapshots über heterogene Storage-Komponenten hinweg (LSM-Tree,
//! Vektor-Index, Graph-Speicher) und stellt deterministische Rollbacks für Agenten-Workflows bereit.
//!
//! ## Öffentliche Checkpoint-Subsystem Architecture (gemäß ADR-011)
//! `contextra-checkpoint` ist der **einzige öffentlich sichtbare Einstiegspunkt** für das Checkpoint-Konzept.
//! Es stellt den Trait [`crate::contextra_ports::CheckpointCoordinator`], die Registry [`PersistentCheckpointStore`]
//! sowie den RAII-Guard [`CheckpointGuard`] für automatisches Rollback bei Fehlern bereit.
//!
//! ## Architektur & Crash-Safety
//! `PersistentCheckpointStore` delegiert Persistenz an ein [`crate::contextra_ports::StorageEngine`]-Objekt
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

pub mod crate::contextra_ports;
pub mod crate::contextra_types;

pub use crate::contextra_types::{ContextraError, Result, TxId, WorkflowState};

// ANCHOR[contextra-checkpoint:MOD-001]
// NOTE: Die Aufteilung in Untermodule kapselt Guards, Cloner, Manifest, Metadaten, Orphan-Reaping und Store.
mod guard;
mod hardlink_cloner;
mod manifest;
mod meta;
mod orphan;
mod store;

// Öffentliche Re-Exports für bequemen Zugriff von außen ohne tiefe Pfade
pub use guard::{CheckpointGuard, PinGuard};
pub use hardlink_cloner::{
    CheckpointHardlinkCloner, DefaultHardlinkCloner, HardlinkCloneResult, SnapshotPinner,
};
pub use manifest::CheckpointManifest;
pub use meta::{CheckpointMeta, StateCheckpoint};
#[allow(deprecated)]
pub use orphan::{
    await_pending_rollbacks, clear_all_orphaned_checkpoints, clear_orphaned_checkpoint,
    get_orphaned_checkpoints, get_orphaned_checkpoints_for_namespace, global_orphan_registry,
    monotonic_timestamp_ms, monotonic_timestamp_ms_with, orphaned_checkpoint_count,
    pending_rollback_count, register_orphaned_checkpoint, register_pinned_seq_no_orphan,
    InstanceOrphanRegistry, OrphanRegistry, OrphanState, PinId, PinnedSeqNoOrphan, RecoveryReport,
};
pub use store::{CheckpointRegistry, PersistentCheckpointStore};
