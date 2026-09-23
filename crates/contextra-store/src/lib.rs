//! `Contextra` Store — LSM-Tree based storage engine.
//!
//! Provides persistent key-value storage with WAL, `MemTable`,
//! `SSTable`, and background compaction.
//!
//! # Checkpoint-Architektur
//! `contextra-store` enthält ein lokales, crate-internes Checkpointing (`pub(crate) mod checkpoint`).
//! Dieses dient ausschließlich als internes MVCC-Snapshot-Pinning (gekoppelt an `SnapshotRegistry`)
//! und darf niemals von außerhalb dieses Crates verwendet werden.
//! Die öffentliche, benannte Checkpoint-API gemäß ADR-011 ("Consolidated Checkpoint Subsystem Architecture")
//! befindet sich im Crate `contextra-checkpoint`.

// INVARIANT: LSM-Tree Storage Engine (Triebwerk — Layer 1).
// DATEN-PFAD: Client → TxBuffer → WAL → MemTable → SSTable → Compaction
// INVARIANTE: tokio::fs für Metadaten/Lifecycle, std::fs::File ausschließlich innerhalb spawn_blocking für Block-Level Random-Access.
// ANCHOR[INTEGRATION:STO-001] STATUS:RESOLVED (TS:2026-08-24T00:00:00Z)
// REVIEW-PASS[1/2] Systematischer Tiefen-Audit von contextra-store (PRÜFER-KONTEXT: FRESH) (TS: 2026-09-13T01:33:57Z) (SESSION: 60ca322c)
// MODUL-HIERARCHIE: lsm.rs orchestriert, memtable/wal/sstable sind Bausteine.

#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

#[cfg(not(loom))]
pub(crate) mod checkpoint;
#[cfg(not(loom))]
pub mod compaction;
pub mod kv_locks;
#[cfg(not(loom))]
pub mod lsm;
#[cfg(not(loom))]
pub mod manifest;
#[cfg(not(loom))]
pub mod memtable;
#[cfg(not(loom))]
pub mod sstable;
#[cfg(not(loom))]
pub mod system_pressure;
#[cfg(not(loom))]
pub mod tenant_codec;
pub(crate) mod util;
pub mod wal;

// WP-4.1 (UNIMPLEMENTED): mmap-basierter SSTable-Zugriff für Out-of-Core-Daten.
// Aktuell: reguläres tokio::fs/std::fs File-I/O in sstable.rs.
// Tracking-Issue: [ISSUE-NUMMER]

#[cfg(not(loom))]
pub use compaction::{CompactionConfig, CompactionEngine};
pub use kv_locks::{KeyGuard, KvKeyLocks, LockError, MultiKeyGuard};
#[cfg(not(loom))]
pub use lsm::{LsmConfig, LsmStorage};
#[cfg(not(loom))]
pub use manifest::{Manifest, ManifestEntry};
#[cfg(not(loom))]
pub use system_pressure::{PressureLevel, SystemPressure, SystemPressureMonitor};
#[cfg(not(loom))]
pub use tenant_codec::TenantKeyCodec;
