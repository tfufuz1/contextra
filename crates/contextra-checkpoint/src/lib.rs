#![deny(missing_docs)]
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
//! Es stellt den Trait `CheckpointCoordinator` (unter `compat`), die Registry [`PersistentCheckpointStore`]
//! sowie den RAII-Guard [`CheckpointGuard`] für automatisches Rollback bei Fehlern bereit.
//!
//! **Hinweis zur Abgrenzung:**
//! Das Crate `contextra-store` besitzt ein lokales, crate-internes Checkpoint-Modul (`pub(crate)`). Dieses ist ein reines
//! Implementierungsdetail für MVCC-Snapshot-Pinning (gekoppelt an `SnapshotRegistry`) und darf NIEMALS direkt von außerhalb
//! des Store-Crates verwendet werden.
//!
//! ## Architektur & Crash-Safety
//! `PersistentCheckpointStore` delegiert Persistenz an ein [`SnapshotSource`]-Objekt
//! und cacht aktive Checkpoints in einem thread-sicheren In-Memory-Store (`parking_lot::RwLock`).
//!
//! **Drop-Semantik & Non-Blocking-I/O:**
//! `CheckpointGuard::drop()` und `PinGuard::drop()` führen ausschließlich In-Memory-Mutationen im
//! `InstanceOrphanRegistry` aus ohne blockierendes Disk-I/O. Persistenz auf Disk erfolgt entkoppelt
//! über Piggyback-Flushes (z.B. bei commit/rollback/unpin) sowie explizite Lifecycle-Flushes (`shutdown()`, `close()`).
//!
//! ## Einbindung & Abhängigkeiten
//! - **Abhängigkeiten (normal):** `contextra-types`, `contextra-ports`, `bytes`, `tokio`, `tracing`, `serde`, `serde_json`, `parking_lot`, `blake3`, `thiserror`.
//! - **Abhängigkeiten (dev):** `contextra-testkit`, `tempfile`, `proptest`, `criterion`.
//! - **Wird eingebunden von:** `contextra-agent`, `contextra-db`, `contextra-engine`.

#![forbid(unsafe_code)]

#[cfg(feature = "workspace")]
pub mod compat;
pub mod contract;
mod error;
mod fs_seam;
mod guard;
mod hardlink_cloner;
mod manifest;
mod meta;
mod orphan;
mod snapshot_source;
mod store;
mod vfs;

pub use vfs::CheckpointVfs;

// Öffentliche Re-Exports für bequemen Zugriff von außen ohne tiefe Pfade
pub use error::{CheckpointError, CheckpointResult};
pub use fs_seam::{FsSeam, StdFsSeam};
pub use guard::{CheckpointGuard, PinGuard};
pub use hardlink_cloner::{
    CheckpointHardlinkCloner, DefaultHardlinkCloner, HardlinkCloneResult, SnapshotPinner,
};
pub use manifest::CheckpointManifest;
pub use meta::{CheckpointMeta, StateCheckpoint, CURRENT_ID_WIDTH_BITS};
#[allow(deprecated)]
pub use orphan::{
    await_pending_rollbacks, clear_all_orphaned_checkpoints, clear_orphaned_checkpoint,
    get_orphaned_checkpoints, get_orphaned_checkpoints_for_namespace, global_orphan_registry,
    monotonic_timestamp_ms, monotonic_timestamp_ms_with, orphaned_checkpoint_count,
    pending_rollback_count, register_orphaned_checkpoint, register_pinned_seq_no_orphan,
    InstanceOrphanRegistry, OrphanRegistry, OrphanState, PinId, PinnedSeqNoOrphan, RecoveryReport,
};
pub use snapshot_source::{BoxFuture as LocalBoxFuture, SnapshotSource};
pub use store::{CheckpointRegistry, PersistentCheckpointStore};
